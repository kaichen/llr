use serde_json::{json, Value};

/// Convert Anthropic `/v1/messages` request body → OpenAI `/v1/chat/completions`.
/// Returns (new_path, new_body) on success, or `None` if the path isn't Anthropic.
pub fn anthropic_to_openai(path: &str, body: &[u8]) -> Option<(String, Vec<u8>)> {
    if !path.starts_with("/v1/messages") {
        return None;
    }

    let v: Value = serde_json::from_slice(body).ok()?;

    // Collect messages -------------------------------------------------------
    let mut messages = Vec::<Value>::new();

    if let Some(system) = v.get("system").and_then(|s| s.as_str()) {
        messages.push(json!({ "role": "system", "content": system }));
    }

    if let Some(arr) = v.get("messages").and_then(|m| m.as_array()) {
        for m in arr {
            let role_src = m.get("role")?.as_str()?;
            let openai_role = match role_src {
                "assistant" => "assistant",
                "user" | "human" => "user",
                _ => "user",
            };

            // Anthropic allows an array for `content`; flatten to plain string.
            let content = match m.get("content")? {
                Value::String(s) => s.clone(),
                Value::Array(a) => a
                    .iter()
                    .filter_map(|seg| seg.get("text").and_then(|t| t.as_str()))
                    .collect::<Vec<_>>()
                    .join(""),
                _ => "".into(),
            };

            messages.push(json!({ "role": openai_role, "content": content }));
        }
    }

    // Build OpenAI‑style body -----------------------------------------------
    let mut out = json!({
        "model": v.get("model").cloned().unwrap_or(json!("gpt-4o-mini")),
        "messages": messages,
    });

    if let Some(x) = v.get("max_tokens") {
        out["max_completion_tokens"] = x.clone();
    }
    if let Some(x) = v.get("temperature") {
        out["temperature"] = x.clone();
    }
    if let Some(x) = v.get("top_p") {
        out["top_p"] = x.clone();
    }
    if let Some(x) = v.get("stream") {
        out["stream"] = x.clone();
    }
    if let Some(x) = v.get("stop_sequences") {
        out["stop"] = x.clone();
    }

    let bytes = serde_json::to_vec(&out).ok()?;
    Some(("/v1/chat/completions".to_string(), bytes))
}
