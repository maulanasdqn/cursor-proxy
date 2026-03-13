use std::sync::Arc;
use std::time::Instant;

use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::debug;

use cursor_proxy_errors::AppError;
use cursor_proxy_models::get_anthropic_aliases;
use cursor_proxy_types::{Config, CursorModel};

use crate::infrastructure::http::dto::ModelEntry;

pub struct ModelCache {
    pub at: Instant,
    pub models: Vec<CursorModel>,
}

pub struct ListModels {
    config: Arc<Config>,
    cache: Arc<RwLock<Option<ModelCache>>>,
}

impl ListModels {
    pub fn new(config: Arc<Config>, cache: Arc<RwLock<Option<ModelCache>>>) -> Self {
        Self { config, cache }
    }

    pub async fn execute(&self) -> Result<Vec<ModelEntry>, AppError> {
        {
            let cache = self.cache.read().await;
            if let Some(c) = cache.as_ref() {
                if c.at.elapsed() < std::time::Duration::from_secs(300) {
                    return Ok(build_entries(&c.models));
                }
            }
        }

        debug!(bin = %self.config.agent_bin, "fetching model list");

        let output = Command::new(&self.config.agent_bin)
            .args(["--list-models"])
            .output()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to run agent --list-models: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::AgentError {
                code: output.status.code().unwrap_or(-1),
                stderr: stderr.into_owned(),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let models = parse_model_list(&stdout);

        *self.cache.write().await = Some(ModelCache {
            at: Instant::now(),
            models: models.clone(),
        });

        Ok(build_entries(&models))
    }
}

fn parse_model_list(output: &str) -> Vec<CursorModel> {
    let mut seen = std::collections::HashSet::new();
    output
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let (id, rest) = line.split_once(" - ")?;
            let id = id.trim().to_string();
            if id.is_empty() || !seen.insert(id.clone()) {
                return None;
            }
            let name = rest
                .trim()
                .trim_end_matches(|c: char| c == ')')
                .rsplit_once('(')
                .map(|(before, _)| before.trim())
                .unwrap_or(rest.trim())
                .to_string();
            let name = if name.is_empty() { id.clone() } else { name };
            Some(CursorModel { id, name })
        })
        .collect()
}

fn build_entries(models: &[CursorModel]) -> Vec<ModelEntry> {
    let mut data: Vec<ModelEntry> = models
        .iter()
        .map(|m| ModelEntry {
            id: m.id.clone(),
            owned_by: "cursor".into(),
            name: m.name.clone(),
        })
        .collect();

    let ids: Vec<String> = models.iter().map(|m| m.id.clone()).collect();
    for alias in get_anthropic_aliases(&ids) {
        data.push(ModelEntry {
            id: alias.anthropic_id.to_string(),
            owned_by: "cursor".into(),
            name: alias.name.to_string(),
        });
    }

    data
}
