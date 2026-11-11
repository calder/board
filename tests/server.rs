use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use anyhow::{Result, bail};
use async_trait::async_trait;
use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, Request, StatusCode};
use board::{Server, State};
use mockall::mock;
use serde_json::{Value, json};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio_stream::StreamExt;
use tower::ServiceExt;
use vestaboard::{COLS, Grid, ROWS};

mock! {
    Board {}

    #[async_trait]
    impl board::Board for Board {
        async fn set_grid(&self, grid: &Grid) -> Result<()>;
    }
}

/// Make a fresh state file path.
fn state_path() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let n = NEXT.fetch_add(1, Ordering::SeqCst);

    std::env::temp_dir().join(format!("board-test-{}-{n}/state.json", std::process::id()))
}

/// Start a server whose mock board fails its first `failures` writes and reports the rest.
fn start(path: PathBuf, failures: usize) -> (Arc<Server>, UnboundedReceiver<Grid>) {
    let (sender, writes) = unbounded_channel();
    let calls = AtomicUsize::new(0);
    let mut board = MockBoard::new();
    board.expect_set_grid().returning(move |grid| {
        if calls.fetch_add(1, Ordering::SeqCst) < failures {
            bail!("test failure");
        }
        sender.send(*grid).unwrap();

        Ok(())
    });

    (Server::start(Box::new(board), path).unwrap(), writes)
}

/// Start a server with fresh state and a working mock board.
fn test_server() -> (Arc<Server>, UnboundedReceiver<Grid>) {
    start(state_path(), 0)
}

/// Let the server run for a while.
async fn settle() {
    tokio::time::sleep(Duration::from_secs(60 * 60)).await;
}

/// Send a request and return its status and body.
async fn send(server: &Arc<Server>, method: Method, uri: &str, body: &str) -> (StatusCode, String) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = server.clone().router().oneshot(request).await.unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();

    (status, String::from_utf8(body.to_vec()).unwrap())
}

/// Post a body and expect a 400 with its message.
async fn rejected(server: &Arc<Server>, body: &str) -> String {
    let (status, body) = send(server, Method::POST, "/grid", body).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

    body
}

/// Make a request body for rows of codes.
fn body(rows: &[Vec<u32>]) -> String {
    json!({ "grid": rows }).to_string()
}

/// Make blank rows.
fn blank() -> Vec<Vec<u32>> {
    vec![vec![0; COLS]; ROWS]
}

#[tokio::test(start_paused = true)]
async fn starts_from_blank_grid() {
    let path = state_path();
    let (server, mut writes) = start(path.clone(), 0);
    assert_eq!(server.state().grid, Grid::EMPTY);
    assert_eq!(writes.recv().await, Some(Grid::EMPTY));
    assert!(!path.exists());
    let (status, grid) = send(&server, Method::GET, "/grid", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&grid).unwrap(),
        json!(blank())
    );

    let request = Request::builder()
        .uri("/events")
        .body(Body::from(""))
        .unwrap();
    let response = server.clone().router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let mut stream = response.into_body().into_data_stream();
    let chunk = stream.next().await.unwrap().unwrap();
    let event = String::from_utf8(chunk.to_vec()).unwrap();
    let expected = format!("data: {}", json!(blank()));
    assert!(event.contains(&expected), "{event}");
}

