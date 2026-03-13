use cursor_proxy_test::{mock_config, mock_config_with_failing_agent, spawn_server};
use serde_json::json;

#[tokio::test]
async fn sync_messages() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "claude-opus-4-6",
            "max_tokens": 4096,
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["type"], "message");
    assert_eq!(body["role"], "assistant");
    assert!(body["id"].as_str().unwrap().starts_with("msg_"));
    assert_eq!(body["stop_reason"], "end_turn");

    let text = body["content"][0]["text"].as_str().unwrap();
    assert!(
        text.contains("mock agent"),
        "Expected mock response, got: {text}"
    );
}

#[tokio::test]
async fn sync_messages_includes_usage() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let body: serde_json::Value = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "test"}]
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(body["usage"]["input_tokens"].is_number());
    assert!(body["usage"]["output_tokens"].is_number());
}

#[tokio::test]
async fn missing_max_tokens_returns_400() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

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

    assert_eq!(resp.status(), 400);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["error"]["message"]
        .as_str()
        .unwrap()
        .contains("max_tokens"));
}

#[tokio::test]
async fn streaming_messages() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "claude-opus-4-6",
            "max_tokens": 4096,
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
    assert!(
        text.contains("event: message_start"),
        "Should contain message_start event"
    );
    assert!(
        text.contains("event: content_block_start"),
        "Should contain content_block_start event"
    );
    assert!(
        text.contains("event: content_block_delta"),
        "Should contain content_block_delta event"
    );
    assert!(
        text.contains("event: content_block_stop"),
        "Should contain content_block_stop event"
    );
    assert!(
        text.contains("event: message_delta"),
        "Should contain message_delta event"
    );
    assert!(
        text.contains("event: message_stop"),
        "Should contain message_stop event"
    );
}

#[tokio::test]
async fn streaming_event_sequence_is_correct() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let text = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "hello"}],
            "stream": true
        }))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let events: Vec<&str> = text
        .lines()
        .filter_map(|l| l.strip_prefix("event: "))
        .collect();

    assert!(!events.is_empty());
    assert_eq!(events[0], "message_start", "First event must be message_start");
    assert_eq!(events[1], "content_block_start", "Second event must be content_block_start");

    let last_three = &events[events.len() - 3..];
    assert_eq!(last_three[0], "content_block_stop");
    assert_eq!(last_three[1], "message_delta");
    assert_eq!(last_three[2], "message_stop");
}

#[tokio::test]
async fn streaming_deltas_contain_text() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let text = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "hello"}],
            "stream": true
        }))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let mut found_text_delta = false;
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data: ") {
            if let Ok(obj) = serde_json::from_str::<serde_json::Value>(data) {
                if obj["type"] == "content_block_delta" {
                    let delta_text = obj["delta"]["text"].as_str().unwrap_or("");
                    if !delta_text.is_empty() {
                        found_text_delta = true;
                    }
                }
            }
        }
    }

    assert!(found_text_delta, "Should have at least one non-empty text delta");
}

#[tokio::test]
async fn with_system_prompt() {
    let addr = spawn_server(mock_config()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "max_tokens": 1024,
            "system": "You are a helpful assistant.",
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn agent_failure_returns_500() {
    let addr = spawn_server(mock_config_with_failing_agent()).await;
    let url = format!("http://{addr}/v1/messages");

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&json!({
            "model": "auto",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": "hello"}]
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 500);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "cursor_cli_error");
}
