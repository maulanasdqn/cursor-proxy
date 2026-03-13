use std::sync::Arc;

use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use cursor_proxy_types::Config;

pub async fn auth(config: axum::extract::State<Arc<Config>>, req: Request, next: Next) -> Response {
    let api_key = match &config.api_key {
        Some(key) => key,
        None => return next.run(req).await,
    };

    let auth_header = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .or_else(|| auth_header.and_then(|h| h.strip_prefix("bearer ")));

    match token {
        Some(t) if t == api_key => next.run(req).await,
        _ => (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "error": {
                    "message": "Invalid or missing API key",
                    "code": "unauthorized"
                }
            })),
        )
            .into_response(),
    }
}
