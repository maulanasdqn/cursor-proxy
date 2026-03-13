mod mapping;
mod resolution;

pub use mapping::{ModelAlias, get_anthropic_aliases, resolve_to_cursor_model};
pub use resolution::{normalize_model_id, resolve_model};
