use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::{Extension, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use cursor_proxy_agent::AgentStreamEvent;
use cursor_proxy_errors::AppError;
use cursor_proxy_types::Config;

use crate::application::{ChatCompletions, ExecuteResult, ListModels};
use crate::infrastructure::http::dto::*;

pub async fn handle_chat_completions(
    State(_config): State<Arc<Config>>,
    Extension(use_case): Extension<Arc<ChatCompletions>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<ChatCompletionRequest>,
) -> Result<Response, AppError> {
    let header_ws = headers
        .get("x-cursor-workspace")
        .and_then(|v| v.to_str().ok());

    match use_case.execute(body, header_ws).await? {
        ExecuteResult::Sync(r) => sync_response(r),
        ExecuteResult::Stream {
            id,
            created,
            model,
            event_stream,
            workspace,
        } => stream_response(id, created, model, event_stream, workspace),
    }
}

fn sync_response(r: SyncResponse) -> Result<Response, AppError> {
    Ok(Json(ChatCompletionResponse {
        id: r.id,
        object: "chat.completion",
        created: r.created,
        model: r.model,
        choices: vec![Choice {
            index: 0,
            message: ChoiceMessage {
                role: "assistant",
                content: r.content,
            },
            finish_reason: "stop",
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    })
    .into_response())
}

fn stream_response(
    id: String,
    created: i64,
    model: String,
    event_stream: std::pin::Pin<Box<dyn futures::Stream<Item = AgentStreamEvent> + Send>>,
    workspace: cursor_proxy_agent::WorkspaceResult,
) -> Result<Response, AppError> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(64);

    tokio::spawn(async move {
        let _ws = workspace;
        tokio::pin!(event_stream);

        while let Some(event) = event_stream.next().await {
            let data = match event {
                AgentStreamEvent::Text(text) => {
                    let chunk = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk",
                        created,
                        model: model.clone(),
                        choices: vec![ChunkChoice {
                            index: 0,
                            delta: Delta {
                                content: Some(text),
                            },
                            finish_reason: None,
                        }],
                    };
                    format!(
                        "data: {}\n\n",
                        serde_json::to_string(&chunk).unwrap_or_default()
                    )
                }
                AgentStreamEvent::Done => {
                    let chunk = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk",
                        created,
                        model: model.clone(),
                        choices: vec![ChunkChoice {
                            index: 0,
                            delta: Delta { content: None },
                            finish_reason: Some("stop"),
                        }],
                    };
                    let mut s = format!(
                        "data: {}\n\n",
                        serde_json::to_string(&chunk).unwrap_or_default()
                    );
                    s.push_str("data: [DONE]\n\n");
                    s
                }
            };
            if tx.send(data).await.is_err() {
                break;
            }
        }
    });

    let body = Body::from_stream(ReceiverStream::new(rx).map(Ok::<_, std::convert::Infallible>));

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .body(body)
        .unwrap_or_default())
}

pub async fn handle_models(
    Extension(use_case): Extension<Arc<ListModels>>,
) -> Result<Json<ModelListResponse>, AppError> {
    let entries = use_case.execute().await?;
    Ok(Json(ModelListResponse {
        object: "list",
        data: entries
            .into_iter()
            .map(|e| ModelEntryResponse {
                id: e.id,
                object: "model",
                owned_by: e.owned_by,
                name: e.name,
            })
            .collect(),
    }))
}

pub async fn handle_health(State(config): State<Arc<Config>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ok": true,
        "version": env!("CARGO_PKG_VERSION"),
        "workspace": config.workspace,
        "mode": "ask",
        "defaultModel": config.default_model,
        "force": config.force,
        "approveMcps": config.approve_mcps,
        "strictModel": config.strict_model,
    }))
}
