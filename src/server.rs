use std::convert::Infallible;
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

use crate::board::Board;

/// How often to reread the board to catch changes made elsewhere.
const REFRESH: Duration = Duration::from_secs(30);

/// Board with the server's copy of what it shows.
pub struct Server {
    /// Board locked for each read or send so a slow read can't overwrite a newer send.
    board: Mutex<Board>,

    /// Latest snapshot for requests and open pages.
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
    /// Constructor.
    pub fn new(board: Board) -> Server {
        Server {
            board: Mutex::new(board),
            snapshot: Sender::new(Snapshot::default()),
        }
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

    /// Get the latest snapshot.
    pub fn snapshot(&self) -> Snapshot {
        *self.snapshot.borrow()
    }

    /// Record a board contact and notify open pages if anything changed.
    fn record(&self, grid: Option<Grid>, live: bool) {
        self.snapshot.send_if_modified(|snapshot| {
            let next = Snapshot {
                grid: grid.or(snapshot.grid),
                live,
            };
            let changed = *snapshot != next;
            *snapshot = next;

            changed
        });
    }

    /// Show a grid and record the result.
    async fn show(&self, grid: Grid) -> Result<()> {
        let board = self.board.lock().await;
        match board.set_grid(&grid).await {
            Ok(()) => {
                self.record(Some(grid), true);

                Ok(())
            }
            Err(err) => {
                self.record(None, false);

                Err(err)
            }
        }
    }
}

/// Serve the web page and HTTP API until the process exits.
pub async fn serve(board: Board, host: &str, port: u16) -> Result<()> {
    let server = Arc::new(Server::new(board));
    tokio::spawn(refresh(server.clone()));
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    eprintln!("Listening on {host}:{port}");
    axum::serve(listener, server.router()).await?;

    Ok(())
}

/// Reread the board now and every `REFRESH`.
async fn refresh(server: Arc<Server>) {
    let mut ticks = tokio::time::interval(REFRESH);
    let mut failing = false;
    loop {
        ticks.tick().await;
        let board = server.board.lock().await;
        match board.get_grid().await {
            Ok(grid) => {
                failing = false;
                server.record(Some(grid), true);
            }
            Err(err) => {
                if !failing {
                    eprintln!("Can't read the board: {err:#}");
                }
                failing = true;
                server.record(None, false);
            }
        }
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
