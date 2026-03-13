use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub api_key: Option<String>,
    pub agent_bin: String,
    pub host: String,
    pub port: u16,
    pub default_model: String,
    pub timeout_ms: u64,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub chat_only_workspace: bool,
    pub verbose: bool,
    pub strict_model: bool,
    pub sessions_log: Option<String>,
    pub workspace: String,
    pub approve_mcps: bool,
    pub force: bool,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            api_key: env::var("CURSOR_BRIDGE_API_KEY").ok().filter(|s| !s.is_empty()),
            agent_bin: env::var("CURSOR_AGENT_BIN")
                .or_else(|_| env::var("CURSOR_CLI_BIN"))
                .or_else(|_| env::var("CURSOR_CLI_PATH"))
                .unwrap_or_else(|_| "agent".into()),
            host: env::var("CURSOR_BRIDGE_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("CURSOR_BRIDGE_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8765),
            default_model: env::var("CURSOR_BRIDGE_DEFAULT_MODEL")
                .unwrap_or_else(|_| "auto".into()),
            timeout_ms: env::var("CURSOR_BRIDGE_TIMEOUT_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(300_000),
            tls_cert: env::var("CURSOR_BRIDGE_TLS_CERT").ok().filter(|s| !s.is_empty()),
            tls_key: env::var("CURSOR_BRIDGE_TLS_KEY").ok().filter(|s| !s.is_empty()),
            chat_only_workspace: env::var("CURSOR_BRIDGE_CHAT_ONLY_WORKSPACE")
                .map(|s| s != "false" && s != "0")
                .unwrap_or(true),
            verbose: env::var("CURSOR_BRIDGE_VERBOSE")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
            strict_model: env::var("CURSOR_BRIDGE_STRICT_MODEL")
                .map(|s| s != "false" && s != "0")
                .unwrap_or(true),
            sessions_log: env::var("CURSOR_BRIDGE_SESSIONS_LOG").ok().filter(|s| !s.is_empty()),
            workspace: env::var("CURSOR_BRIDGE_WORKSPACE")
                .unwrap_or_else(|_| env::current_dir().unwrap_or_default().to_string_lossy().into()),
            approve_mcps: env::var("CURSOR_BRIDGE_APPROVE_MCPS")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
            force: env::var("CURSOR_BRIDGE_FORCE")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false),
        }
    }
}
