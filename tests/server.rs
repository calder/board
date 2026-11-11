use std::sync::{Arc, Mutex};

use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Method, Request, StatusCode};
use board::{Board, Server};
use serde_json::{Value, json};
use tower::ServiceExt;
use vestaboard::{COLS, Grid, ROWS};

/// Make a server with a dry-run board.
fn test_server() -> Arc<Server> {
    Arc::new(Server::new(Board::Test {
        shown: Mutex::new(Grid::EMPTY),
    }))
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

#[tokio::test]
async fn shows_grid() {
    let server = test_server();
    let mut rows = blank();
    rows[0][0] = 8;
    rows[0][1] = 9;
    rows[5] = vec![63; COLS];
    assert_eq!(
        send(&server, Method::POST, "/grid", &body(&rows)).await,
        (StatusCode::NO_CONTENT, String::new())
    );
    let (status, snapshot) = send(&server, Method::GET, "/board", "").await;
    assert_eq!(status, StatusCode::OK);
    let snapshot: Value = serde_json::from_str(&snapshot).unwrap();
    assert_eq!(snapshot["grid"], json!(rows));
    assert_eq!(snapshot["live"], true);
}

#[tokio::test]
async fn message_returns_no_content() {
    let server = test_server();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/message")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"text": "HI"}"#))
        .unwrap();
    let response = server.clone().router().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert!(server.snapshot().grid.is_some());
}

#[tokio::test]
async fn rejects_bad_shapes() {
    let server = test_server();
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
    assert!(server.snapshot().grid.is_none());
}

#[tokio::test]
async fn rejects_unknown_codes() {
    let server = test_server();
    let mut rows = blank();
    rows[1][3] = 99;
    let unknown = rejected(&server, &body(&rows)).await;
    assert!(
        unknown.starts_with("invalid request: grid: row 1, column 3: unknown character code 99"),
        "{unknown}"
    );
    assert!(server.snapshot().grid.is_none());
}

#[tokio::test]
async fn rejects_bad_json() {
    let server = test_server();
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
    assert!(server.snapshot().grid.is_none());
}
