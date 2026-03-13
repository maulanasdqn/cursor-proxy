use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::RwLock;

use cursor_proxy_types::Config;

pub fn mock_config() -> Config {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Config {
        api_key: None,
        agent_bin: format!("{manifest_dir}/fixtures/mock_agent.sh"),
        host: "127.0.0.1".into(),
        port: 0,
        default_model: "auto".into(),
        timeout_ms: 30_000,
        tls_cert: None,
        tls_key: None,
        chat_only_workspace: true,
        verbose: false,
        strict_model: true,
        sessions_log: None,
        workspace: "/tmp".into(),
        approve_mcps: false,
        force: false,
    }
}

pub fn mock_config_with_auth(api_key: &str) -> Config {
    let mut config = mock_config();
    config.api_key = Some(api_key.to_string());
    config
}

pub fn mock_config_with_failing_agent() -> Config {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mut config = mock_config();
    config.agent_bin = format!("{manifest_dir}/fixtures/mock_agent_fail.sh");
    config
}

pub fn build_app(config: Config) -> Router {
    let config = Arc::new(config);
    let last_model: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));
    let model_cache = Arc::new(RwLock::new(None));

    let chat_completions = Arc::new(
        cursor_proxy_openai::application::ChatCompletions::new(
            config.clone(),
            last_model.clone(),
        ),
    );
    let list_models = Arc::new(
        cursor_proxy_openai::application::ListModels::new(config.clone(), model_cache),
    );
    let messages = Arc::new(
        cursor_proxy_anthropic::application::Messages::new(config.clone(), last_model),
    );

    let openai_routes = cursor_proxy_openai::infrastructure::http::routes::routes(
        config.clone(),
        chat_completions,
        list_models,
    );
    let anthropic_routes =
        cursor_proxy_anthropic::infrastructure::http::routes::routes(messages);

    Router::new()
        .merge(openai_routes)
        .merge(anthropic_routes)
        .layer(axum::middleware::from_fn_with_state(
            config.clone(),
            cursor_proxy_auth::auth,
        ))
}

pub async fn spawn_server(config: Config) -> SocketAddr {
    let app = build_app(config);
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    addr
}
