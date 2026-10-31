use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use axum::extract::{Json, State};
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use rust_embed::Embed;
use vestaboard::Message;

use crate::board::Board;

type Reply = Result<String, (StatusCode, String)>;

/// Web page built by `trunk build --release` in `web/`.
#[derive(Embed)]
#[folder = "web/dist/"]
#[allow_missing = true]
struct Page;

/// Serve the web page and HTTP API until the process exits.
pub async fn serve(board: Board, host: &str, port: u16) -> Result<()> {
    let app = Router::new()
        .route("/message", post(post_message))
        .fallback(page)
        .with_state(Arc::new(board));
    let listener = tokio::net::TcpListener::bind((host, port)).await?;
    eprintln!("Listening on {host}:{port}");
    axum::serve(listener, app).await?;

    Ok(())
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

async fn post_message(State(board): State<Arc<Board>>, Json(message): Json<Message>) -> Reply {
    let grid = message
        .to_grid()
        .map_err(|err| (StatusCode::BAD_REQUEST, format!("{err}\n")))?;
    board.show_grid(&grid).await.map_err(internal)?;

    Ok("Shown\n".into())
}

fn internal(err: anyhow::Error) -> (StatusCode, String) {
    eprintln!("{err:#}");

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:#}\n"))
}
