use cursor_proxy_test::{mock_config, mock_config_with_failing_agent, spawn_server};
use serde_json::json;

#[tokio::test]
async fn sync_chat_completion() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "claude-opus-4-6",
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["object"], "chat.completion");
    assert!(body["id"].as_str().unwrap().starts_with("chatcmpl_"));
    assert_eq!(body["choices"][0]["finish_reason"], "stop");
    assert_eq!(body["choices"][0]["message"]["role"], "assistant");

    let content = body["choices"][0]["message"]["content"].as_str().unwrap();
    assert!(
        content.contains("mock agent"),
        "Expected mock agent response, got: {content}"
    );
}

#[tokio::test]
async fn sync_completion_includes_usage() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "messages": [{"role": "user", "content": "test"}]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(body["usage"]["prompt_tokens"].is_number());
    assert!(body["usage"]["completion_tokens"].is_number());
    assert!(body["usage"]["total_tokens"].is_number());
}

#[tokio::test]
async fn streaming_chat_completion() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "claude-opus-4-6",
            "messages": [{"role": "user", "content": "hello"}],
            "stream": true
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "text/event-stream"
    );

    let text = resp.text().await.unwrap();
    assert!(text.contains("data: "), "Should contain SSE data lines");
    assert!(text.contains("data: [DONE]"), "Should end with [DONE]");
    assert!(
        text.contains("chat.completion.chunk"),
        "Should contain chunk objects"
    );
}

#[tokio::test]
async fn streaming_contains_text_deltas() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let text = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "messages": [{"role": "user", "content": "hello"}],
            "stream": true
        }))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let mut found_content = false;
    let mut found_stop = false;

    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if data == "[DONE]" {
                continue;
            }
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(data) {
                if let Some(content) = obj["choices"][0]["delta"]["content"].as_str() {
                    if !content.is_empty() {
                        found_content = true;
                    }
                }
                if obj["choices"][0]["finish_reason"] == "stop" {
                    found_stop = true;
                }
            }
        }
    }

    assert!(found_content, "Should have content deltas");
    assert!(found_stop, "Should have stop finish_reason");
}

#[tokio::test]
async fn agent_failure_returns_500() {
    let addr = spawn_server(mock_config_with_failing_agent()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 500);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "cursor_cli_error");
}

#[tokio::test]
async fn model_with_provider_prefix() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .post(&url)
        .json(&json!({
            "model": "anthropic/claude-opus-4-6",
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(body["object"], "chat.completion");
}

#[tokio::test]
async fn empty_messages_still_works() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/chat/completions");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "messages": []
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}
