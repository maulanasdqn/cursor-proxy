use std::sync::Arc;
use tokio::sync::RwLock;

use cursor_proxy_agent::{self as agent, build_from_openai, resolve_workspace};
use cursor_proxy_errors::AppError;
use cursor_proxy_models::{normalize_model_id, resolve_model, resolve_to_cursor_model};
use cursor_proxy_types::Config;

use crate::infrastructure::http::dto::{ChatCompletionRequest, SyncResponse};

pub struct ChatCompletions {
    config: Arc<Config>,
    last_model: Arc<RwLock<Option<String>>>,
}

impl ChatCompletions {
    pub fn new(config: Arc<Config>, last_model: Arc<RwLock<Option<String>>>) -> Self {
        Self { config, last_model }
    }

    pub async fn execute(
        &self,
        body: ChatCompletionRequest,
        header_ws: Option<&str>,
    ) -> Result<ExecuteResult, AppError> {
        let requested = body.model.as_deref().map(normalize_model_id);
        let last = self.last_model.read().await.clone();
        let model = resolve_model(requested, last.as_deref(), &self.config);
        let cursor_model = resolve_to_cursor_model(&model)
            .map(|s| s.to_string())
            .unwrap_or_else(|| model.clone());

        if let Some(explicit) = requested.filter(|r| !r.is_empty() && *r != "auto") {
            *self.last_model.write().await = Some(explicit.to_string());
        }

        let prompt = build_from_openai(&body.messages);
        let workspace = resolve_workspace(&self.config, header_ws)?;
        let id = format!("chatcmpl_{}", uuid::Uuid::new_v4().simple());
        let created = chrono::Utc::now().timestamp();

        if body.stream {
            let workspace_dir = workspace.dir.clone();
            let event_stream =
                agent::run_stream(&self.config, &workspace_dir, &cursor_model, &prompt)?;
            return Ok(ExecuteResult::Stream {
                id,
                created,
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

        Ok(ExecuteResult::Sync(SyncResponse {
            id,
            created,
            model,
            content: out.stdout.trim().to_string(),
        }))
    }
}

pub enum ExecuteResult {
    Sync(SyncResponse),
    Stream {
        id: String,
        created: i64,
        model: String,
        event_stream:
            std::pin::Pin<Box<dyn futures::Stream<Item = agent::AgentStreamEvent> + Send>>,
        workspace: agent::WorkspaceResult,
    },
}
