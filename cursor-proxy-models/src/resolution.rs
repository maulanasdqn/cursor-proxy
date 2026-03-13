use cursor_proxy_types::Config;

pub fn normalize_model_id(raw: &str) -> &str {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return trimmed;
    }
    match trimmed.rsplit_once('/') {
        Some((_, after)) => after,
        None => trimmed,
    }
}

pub fn resolve_model(
    requested: Option<&str>,
    last_model: Option<&str>,
    config: &Config,
) -> String {
    let explicit = requested.filter(|r| !r.is_empty() && *r != "auto");

    if let Some(model) = explicit {
        return model.to_string();
    }

    if config.strict_model {
        if let Some(last) = last_model {
            return last.to_string();
        }
    }

    if let Some(r) = requested {
        return r.to_string();
    }

    if let Some(last) = last_model {
        return last.to_string();
    }

    config.default_model.clone()
}
