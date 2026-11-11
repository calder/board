use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use rust_embed::Embed;
use serde::Deserialize;
use tokio::sync::Mutex;
use tokio::sync::watch::Sender;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::{Stream, StreamExt};
use vestaboard::{Grid, Message, Snapshot};

use crate::app_state::AppState;
use crate::board::Board;

/// How often to check for a failed write to retry.
const RETRY: Duration = Duration::from_secs(30);

/// Longest wait between retries.
const MAX_BACKOFF: Duration = Duration::from_secs(10 * 60);

/// Server with exclusive control of a board.
pub struct Server {
    /// Board.
    board: Board,

    /// File the desired state persists to.
    path: PathBuf,

    /// Grid last written to the board locked for each write.
    written: Mutex<Option<Grid>>,

    /// Desired grid and board health for requests and open pages.
    snapshot: Sender<Snapshot>,
}

type Reply = Result<StatusCode, (StatusCode, String)>;

/// Web page built by `trunk build --release` in `web/`.
#[derive(Embed)]
#[folder = "web/dist/"]
#[allow_missing = true]
struct Page;

/// Full grid to show.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GridRequest {
    grid: Grid,
}

impl Server {
    /// Construct a server restoring the desired state from `path`.
    pub fn new(board: Board, path: PathBuf) -> Result<Server> {
        let state = AppState::load(&path)?;

        Ok(Server {
            board,
            path,
            written: Mutex::new(None),
            snapshot: Sender::new(Snapshot {
                grid: state.grid,
                live: false,
            }),
        })
    }

    /// Route the web page and HTTP API.
    pub fn router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/events", get(board_events))
            .route("/grid", get(get_grid).post(post_grid))
            .route("/message", post(post_message))
            .fallback(page)
            .with_state(self)
    }

    /// Get the board.
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Get the latest snapshot.
    pub fn snapshot(&self) -> Snapshot {
        *self.snapshot.borrow()
    }

    /// Make `grid` the desired grid then save and write it.
    pub async fn show(&self, grid: Grid) -> Result<()> {
        self.update(|snapshot| snapshot.grid = Some(grid));
        let mut written = self.written.lock().await;
        // A newer request may have replaced the desired grid while this one waited.
        AppState {
            grid: self.snapshot().grid,
        }
        .save(&self.path)?;

        self.write(&mut written).await
    }

    /// Write the desired grid unless it was already written.
    pub async fn sync(&self) -> Result<()> {
        let mut written = self.written.lock().await;

        self.write(&mut written).await
    }

    /// Write the desired grid to the locked board unless it was already written.
    async fn write(&self, written: &mut Option<Grid>) -> Result<()> {
        let Some(grid) = self.snapshot().grid else {
            return Ok(());
        };
        if *written == Some(grid) {
            return Ok(());
        }
        let result = self.board.set_grid(&grid).await;
        *written = result.is_ok().then_some(grid);
        self.update(|snapshot| snapshot.live = result.is_ok());

        result
    }

    /// Change the snapshot and notify open pages if anything changed.
    fn update(&self, change: impl FnOnce(&mut Snapshot)) {
        self.snapshot.send_if_modified(|snapshot| {
            let before = *snapshot;
            change(snapshot);

            *snapshot != before
        });
    }
}

/// Serve the web page and HTTP API until the process exits.
pub async fn serve(board: Board, state: PathBuf, host: &str, port: u16) -> Result<()> {
    let server = Arc::new(Server::new(board, state)?);
    tokio::spawn(control(server.clone()));
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    eprintln!("Listening on {host}:{port}");
    axum::serve(listener, server.router()).await?;

    Ok(())
}

/// Write the desired grid now and retry failures backing off up to `MAX_BACKOFF`.
async fn control(server: Arc<Server>) {
    let mut wait = RETRY;
    let mut failing = false;
    loop {
        match server.sync().await {
            Ok(()) => {
                failing = false;
                wait = RETRY;
            }
            Err(err) => {
                if !failing {
                    eprintln!("Can't write the board: {err:#}");
                }
                failing = true;
                wait = (wait * 2).min(MAX_BACKOFF);
            }
        }
        tokio::time::sleep(wait).await;
    }
}

async fn page(uri: Uri) -> Response {
    let path = match uri.path().trim_start_matches('/') {
        "" => "index.html",
        path => path,
    };
    let Some(file) = Page::get(path) else {
        let hint = if path == "index.html" {
            "Build the page first with `trunk build --release` in web/\n"
        } else {
            "Not found\n"
        };

        return (StatusCode::NOT_FOUND, hint).into_response();
    };

    ([(CONTENT_TYPE, file.metadata.mimetype())], file.data).into_response()
}

async fn get_grid(State(server): State<Arc<Server>>) -> Json<Snapshot> {
    Json(server.snapshot())
}

/// Stream the snapshot now and whenever it changes.
async fn board_events(
    State(server): State<Arc<Server>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = WatchStream::new(server.snapshot.subscribe()).map(|snapshot| {
        Ok(Event::default().data(serde_json::to_string(&snapshot).expect("snapshots serialize")))
    });

    Sse::new(events).keep_alive(KeepAlive::default())
}

async fn post_message(State(server): State<Arc<Server>>, Json(message): Json<Message>) -> Reply {
    let grid = message
        .to_grid()
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}\n")))?;
    server.show(grid).await.map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Show a full grid of character codes.
async fn post_grid(
    State(server): State<Arc<Server>>,
    request: Result<Json<GridRequest>, JsonRejection>,
) -> Reply {
    let Json(request) = request.map_err(|rejection| {
        // Prefer serde's message naming the bad field over axum's summary.
        let mut detail = rejection.body_text();
        let mut source = std::error::Error::source(&rejection);
        while let Some(err) = source {
            detail = err.to_string();
            source = err.source();
        }

        (
            StatusCode::BAD_REQUEST,
            format!("invalid request: {detail}\n"),
        )
    })?;
    server.show(request.grid).await.map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}

fn internal(err: anyhow::Error) -> (StatusCode, String) {
    eprintln!("{err:#}");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}\n"))
}
