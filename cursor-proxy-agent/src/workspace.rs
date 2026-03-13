use cursor_proxy_types::Config;
use tempfile::TempDir;

pub struct WorkspaceResult {
    pub dir: String,
    pub _temp_dir: Option<TempDir>,
}

pub fn resolve_workspace(
    config: &Config,
    header_workspace: Option<&str>,
) -> std::io::Result<WorkspaceResult> {
    if config.chat_only_workspace {
        let tmp = TempDir::with_prefix("cursor-proxy-")?;
        let dir = tmp.path().to_string_lossy().into_owned();
        return Ok(WorkspaceResult {
            dir,
            _temp_dir: Some(tmp),
        });
    }

    let dir = header_workspace
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| config.workspace.clone());

    Ok(WorkspaceResult {
        dir,
        _temp_dir: None,
    })
}
