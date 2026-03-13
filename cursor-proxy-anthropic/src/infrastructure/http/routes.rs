use std::sync::Arc;

use axum::Router;
use axum::routing::post;

use crate::application::Messages;
use crate::infrastructure::http::handlers;

pub fn routes(messages: Arc<Messages>) -> Router {
    Router::new()
        .route("/v1/messages", post(handlers::handle_messages))
        .layer(axum::Extension(messages))
}
