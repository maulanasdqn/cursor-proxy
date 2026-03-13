use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use cursor_proxy_types::Config;

use crate::application::{ChatCompletions, ListModels};
use crate::infrastructure::http::handlers;

pub fn routes(
    config: Arc<Config>,
    chat_completions: Arc<ChatCompletions>,
    list_models: Arc<ListModels>,
) -> Router {
    Router::new()
        .route("/health", get(handlers::handle_health))
        .route("/v1/models", get(handlers::handle_models))
        .route(
            "/v1/chat/completions",
            post(handlers::handle_chat_completions),
        )
        .layer(axum::Extension(chat_completions))
        .layer(axum::Extension(list_models))
        .with_state(config)
}
