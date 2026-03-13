use cursor_proxy_test::{mock_config, spawn_server};

#[tokio::test]
async fn list_models_returns_cursor_and_aliases() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/models");

    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["object"], "list");

    let data = body["data"].as_array().unwrap();
    assert!(!data.is_empty());

    let ids: Vec<&str> = data.iter().filter_map(|m| m["id"].as_str()).collect();

    assert!(
        ids.contains(&"opus-4.6"),
        "Should contain cursor model opus-4.6"
    );
    assert!(
        ids.contains(&"sonnet-4.6"),
        "Should contain cursor model sonnet-4.6"
    );
    assert!(
        ids.contains(&"claude-opus-4-6"),
        "Should contain anthropic alias claude-opus-4-6"
    );
}

#[tokio::test]
async fn model_entries_have_correct_shape() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/models");

    let body: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    let first = &body["data"][0];

    assert!(first["id"].is_string());
    assert_eq!(first["object"], "model");
    assert!(first["owned_by"].is_string());
    assert!(first["name"].is_string());
}

#[tokio::test]
async fn models_are_cached() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/models");

    let body1: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();
    let body2: serde_json::Value = reqwest::get(&url).await.unwrap().json().await.unwrap();

    assert_eq!(body1, body2);
}

#[tokio::test]
async fn models_with_failing_agent_returns_error() {
    let config = cursor_proxy_test::mock_config_with_failing_agent();
    let addr = spawn_server(config).await;
    let url = format!("http://{addr}/v1/models");

    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 500);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["error"]["message"].is_string());
}
