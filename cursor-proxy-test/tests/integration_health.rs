use cursor_proxy_test::{mock_config, spawn_server};

#[tokio::test]
async fn health_returns_ok() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/health");

    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["ok"], true);
    assert_eq!(body["mode"], "ask");
    assert_eq!(body["defaultModel"], "auto");
}

#[tokio::test]
async fn health_includes_version() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/health");

    let body: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    assert!(body["version"].is_string());
    assert!(!body["version"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn health_reflects_config() {
    let mut config = mock_config();
    config.default_model = "opus-4.6".into();
    config.force = true;
    config.strict_model = false;

    let addr = spawn_server(config).await;
    let url = format!("http://{addr}/health");

    let body: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    assert_eq!(body["defaultModel"], "opus-4.6");
    assert_eq!(body["force"], true);
    assert_eq!(body["strictModel"], false);
}