#[tokio::test(start_paused = true)]
async fn shows_grid() {
    let (server, mut writes) = test_server();
    assert_eq!(writes.recv().await, Some(Grid::EMPTY));
    let mut rows = blank();
    rows[0][0] = 8;
    rows[0][1] = 9;
    rows[5] = vec![63; COLS];
    assert_eq!(
        send(&server, Method::POST, "/grid", &body(&rows)).await,
        (StatusCode::NO_CONTENT, String::new())
    );
    let (status, grid) = send(&server, Method::GET, "/grid", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(serde_json::from_str::<Value>(&grid).unwrap(), json!(rows));
    let written = writes.recv().await.unwrap();
    assert_eq!(serde_json::to_value(written).unwrap(), json!(rows));
}

#[tokio::test(start_paused = true)]
async fn shows_message() {
    let (server, mut writes) = test_server();
    assert_eq!(writes.recv().await, Some(Grid::EMPTY));
    let (status, _) = send(&server, Method::POST, "/message", r#"{"text": "HI"}"#).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let grid = Grid::from_text("HI").unwrap();
    assert_eq!(server.state().grid, grid);
    assert_eq!(writes.recv().await, Some(grid));
}

#[tokio::test(start_paused = true)]
async fn rejects_bad_shapes() {
    let (server, mut writes) = test_server();
    let rows = blank();
    let short = rejected(&server, &body(&rows[..5])).await;
    assert!(short.contains("grid: expected 6 rows, got 5"), "{short}");
    let mut wide = blank();
    wide[4].push(0);
    let wide = rejected(&server, &body(&wide)).await;
    assert!(
        wide.contains("grid: row 4: expected 22 columns, got 23"),
        "{wide}"
    );
    assert_eq!(server.state().grid, Grid::EMPTY);
    settle().await;
    while let Ok(grid) = writes.try_recv() {
        assert_eq!(grid, Grid::EMPTY);
    }
}

#[tokio::test(start_paused = true)]
async fn rejects_unknown_codes() {
    let (server, _) = test_server();
    let mut rows = blank();
    rows[1][3] = 99;
    let unknown = rejected(&server, &body(&rows)).await;
    assert!(
        unknown.starts_with("invalid request: grid: row 1, column 3: unknown character code 99"),
        "{unknown}"
    );
    assert_eq!(server.state().grid, Grid::EMPTY);
}

#[tokio::test(start_paused = true)]
async fn rejects_bad_json() {
    let (server, _) = test_server();
    let extra = json!({ "grid": blank(), "live": true }).to_string();
    let extra = rejected(&server, &extra).await;
    assert!(
        extra.starts_with("invalid request: ") && extra.contains("unknown field `live`"),
        "{extra}"
    );
    let missing = rejected(&server, "{}").await;
    assert!(missing.contains("missing field `grid`"), "{missing}");
    let negative = rejected(&server, r#"{"grid": [[-1]]}"#).await;
    assert!(negative.starts_with("invalid request: "), "{negative}");
    assert_eq!(server.state().grid, Grid::EMPTY);
}

#[tokio::test(start_paused = true)]
async fn restores_state() {
    let path = state_path();
    let mut rows = blank();
    rows[2][8] = 8;
    let grid: Grid = serde_json::from_value(json!(rows)).unwrap();
    let (server, mut writes) = start(path.clone(), 0);
    server.show(grid).unwrap();
    assert_eq!(writes.recv().await, Some(grid));
    assert_eq!(State::load(&path).unwrap(), State { grid });
    let (server, mut writes) = start(path, 0);
    assert_eq!(server.state().grid, grid);
    assert_eq!(writes.recv().await, Some(grid));
}

#[tokio::test(start_paused = true)]
async fn keeps_writing_grid() {
    let (server, mut writes) = test_server();
    let grid = Grid::from_text("HI").unwrap();
    server.show(grid).unwrap();
    assert_eq!(writes.recv().await, Some(grid));
    assert_eq!(writes.recv().await, Some(grid));
}

#[tokio::test(start_paused = true)]
async fn retries_after_failure() {
    let (server, mut writes) = start(state_path(), 3);
    let grid = Grid::from_text("HI").unwrap();
    server.show(grid).unwrap();
    assert_eq!(writes.recv().await, Some(grid));
}

#[tokio::test(start_paused = true)]
async fn writes_newer_grid() {
    let (server, mut writes) = test_server();
    let grid = Grid::from_text("HI").unwrap();
    server.show(grid).unwrap();
    assert_eq!(writes.recv().await, Some(grid));
    let grid = Grid::from_text("BYE").unwrap();
    server.show(grid).unwrap();
    assert_eq!(writes.recv().await, Some(grid));
}
