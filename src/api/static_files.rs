use rust_embed::RustEmbed;
use axum::{
    response::{Html, IntoResponse, Response},
    http::{header, StatusCode, Uri},
};

#[derive(RustEmbed)]
#[folder = "frontend/dist/voltexdl/browser/"]
struct Asset;

pub async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/');

    if path.is_empty() {
        return serve_index().await;
    }

    match Asset::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => serve_index().await,
    }
}

async fn serve_index() -> Response {
    match Asset::get("index.html") {
        Some(content) => Html(content.data).into_response(),
        None => (StatusCode::NOT_FOUND, "Frontend not built. Run 'cd frontend && yarn build'").into_response(),
    }
}
