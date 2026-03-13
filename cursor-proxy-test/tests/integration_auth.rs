use cursor_proxy_test::{mock_config, mock_config_with_auth, spawn_server};

#[tokio::test]
async fn no_auth_configured_allows_all_requests() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/health");

    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn missing_token_returns_401() {
    let addr = spawn_server(mock_config_with_auth("test-key")).await;
    let url = format!("http://{addr}/health");

    let resp = reqwest::get(&url).await.unwrap();
    assert_eq!(resp.status(), 401);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["error"]["message"].as_str().unwrap().contains("API key"));
}

#[tokio::test]
async fn wrong_token_returns_401() {
    let addr = spawn_server(mock_config_with_auth("correct-key")).await;
    let url = format!("http://{addr}/health");

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", "Bearer wrong-key")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[tokio::test]
async fn correct_token_allows_request() {
    let addr = spawn_server(mock_config_with_auth("my-secret")).await;
    let url = format!("http://{addr}/health");

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", "Bearer my-secret")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn lowercase_bearer_prefix_works() {
    let addr = spawn_server(mock_config_with_auth("my-secret")).await;
    let url = format!("http://{addr}/health");

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("Authorization", "bearer my-secret")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn auth_applies_to_all_routes() {
    let addr = spawn_server(mock_config_with_auth("secret")).await;
    let client = reqwest::Client::new();

    let endpoints = vec![
        format!("http://{addr}/health"),
        format!("http://{addr}/v1/models"),
    ];

    for url in endpoints {
        let resp = client.get(&url).send().await.unwrap();
        assert_eq!(resp.status(), 401, "Expected 401 for {url}");
    }
}
