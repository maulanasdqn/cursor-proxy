use cursor_proxy_agent::{build_from_anthropic, build_from_openai};
use serde_json::json;

#[test]
fn simple_user_message() {
    let messages = vec![json!({"role": "user", "content": "hello"})];
    let result = build_from_openai(&messages);
    assert_eq!(result, "User: hello\n\nAssistant:");
}

#[test]
fn system_then_user() {
    let messages = vec![
        json!({"role": "system", "content": "You are helpful."}),
        json!({"role": "user", "content": "hello"}),
    ];
    let result = build_from_openai(&messages);
    assert_eq!(
        result,
        "System:\nYou are helpful.\n\nUser: hello\n\nAssistant:"
    );
}

#[test]
fn multiple_system_messages_joined() {
    let messages = vec![
        json!({"role": "system", "content": "Rule one."}),
        json!({"role": "system", "content": "Rule two."}),
        json!({"role": "user", "content": "go"}),
    ];
    let result = build_from_openai(&messages);
    assert!(result.starts_with("System:\nRule one.\n\nRule two."));
}

#[test]
fn developer_role_treated_as_system() {
    let messages = vec![
        json!({"role": "developer", "content": "Be concise."}),
        json!({"role": "user", "content": "hi"}),
    ];
    let result = build_from_openai(&messages);
    assert!(result.starts_with("System:\nBe concise."));
}

#[test]
fn multi_turn_conversation() {
    let messages = vec![
        json!({"role": "user", "content": "hello"}),
        json!({"role": "assistant", "content": "hi there"}),
        json!({"role": "user", "content": "how are you?"}),
    ];
    let result = build_from_openai(&messages);
    assert!(result.contains("User: hello"));
    assert!(result.contains("Assistant: hi there"));
    assert!(result.contains("User: how are you?"));
    assert!(result.ends_with("\n\nAssistant:"));
}

#[test]
fn tool_role_handled() {
    let messages = vec![
        json!({"role": "user", "content": "call the tool"}),
        json!({"role": "tool", "content": "tool result here"}),
    ];
    let result = build_from_openai(&messages);
    assert!(result.contains("Tool: tool result here"));
}

#[test]
fn function_role_handled() {
    let messages = vec![json!({"role": "function", "content": "function output"})];
    let result = build_from_openai(&messages);
    assert!(result.contains("Tool: function output"));
}

#[test]
fn content_array_with_text_parts() {
    let messages = vec![json!({
        "role": "user",
        "content": [
            {"type": "text", "text": "first part "},
            {"type": "text", "text": "second part"},
        ]
    })];
    let result = build_from_openai(&messages);
    assert!(result.contains("User: first part second part"));
}

#[test]
fn content_array_ignores_non_text_parts() {
    let messages = vec![json!({
        "role": "user",
        "content": [
            {"type": "image", "url": "http://example.com/img.png"},
            {"type": "text", "text": "describe this"},
        ]
    })];
    let result = build_from_openai(&messages);
    assert!(result.contains("User: describe this"));
}

#[test]
fn empty_content_skipped() {
    let messages = vec![
        json!({"role": "user", "content": ""}),
        json!({"role": "user", "content": "actual message"}),
    ];
    let result = build_from_openai(&messages);
    assert!(!result.contains("User: \n"));
    assert!(result.contains("User: actual message"));
}

#[test]
fn empty_messages_produces_bare_assistant_marker() {
    let result = build_from_openai(&[]);
    assert_eq!(result, "\n\nAssistant:");
}

#[test]
fn anthropic_with_system_string() {
    let messages = vec![json!({"role": "user", "content": "hi"})];
    let system = json!("You are a pirate.");
    let result = build_from_anthropic(&messages, Some(&system));
    assert!(result.contains("System:\nYou are a pirate."));
    assert!(result.contains("User: hi"));
}

#[test]
fn anthropic_with_system_array() {
    let messages = vec![json!({"role": "user", "content": "hi"})];
    let system = json!([
        {"type": "text", "text": "First instruction"},
        {"type": "text", "text": "Second instruction"},
    ]);
    let result = build_from_anthropic(&messages, Some(&system));
    assert!(result.contains("First instruction\nSecond instruction"));
}

#[test]
fn anthropic_without_system() {
    let messages = vec![json!({"role": "user", "content": "hi"})];
    let result = build_from_anthropic(&messages, None);
    assert!(!result.contains("System:"));
    assert!(result.contains("User: hi"));
}

#[test]
fn anthropic_multi_turn() {
    let messages = vec![
        json!({"role": "user", "content": "hello"}),
        json!({"role": "assistant", "content": "greetings"}),
        json!({"role": "user", "content": "explain X"}),
    ];
    let result = build_from_anthropic(&messages, None);
    assert!(result.contains("User: hello"));
    assert!(result.contains("Assistant: greetings"));
    assert!(result.contains("User: explain X"));
}
