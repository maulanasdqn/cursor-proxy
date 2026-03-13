use std::sync::Arc;

use axum::Json;
use axum::body::Body;
use axum::extract::Extension;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use futures::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use cursor_proxy_agent::AgentStreamEvent;
use cursor_proxy_errors::AppError;

use crate::application::{ExecuteResult, Messages};
use crate::infrastructure::http::dto::*;

pub async fn handle_messages(
    Extension(use_case): Extension<Arc<Messages>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<AnthropicRequest>,
) -> Result<Response, AppError> {
    let header_ws = headers
        .get("x-cursor-workspace")
        .and_then(|v| v.to_str().ok());

    match use_case.execute(body, header_ws).await? {
        ExecuteResult::Sync { id, model, content } => sync_response(id, model, content),
        ExecuteResult::Stream {
            id,
            model,
            event_stream,
            workspace,
        } => stream_response(id, model, event_stream, workspace),
    }
}

fn sync_response(id: String, model: String, content: String) -> Result<Response, AppError> {
    Ok(Json(MessagesResponse {
        id,
        type_field: "message",
        role: "assistant",
        content: vec![ContentBlock {
            type_field: "text",
            text: content,
        }],
        model,
        stop_reason: "end_turn",
        usage: AnthropicUsage {
            input_tokens: 0,
            output_tokens: 0,
        },
    })
    .into_response())
}

fn stream_response(
    id: String,
    model: String,
    event_stream: std::pin::Pin<Box<dyn futures::Stream<Item = AgentStreamEvent> + Send>>,
    workspace: cursor_proxy_agent::WorkspaceResult,
) -> Result<Response, AppError> {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(64);

    tokio::spawn(async move {
        let _ws = workspace;

        let start = MessageStartEvent {
            type_field: "message_start",
            message: MessageStartPayload {
                id: id.clone(),
                type_field: "message",
                role: "assistant",
                content: vec![],
                model: model.clone(),
                usage: AnthropicUsage {
                    input_tokens: 0,
                    output_tokens: 0,
                },
            },
        };
        let _ = tx
            .send(format!(
                "event: message_start\ndata: {}\n\n",
                serde_json::to_string(&start).unwrap_or_default()
            ))
            .await;

        let block_start = ContentBlockStart {
            type_field: "content_block_start",
            index: 0,
            content_block: ContentBlock {
                type_field: "text",
                text: String::new(),
            },
        };
        let _ = tx
            .send(format!(
                "event: content_block_start\ndata: {}\n\n",
                serde_json::to_string(&block_start).unwrap_or_default()
            ))
            .await;

        tokio::pin!(event_stream);

        while let Some(event) = event_stream.next().await {
            let msg = match event {
                AgentStreamEvent::Text(text) => {
                    let delta = ContentBlockDelta {
                        type_field: "content_block_delta",
                        index: 0,
                        delta: TextDelta {
                            type_field: "text_delta",
                            text,
                        },
                    };
                    format!(
                        "event: content_block_delta\ndata: {}\n\n",
                        serde_json::to_string(&delta).unwrap_or_default()
                    )
                }
                AgentStreamEvent::Done => {
                    let mut s = format!(
                        "event: content_block_stop\ndata: {}\n\n",
                        serde_json::to_string(
                            &serde_json::json!({"type": "content_block_stop", "index": 0})
                        )
                        .unwrap_or_default()
                    );
                    let msg_delta = MessageDelta {
                        type_field: "message_delta",
                        delta: MessageDeltaPayload {
                            stop_reason: "end_turn",
                        },
                        usage: AnthropicUsage {
                            input_tokens: 0,
                            output_tokens: 0,
                        },
                    };
                    s.push_str(&format!(
                        "event: message_delta\ndata: {}\n\n",
                        serde_json::to_string(&msg_delta).unwrap_or_default()
                    ));
                    s.push_str(&format!(
                        "event: message_stop\ndata: {}\n\n",
                        serde_json::to_string(&serde_json::json!({"type": "message_stop"}))
                            .unwrap_or_default()
                    ));
                    s
                }
            };
            if tx.send(msg).await.is_err() {
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
