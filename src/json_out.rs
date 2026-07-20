use crate::*;

#[cfg(any(feature = "wasm", test))]
pub(crate) fn parsed_summary_json(parsed: &Parsed) -> String {
    let mut json = String::new();
    json.push_str("{\"ok\":");
    json.push_str(if parsed.best.is_some() {
        "true"
    } else {
        "false"
    });
    json.push_str(",\"input\":");
    push_json_string(&mut json, &parsed.input);
    json.push_str(",\"best\":");
    if let Some(best) = &parsed.best {
        push_reading_json(&mut json, best);
    } else {
        json.push_str("null");
    }
    json.push_str(",\"issues\":[");
    for (idx, issue) in ranked_findings(parsed).iter().enumerate() {
        if idx > 0 {
            json.push(',');
        }
        json.push_str("{\"code\":");
        push_json_string(&mut json, issue.code.as_str());
        json.push_str(",\"severity\":");
        push_json_string(&mut json, issue.severity.as_str());
        json.push_str(",\"rank\":");
        json.push_str(&issue.rank.to_string());
        json.push_str(",\"ref_text\":");
        push_json_string(&mut json, &issue.ref_text);
        json.push('}');
    }
    json.push_str("]}");
    json
}

#[cfg(feature = "wasm")]
pub(crate) fn parsed_matches_summary_json(source: &str, matches: &[ParsedMatch]) -> String {
    let mut json = String::new();
    json.push('[');
    for (idx, parsed_match) in matches.iter().enumerate() {
        if idx > 0 {
            json.push(',');
        }
        json.push_str("{\"start\":");
        json.push_str(&parsed_match.start.to_string());
        json.push_str(",\"end\":");
        json.push_str(&parsed_match.end.to_string());
        json.push_str(",\"byteStart\":");
        json.push_str(&parsed_match.start.to_string());
        json.push_str(",\"byteEnd\":");
        json.push_str(&parsed_match.end.to_string());
        let char_start = byte_to_char_offset(source, parsed_match.start);
        let char_end = byte_to_char_offset(source, parsed_match.end);
        json.push_str(",\"charStart\":");
        json.push_str(&char_start.to_string());
        json.push_str(",\"charEnd\":");
        json.push_str(&char_end.to_string());
        json.push_str(",\"text\":");
        push_json_string(&mut json, &parsed_match.text);
        json.push_str(",\"parsed\":");
        json.push_str(&parsed_summary_json(&parsed_match.parsed));
        json.push('}');
    }
    json.push(']');
    json
}

#[cfg(feature = "wasm")]
pub(crate) fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset].chars().count()
}

#[cfg(any(feature = "wasm", test))]
pub(crate) fn push_reading_json(json: &mut String, reading: &Reading) {
    json.push_str("{\"kind\":");
    push_json_string(json, kind_str(reading.kind));
    if let Some(custom_kind) = &reading.custom_kind {
        json.push_str(",\"customKind\":");
        push_json_string(json, custom_kind);
    }
    if let Some(value) = reading.value {
        json.push_str(",\"value\":");
        json.push_str(&format_number(value));
    }
    if let Some(unit) = &reading.unit {
        json.push_str(",\"unit\":");
        push_json_string(json, unit);
    }
    if let Some(dimension) = reading.dimension {
        json.push_str(",\"dimension\":");
        push_json_string(json, dimension.as_str());
    }
    if let Some(date) = &reading.date {
        json.push_str(",\"date\":");
        push_json_string(json, date);
    }
    if let Some(recurrence) = &reading.recurrence {
        json.push_str(",\"recurrence\":");
        push_json_string(json, recurrence);
    }
    if let Some(timezone) = &reading.timezone {
        json.push_str(",\"timezone\":");
        push_json_string(json, timezone);
    }
    json.push('}');
}

pub(crate) fn kind_str(kind: Kind) -> &'static str {
    match kind {
        Kind::Quantity => "quantity",
        Kind::Date => "date",
        Kind::Range => "range",
        Kind::Number => "number",
        Kind::Recurrence => "recurrence",
    }
}

#[cfg(any(feature = "wasm", test))]
pub(crate) fn push_json_string(json: &mut String, value: &str) {
    json.push('"');
    for ch in value.chars() {
        match ch {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            ch if ch.is_control() => json.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => json.push(ch),
        }
    }
    json.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_parsed_summary_json_for_adapters() {
        let parsed = parse("5尺3寸", None);
        let json = parsed_summary_json(&parsed);
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"kind\":\"quantity\""));
        assert!(json.contains("\"unit\":\"m\""));
        assert!(json.contains("\"dimension\":\"length\""));

        let failed = parsed_summary_json(&parse("3pm Europe/Paris", None));
        assert!(failed.contains("\"ok\":false"));
        assert!(failed.contains("\"code\":\"TIMEZONE_UNSUPPORTED\""));
        assert!(failed.contains("\"severity\":\"error\""));
    }
}
