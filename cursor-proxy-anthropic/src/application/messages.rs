use std::sync::Arc;

use tokio::sync::RwLock;

use cursor_proxy_agent::{self as agent, build_from_anthropic, resolve_workspace};
use cursor_proxy_errors::AppError;
use cursor_proxy_models::{normalize_model_id, resolve_model, resolve_to_cursor_model};
use cursor_proxy_types::Config;

use crate::infrastructure::http::dto::AnthropicRequest;

pub struct Messages {
    config: Arc<Config>,
    last_model: Arc<RwLock<Option<String>>>,
}

impl Messages {
    pub fn new(config: Arc<Config>, last_model: Arc<RwLock<Option<String>>>) -> Self {
        Self { config, last_model }
    }

    pub async fn execute(
        &self,
        body: AnthropicRequest,
        header_ws: Option<&str>,
    ) -> Result<ExecuteResult, AppError> {
        if body.max_tokens.is_none() {
            return Err(AppError::BadRequest("max_tokens is required".into()));
        }

        let requested = body.model.as_deref().map(normalize_model_id);
        let last = self.last_model.read().await.clone();
        let model = resolve_model(requested, last.as_deref(), &self.config);
        let cursor_model = resolve_to_cursor_model(&model)
            .map(|s| s.to_string())
            .unwrap_or_else(|| model.clone());

        if let Some(explicit) = requested.filter(|r| !r.is_empty() && *r != "auto") {
            *self.last_model.write().await = Some(explicit.to_string());
        }

        let prompt = build_from_anthropic(&body.messages, body.system.as_ref());
        let workspace = resolve_workspace(&self.config, header_ws)?;
        let id = format!("msg_{}", uuid::Uuid::new_v4().simple());

        if body.stream {
            let workspace_dir = workspace.dir.clone();
            let event_stream =
                agent::run_stream(&self.config, &workspace_dir, &cursor_model, &prompt)?;
            return Ok(ExecuteResult::Stream {
                id,
                model,
                event_stream: Box::pin(event_stream),
                workspace,
            });
        }

        let out = agent::run_sync(&self.config, &workspace.dir, &cursor_model, &prompt).await?;
        if out.code != 0 {
            return Err(AppError::AgentError {
                code: out.code,
                stderr: out.stderr,
            });
        }

        Ok(ExecuteResult::Sync {
            id,
            model,
            content: out.stdout.trim().to_string(),
        })
    }
}

pub enum ExecuteResult {
    Sync {
        id: String,
        model: String,
        content: String,
    },
    Stream {
        id: String,
        model: String,
        event_stream:
            std::pin::Pin<Box<dyn futures::Stream<Item = agent::AgentStreamEvent> + Send>>,
        workspace: agent::WorkspaceResult,
    },
}
