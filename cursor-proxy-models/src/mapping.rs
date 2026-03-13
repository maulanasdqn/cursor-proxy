pub fn resolve_to_cursor_model(model: &str) -> Option<&'static str> {
    match model.to_lowercase().as_str() {
        "claude-opus-4-6" | "claude-opus-4-6-20250610" => Some("opus-4.6"),
        "claude-sonnet-4-6" | "claude-sonnet-4-6-20250514" => Some("sonnet-4.6"),
        "claude-sonnet-4-5-20250514" => Some("sonnet-4.5"),
        "claude-4-opus" | "claude-4.6-opus" => Some("opus-4.6"),
        "claude-4-sonnet" | "claude-4.6-sonnet" => Some("sonnet-4.6"),
        "claude-opus-4-6-thinking" => Some("opus-4.6-thinking"),
        "claude-sonnet-4-6-thinking" => Some("sonnet-4.6-thinking"),
        "claude-sonnet-4-5-thinking" | "claude-sonnet-4-5-20250514-thinking" => {
            Some("sonnet-4.5-thinking")
        }
        "claude-haiku-4-5" | "claude-haiku-4-5-20251001" => Some("sonnet-4.5"),
        _ => None,
    }
}

pub struct ModelAlias {
    pub cursor_id: &'static str,
    pub anthropic_id: &'static str,
    pub name: &'static str,
}

const ANTHROPIC_ALIASES: &[ModelAlias] = &[
    ModelAlias {
        cursor_id: "opus-4.6",
        anthropic_id: "claude-opus-4-6",
        name: "Claude 4.6 Opus",
    },
    ModelAlias {
        cursor_id: "sonnet-4.6",
        anthropic_id: "claude-sonnet-4-6",
        name: "Claude 4.6 Sonnet",
    },
    ModelAlias {
        cursor_id: "sonnet-4.5",
        anthropic_id: "claude-sonnet-4-5-20250514",
        name: "Claude 4.5 Sonnet",
    },
    ModelAlias {
        cursor_id: "opus-4.6-thinking",
        anthropic_id: "claude-opus-4-6-thinking",
        name: "Claude 4.6 Opus (Thinking)",
    },
    ModelAlias {
        cursor_id: "sonnet-4.6-thinking",
        anthropic_id: "claude-sonnet-4-6-thinking",
        name: "Claude 4.6 Sonnet (Thinking)",
    },
    ModelAlias {
        cursor_id: "sonnet-4.5-thinking",
        anthropic_id: "claude-sonnet-4-5-thinking",
        name: "Claude 4.5 Sonnet (Thinking)",
    },
];

pub fn get_anthropic_aliases(available_ids: &[String]) -> Vec<&'static ModelAlias> {
    ANTHROPIC_ALIASES
        .iter()
        .filter(|a| available_ids.iter().any(|id| id == a.cursor_id))
        .collect()
}
