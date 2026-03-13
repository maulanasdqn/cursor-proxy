use std::process::Stdio;

use futures::Stream;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::{debug, error};

use cursor_proxy_errors::AppError;
use cursor_proxy_types::Config;

pub struct AgentOutput {
    pub stdout: String,
    pub stderr: String,
    pub code: i32,
}

#[derive(Debug)]
pub enum AgentStreamEvent {
    Text(String),
    Done,
}

fn build_cmd_args(
    config: &Config,
    workspace_dir: &str,
    model: &str,
    prompt: &str,
    stream: bool,
) -> Vec<String> {
    let mut args = vec!["--print".to_string()];
    if config.approve_mcps {
        args.push("--approve-mcps".to_string());
    }
    if config.force {
        args.push("--force".to_string());
    }
    if config.chat_only_workspace {
        args.push("--trust".to_string());
    }
    args.extend(["--mode".into(), "ask".into()]);
    args.extend(["--workspace".into(), workspace_dir.to_string()]);
    args.extend(["--model".into(), model.to_string()]);
    if stream {
        args.extend([
            "--stream-partial-output".into(),
            "--output-format".into(),
            "stream-json".into(),
        ]);
    } else {
        args.extend(["--output-format".into(), "text".into()]);
    }
    args.push(prompt.to_string());
    args
}

pub async fn run_sync(
    config: &Config,
    workspace_dir: &str,
    model: &str,
    prompt: &str,
) -> Result<AgentOutput, AppError> {
    let args = build_cmd_args(config, workspace_dir, model, prompt, false);
    debug!(bin = %config.agent_bin, ?args, "spawning agent (sync)");

    let output = Command::new(&config.agent_bin)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to spawn agent: {e}")))?;

    Ok(AgentOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        code: output.status.code().unwrap_or(-1),
    })
}

pub fn run_stream(
    config: &Config,
    workspace_dir: &str,
    model: &str,
    prompt: &str,
) -> Result<impl Stream<Item = AgentStreamEvent> + use<>, AppError> {
    let args = build_cmd_args(config, workspace_dir, model, prompt, true);
    debug!(bin = %config.agent_bin, ?args, "spawning agent (stream)");

    let mut child = Command::new(&config.agent_bin)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| AppError::Internal(format!("Failed to spawn agent: {e}")))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Internal("No stdout from agent".into()))?;

    let stream = async_stream::stream! {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            match parse_stream_line(&line) {
                Some(AgentStreamEvent::Text(t)) => yield AgentStreamEvent::Text(t),
                Some(AgentStreamEvent::Done) => {
                    yield AgentStreamEvent::Done;
                    break;
                }
                None => {}
            }
        }

        match child.wait().await {
            Ok(status) => {
                if !status.success() {
                    error!(code = status.code(), "agent exited with error");
                }
            }
            Err(e) => error!(%e, "failed to wait on agent"),
        }
    };

    Ok(stream)
}

fn parse_stream_line(line: &str) -> Option<AgentStreamEvent> {
    let obj: Value = serde_json::from_str(line).ok()?;

    if obj.get("type").and_then(|t| t.as_str()) == Some("assistant")
        && let Some(content) = obj
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_array())
    {
        let text: String = content
            .iter()
            .filter(|p| p.get("type").and_then(|t| t.as_str()) == Some("text"))
            .filter_map(|p| p.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("");
        if !text.is_empty() {
            return Some(AgentStreamEvent::Text(text));
        }
    }

    if obj.get("type").and_then(|t| t.as_str()) == Some("result")
        && obj.get("subtype").and_then(|t| t.as_str()) == Some("success")
    {
        return Some(AgentStreamEvent::Done);
    }

    None
}
