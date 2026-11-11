use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::extract::Json;
use axum::extract::State as AxumState;
use axum::extract::rejection::JsonRejection;
use axum::http::header::CONTENT_TYPE;
use axum::http::{StatusCode, Uri};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use rust_embed::Embed;
use serde::Deserialize;
use tokio::sync::watch::Sender;
use tokio_stream::wrappers::WatchStream;
use tokio_stream::{Stream, StreamExt};
use vestaboard::{Grid, Message};

use crate::board::Board;
use crate::state::State;

/// Wait between board writes.
const PERIOD: Duration = Duration::from_secs(1);

/// Server with exclusive control of a board.
pub struct Server {
    /// Board kept in sync with the state.
    board: Box<dyn Board>,

    /// File the state persists to.
    path: PathBuf,

    /// App state for requests and open pages.
    state: Sender<State>,
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
    /// Start a server restoring its state from `path` and keeping `board` in sync with it.
    pub fn start(board: Box<dyn Board>, path: PathBuf) -> Result<Arc<Server>> {
        let server = Arc::new(Server {
            board,
            state: Sender::new(State::load(&path)?),
            path,
        });
        tokio::spawn(update(server.clone()));

        Ok(server)
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

    /// Get the state.
    pub fn state(&self) -> State {
        *self.state.borrow()
    }

    /// Make `grid` the desired grid and save it.
    pub fn show(&self, grid: Grid) -> Result<()> {
        let mut result = Ok(());
        // Save while holding the state so concurrent requests save in order.
        self.state.send_if_modified(|state| {
            if state.grid == Some(grid) {
                return false;
            }
            state.grid = Some(grid);
            result = state.save(&self.path);

            true
        });

        result
    }
}

/// Serve the web page and HTTP API until the process exits.
pub async fn serve(board: Box<dyn Board>, state: PathBuf, host: &str, port: u16) -> Result<()> {
    let server = Server::start(board, state)?;
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    log::info!("Listening on {host}:{port}");
    axum::serve(listener, server.router()).await?;

    Ok(())
}

/// Keep the board showing the desired grid.
async fn update(server: Arc<Server>) {
    loop {
        if let Some(grid) = server.state().grid
            && let Err(err) = server.board.set_grid(&grid).await
        {
            log::error!("Error writing to board: {err:#}");
        }
        tokio::time::sleep(PERIOD).await;
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

async fn get_grid(AxumState(server): AxumState<Arc<Server>>) -> Json<Option<Grid>> {
    Json(server.state().grid)
}

/// Stream the desired grid now and whenever it changes.
async fn board_events(
    AxumState(server): AxumState<Arc<Server>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = WatchStream::new(server.state.subscribe())
        .filter_map(|state| state.grid)
        .map(|grid| {
            Ok(Event::default().data(serde_json::to_string(&grid).expect("grids serialize")))
        });

    Sse::new(events).keep_alive(KeepAlive::default())
}

async fn post_message(
    AxumState(server): AxumState<Arc<Server>>,
    Json(message): Json<Message>,
) -> Reply {
    let grid = message
        .to_grid()
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}\n")))?;
    server.show(grid).map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Show a full grid of character codes.
async fn post_grid(
    AxumState(server): AxumState<Arc<Server>>,
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
    server.show(request.grid).map_err(internal)?;

    Ok(StatusCode::NO_CONTENT)
}

fn internal(err: anyhow::Error) -> (StatusCode, String) {
    log::error!("{err:#}");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}\n"))
}
