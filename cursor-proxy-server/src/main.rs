use std::sync::Arc;

use axum::Router;
use tokio::sync::RwLock;
use tracing::info;
use tracing_subscriber::EnvFilter;

use cursor_proxy_types::Config;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let filter = if config.verbose {
        EnvFilter::new("cursor_proxy=debug,tower_http=debug")
    } else {
        EnvFilter::new("cursor_proxy=info")
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

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

    let app = Router::new()
        .merge(openai_routes)
        .merge(anthropic_routes)
        .layer(axum::middleware::from_fn_with_state(
            config.clone(),
            cursor_proxy_auth::auth,
        ));

    let addr = format!("{}:{}", config.host, config.port);

    match (&config.tls_cert, &config.tls_key) {
        (Some(cert), Some(key)) => {
            info!("Starting HTTPS server on {addr}");
            let tls_config = axum_server::tls_rustls::RustlsConfig::from_pem_file(cert, key)
                .await
                .expect("Failed to load TLS config");
            axum_server::bind_rustls(addr.parse().expect("Invalid bind address"), tls_config)
                .serve(app.into_make_service())
                .await
                .expect("Server error");
        }
        _ => {
            info!("Starting HTTP server on {addr}");
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .expect("Failed to bind");
            axum::serve(listener, app).await.expect("Server error");
        }
    }
}
