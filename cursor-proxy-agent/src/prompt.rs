use serde_json::Value;

fn content_to_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| {
                if let Value::String(s) = p {
                    return Some(s.as_str());
                }
                if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                    p.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(""),
        _ => String::new(),
    }
}

pub fn build_from_openai(messages: &[Value]) -> String {
    let mut system_parts = Vec::new();
    let mut convo = Vec::new();

    for m in messages {
        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("");
        let text = m.get("content").map(content_to_text).unwrap_or_default();
        if text.is_empty() {
            continue;
        }

        match role {
            "system" | "developer" => system_parts.push(text),
            "user" => convo.push(format!("User: {text}")),
            "assistant" => convo.push(format!("Assistant: {text}")),
            "tool" | "function" => convo.push(format!("Tool: {text}")),
            _ => {}
        }
    }

    let mut result = String::new();
    if !system_parts.is_empty() {
        result.push_str("System:\n");
        result.push_str(&system_parts.join("\n\n"));
        result.push_str("\n\n");
    }
    result.push_str(&convo.join("\n\n"));
    result.push_str("\n\nAssistant:");
    result
}

fn system_to_text(system: &Value) -> String {
    match system {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| {
                if let Value::String(s) = p {
                    return Some(s.as_str());
                }
                if p.get("type").and_then(|t| t.as_str()) == Some("text") {
                    p.get("text").and_then(|t| t.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

pub fn build_from_anthropic(messages: &[Value], system: Option<&Value>) -> String {
    let mut openai_msgs = Vec::new();

    if let Some(sys) = system {
        let text = system_to_text(sys);
        if !text.is_empty() {
            openai_msgs.push(serde_json::json!({
                "role": "system",
                "content": text,
            }));
        }
    }

    for m in messages {
        let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
        let text = m.get("content").map(content_to_text).unwrap_or_default();
        if text.is_empty() {
            continue;
        }
        openai_msgs.push(serde_json::json!({
            "role": role,
            "content": text,
        }));
    }

    build_from_openai(&openai_msgs)
}
