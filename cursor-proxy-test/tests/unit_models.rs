use cursor_proxy_models::{
    get_anthropic_aliases, normalize_model_id, resolve_model, resolve_to_cursor_model,
};
use cursor_proxy_types::Config;

fn default_config() -> Config {
    cursor_proxy_test::mock_config()
}

#[test]
fn maps_claude_opus_4_6() {
    assert_eq!(resolve_to_cursor_model("claude-opus-4-6"), Some("opus-4.6"));
}

#[test]
fn maps_claude_sonnet_4_6() {
    assert_eq!(
        resolve_to_cursor_model("claude-sonnet-4-6"),
        Some("sonnet-4.6")
    );
}

#[test]
fn maps_claude_sonnet_4_5() {
    assert_eq!(
        resolve_to_cursor_model("claude-sonnet-4-5-20250514"),
        Some("sonnet-4.5")
    );
}

#[test]
fn maps_opus_thinking() {
    assert_eq!(
        resolve_to_cursor_model("claude-opus-4-6-thinking"),
        Some("opus-4.6-thinking")
    );
}

#[test]
fn maps_sonnet_thinking() {
    assert_eq!(
        resolve_to_cursor_model("claude-sonnet-4-6-thinking"),
        Some("sonnet-4.6-thinking")
    );
}

#[test]
fn maps_haiku_to_sonnet() {
    assert_eq!(
        resolve_to_cursor_model("claude-haiku-4-5"),
        Some("sonnet-4.5")
    );
    assert_eq!(
        resolve_to_cursor_model("claude-haiku-4-5-20251001"),
        Some("sonnet-4.5")
    );
}

#[test]
fn maps_generic_aliases() {
    assert_eq!(resolve_to_cursor_model("claude-4-opus"), Some("opus-4.6"));
    assert_eq!(
        resolve_to_cursor_model("claude-4-sonnet"),
        Some("sonnet-4.6")
    );
}

#[test]
fn unknown_model_returns_none() {
    assert_eq!(resolve_to_cursor_model("gpt-4o"), None);
    assert_eq!(resolve_to_cursor_model("some-custom-model"), None);
}

#[test]
fn mapping_is_case_insensitive() {
    assert_eq!(
        resolve_to_cursor_model("Claude-Opus-4-6"),
        Some("opus-4.6")
    );
    assert_eq!(
        resolve_to_cursor_model("CLAUDE-SONNET-4-6"),
        Some("sonnet-4.6")
    );
}

#[test]
fn normalize_strips_provider_prefix() {
    assert_eq!(normalize_model_id("anthropic/claude-opus-4-6"), "claude-opus-4-6");
    assert_eq!(normalize_model_id("openai/gpt-4"), "gpt-4");
    assert_eq!(normalize_model_id("a/b/c"), "c");
}

#[test]
fn normalize_trims_whitespace() {
    assert_eq!(normalize_model_id("  claude-opus-4-6  "), "claude-opus-4-6");
}

#[test]
fn normalize_no_prefix_passthrough() {
    assert_eq!(normalize_model_id("claude-opus-4-6"), "claude-opus-4-6");
}

#[test]
fn normalize_empty_string() {
    assert_eq!(normalize_model_id(""), "");
    assert_eq!(normalize_model_id("   "), "");
}

#[test]
fn resolve_explicit_model_wins() {
    let config = default_config();
    let result = resolve_model(Some("claude-opus-4-6"), None, &config);
    assert_eq!(result, "claude-opus-4-6");
}

#[test]
fn resolve_auto_falls_back_to_last_when_strict() {
    let mut config = default_config();
    config.strict_model = true;
    let result = resolve_model(Some("auto"), Some("claude-opus-4-6"), &config);
    assert_eq!(result, "claude-opus-4-6");
}

#[test]
fn resolve_auto_ignores_last_when_not_strict() {
    let mut config = default_config();
    config.strict_model = false;
    let result = resolve_model(Some("auto"), Some("claude-opus-4-6"), &config);
    assert_eq!(result, "auto");
}

#[test]
fn resolve_none_falls_back_to_last_model() {
    let config = default_config();
    let result = resolve_model(None, Some("sonnet-4.6"), &config);
    assert_eq!(result, "sonnet-4.6");
}

#[test]
fn resolve_none_and_no_last_falls_back_to_default() {
    let config = default_config();
    let result = resolve_model(None, None, &config);
    assert_eq!(result, "auto");
}

#[test]
fn anthropic_aliases_filters_by_available() {
    let available = vec!["opus-4.6".to_string(), "sonnet-4.6".to_string()];
    let aliases = get_anthropic_aliases(&available);
    let ids: Vec<&str> = aliases.iter().map(|a| a.anthropic_id).collect();
    assert!(ids.contains(&"claude-opus-4-6"));
    assert!(ids.contains(&"claude-sonnet-4-6"));
    assert!(!ids.contains(&"claude-sonnet-4-5-20250514"));
}

#[test]
fn anthropic_aliases_empty_when_no_match() {
    let available = vec!["gpt-4o".to_string()];
    let aliases = get_anthropic_aliases(&available);
    assert!(aliases.is_empty());
}
