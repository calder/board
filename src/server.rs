use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use rust_embed::Embed;
use serde::Deserialize;
use tokio::sync::{Mutex, watch};
use tokio_stream::wrappers::WatchStream;
use tokio_stream::{Stream, StreamExt};
use vestaboard::{Grid, Message, Snapshot};

use crate::board::Board;

/// How often to reread the board to catch changes made elsewhere.
const REFRESH: Duration = Duration::from_secs(30);

type Reply = Result<String, (StatusCode, String)>;

/// Web page built by `trunk build --release` in `web/`.
#[derive(Embed)]
#[folder = "web/dist/"]
#[allow_missing = true]
struct Page;

/// Board with the server's copy of what it shows.
struct Server {
    /// Board locked for each read or send so a slow read can't overwrite a newer send.
    board: Mutex<Board>,

    /// Latest snapshot for requests and open pages.
    snapshot: watch::Sender<Snapshot>,
}

impl Server {
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

/// Full grid to show.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GridRequest {
    /// Character codes, 6 rows of 22.
    grid: Vec<Vec<u32>>,
}

/// Serve the web page and HTTP API until the process exits.
pub async fn serve(board: Board, host: &str, port: u16) -> Result<()> {
    let server = Arc::new(Server {
        board: Mutex::new(board),
        snapshot: watch::Sender::new(Snapshot::default()),
    });
    tokio::spawn(refresh(server.clone()));
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    eprintln!("Listening on {host}:{port}");
    axum::serve(listener, app(server)).await?;

    Ok(())
}

/// Routes for the web page and HTTP API.
fn app(server: Arc<Server>) -> Router {
    Router::new()
        .route("/board", get(get_board).post(post_board))
        .route("/board/events", get(board_events))
        .route("/message", post(post_message))
        .fallback(page)
        .with_state(server)
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

    (
        [(header::CONTENT_TYPE, file.metadata.mimetype())],
        file.data,
    )
        .into_response()
}

async fn get_board(State(server): State<Arc<Server>>) -> Json<Snapshot> {
    Json(*server.snapshot.borrow())
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

    Ok("Shown\n".into())
}

/// Show a full grid of character codes.
async fn post_board(
    State(server): State<Arc<Server>>,
    request: Result<Json<GridRequest>, JsonRejection>,
) -> Reply {
    let Json(request) = request.map_err(|rejection| {
        // Prefer serde's message, which names the bad field, over axum's summary.
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
    let grid = Grid::from_code_rows(&request.grid)
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}\n")))?;
    server.show(grid).await.map_err(internal)?;

    Ok("Shown\n".into())
}

fn internal(err: anyhow::Error) -> (StatusCode, String) {
    eprintln!("{err:#}");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}\n"))
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Method, Request};
    use tower::ServiceExt;

    use super::*;

    /// Server with a dry-run board.
    fn test_server() -> Arc<Server> {
        Arc::new(Server {
            board: Mutex::new(Board::Test {
                shown: std::sync::Mutex::new(Grid::EMPTY),
            }),
            snapshot: watch::Sender::new(Snapshot::default()),
        })
    }

    /// Send a request and return its status and body.
    async fn send(server: &Arc<Server>, method: Method, body: &str) -> (StatusCode, String) {
        let request = Request::builder()
            .method(method)
            .uri("/board")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let response = app(server.clone()).oneshot(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    /// Request body for a grid of rows.
    fn body(rows: &[Vec<u32>]) -> String {
        serde_json::json!({ "grid": rows }).to_string()
    }

    #[tokio::test]
    async fn shows_grid() {
        let server = test_server();
        let mut rows = vec![vec![0; 22]; 6];
        rows[0][0] = 8;
        rows[0][1] = 9;
        rows[5] = vec![63; 22];
        assert_eq!(
            send(&server, Method::POST, &body(&rows)).await,
            (StatusCode::OK, "Shown\n".into())
        );
        let (status, snapshot) = send(&server, Method::GET, "").await;
        assert_eq!(status, StatusCode::OK);
        let snapshot: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        assert_eq!(snapshot["grid"], serde_json::json!(rows));
        assert_eq!(snapshot["live"], true);
    }

    #[tokio::test]
    async fn rejects_bad_grids() {
        let server = test_server();
        let rejected = |body: String| {
            let server = server.clone();
            async move {
                let (status, body) = send(&server, Method::POST, &body).await;
                assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

                body
            }
        };
        let rows = vec![vec![0; 22]; 6];
        assert_eq!(rejected(body(&rows[..5])).await, "expected 6 rows, got 5\n");
        let mut wide = rows.clone();
        wide[4].push(0);
        assert_eq!(
            rejected(body(&wide)).await,
            "row 4: expected 22 columns, got 23\n"
        );
        let mut unknown = rows.clone();
        unknown[1][3] = 99;
        assert_eq!(
            rejected(body(&unknown)).await,
            "row 1, column 3: unknown character code 99\n"
        );
        let extra = serde_json::json!({ "grid": rows, "live": true }).to_string();
        let extra = rejected(extra).await;
        assert!(
            extra.starts_with("invalid request: ") && extra.contains("unknown field `live`"),
            "{extra}"
        );
        assert!(
            rejected("{}".into())
                .await
                .starts_with("invalid request: missing field `grid`")
        );
        assert!(
            rejected(r#"{"grid": [[-1]]}"#.into())
                .await
                .starts_with("invalid request: ")
        );
        assert_eq!(
            server.snapshot.borrow().grid,
            None,
            "nothing reached the board"
        );
    }
}
