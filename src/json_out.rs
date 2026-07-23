use crate::*;

#[cfg(any(feature = "wasm", test))]
pub(crate) fn parsed_summary_json(parsed: &Parsed) -> String {
    let mut json = String::new();
    json.push_str("{\"ok\":");
    // The one acceptance rule, not a fourth copy of it.
    json.push_str(if accepts(parsed) { "true" } else { "false" });
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

/// Converts a byte offset in `text` to the char offset a JS caller can use.
///
/// `charStart`/`charEnd` have to describe the same fragment as `byteStart`/
/// `byteEnd`, so both are measured against the original source string the
/// caller passed in. An offset that is not on a char boundary would slice
/// through a multi-byte character and panic, so it is rounded down to the
/// boundary that contains it rather than taken on faith.
#[cfg(feature = "wasm")]
pub(crate) fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
    text[..floor_char_boundary(text, byte_offset)]
        .chars()
        .count()
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
        // JSON has no literal for infinity or NaN, and the output schema already
        // declares `value` as ["number", "null"], so a non-finite value is
        // emitted as null rather than as a token that breaks `JSON.parse`.
        if value.is_finite() {
            push_json_number(json, value);
        } else {
            json.push_str("null");
        }
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
    if let Some(timezone) = &reading.timezone {
        json.push_str(",\"timezone\":");
        push_json_string(json, timezone);
    }
    // A range reading carries its payload entirely in the endpoints: without
    // them the envelope is a bare `{"kind":"range"}`, which reports `ok:true`
    // with no finding while both bounds are gone. The endpoints are emitted as
    // nested readings through this same function so an endpoint carries exactly
    // the fields a top-level reading does.
    if let Some(range) = &reading.range {
        json.push_str(",\"range\":{\"from\":");
        push_reading_json(json, &range.from);
        json.push_str(",\"to\":");
        push_reading_json(json, &range.to);
        json.push('}');
    }
    json.push('}');
}

/// Writes a finite `f64` as a JSON number without display rounding.
///
/// The envelope is a machine transport, so it shares `format_number_exact` with
/// the field-list view rather than the six-decimal `format_number`, which
/// silently collapses `0.0000001 m` to `0` and truncates `2 cups` from
/// `0.473176473` to `0.473176`. The output is always a plain JSON number for
/// every finite value, including subnormals and magnitudes near `f64::MAX`.
#[cfg(any(feature = "wasm", test))]
pub(crate) fn push_json_number(json: &mut String, value: f64) {
    debug_assert!(value.is_finite());
    json.push_str(&crate::adapters::format_number_exact(value));
}

pub(crate) fn kind_str(kind: Kind) -> &'static str {
    match kind {
        Kind::Quantity => "quantity",
        Kind::Date => "date",
        Kind::Range => "range",
        Kind::Number => "number",
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

    #[test]
    fn emits_valid_json_for_non_finite_readings() {
        // `JSON.parse` on the browser side must not see a bare inf/-inf/NaN
        // token, so a non-finite value is emitted as null (the output schema
        // already declares "value": ["number", "null"]).
        for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            let mut json = String::new();
            push_reading_json(&mut json, &Reading::number(value, 0.9));
            assert_eq!(json, "{\"kind\":\"number\",\"value\":null}", "{value}");
            assert!(!json.contains("inf"), "{value}");
            assert!(!json.contains("NaN"), "{value}");
            assert!(!json.contains("nan"), "{value}");
            assert!(is_valid_json(&json), "{json}");
        }

        let mut json = String::new();
        push_reading_json(
            &mut json,
            &Reading::quantity(
                f64::INFINITY,
                "kg",
                Dimension::Mass,
                Provenance::SiMultiple,
                false,
                0.9,
            ),
        );
        assert!(json.contains("\"value\":null"), "{json}");
        assert!(is_valid_json(&json), "{json}");
    }

    #[test]
    fn emits_valid_json_for_overflowing_input() {
        let overflowing = parsed_summary_json(&parse(&"9".repeat(400), None));
        assert!(overflowing.contains("\"ok\":false"), "{overflowing}");
        assert!(overflowing.contains("\"best\":null"), "{overflowing}");
        assert!(!overflowing.contains("inf"), "{overflowing}");
        assert!(!overflowing.contains("NaN"), "{overflowing}");
        assert!(is_valid_json(&overflowing), "{overflowing}");

        let large = parsed_summary_json(&parse("100000000000000000000", None));
        assert!(large.contains("\"value\":100000000000000000000"), "{large}");
        assert!(is_valid_json(&large), "{large}");
    }

    /// The JSON emitter is a machine transport, so it must not round: the old
    /// six-decimal `format_number` collapsed anything below 5e-7 to `0`. Every
    /// finite magnitude must still come out as a plain JSON number that
    /// round-trips back to the same `f64`.
    #[test]
    fn emits_full_precision_numbers_without_exponents() {
        for value in [
            0.0000001_f64,
            0.473176473,
            f64::MIN_POSITIVE,
            5e-324,
            f64::MAX,
            -f64::MAX,
            1e20,
            -0.0,
            0.0,
        ] {
            let mut json = String::new();
            push_reading_json(&mut json, &Reading::number(value, 0.9));
            assert!(is_valid_json(&json), "{value}: {json}");
            let text = json
                .trim_start_matches("{\"kind\":\"number\",\"value\":")
                .trim_end_matches('}');
            assert!(
                !text.contains('e') && !text.contains('E'),
                "{value}: {json}"
            );
            assert_eq!(text.parse::<f64>().expect(text), value, "{json}");
        }

        // -0.0 is normalized so the envelope never carries a "-0" token.
        let mut negative_zero = String::new();
        push_reading_json(&mut negative_zero, &Reading::number(-0.0, 0.9));
        assert_eq!(negative_zero, "{\"kind\":\"number\",\"value\":0}");
    }

    /// The emitter escapes `"`, `\`, and every Unicode `Cc` character.
    ///
    /// That is a superset of what RFC 8259 requires: the grammar forbids raw
    /// characters inside a string only in `U+0000`–`U+001F` (plus the quote and
    /// the backslash), so `U+007F` and `U+0080`–`U+009F` would be legal
    /// unescaped. They are escaped anyway — `char::is_control` is the whole `Cc`
    /// category — because a `` in the envelope survives every consumer,
    /// while a raw one is invisible in a log and mangled by anything that
    /// re-encodes it. This test pins that stricter behavior, not the RFC's
    /// minimum.
    ///
    /// The quote, newline, carriage return and tab arms have named escapes; the
    /// backslash arm and the generic `\u{XXXX}` control arm are the two that no
    /// parse in the corpus reached, and dropping either one of *those* emits a
    /// document that no JSON parser accepts — a lone `\` at the end of `"5 kg\"`
    /// runs the string past its closing quote, and a raw `U+0001` is a control
    /// character where the grammar allows none.
    #[test]
    fn escapes_the_two_structural_characters_and_every_cc_character() {
        let hostile = "a\\b\"c\u{1}d\ne\tf\u{7f}g\rh";
        let mut json = String::new();
        push_json_string(&mut json, hostile);
        assert_eq!(
            json, "\"a\\\\b\\\"c\\u0001d\\ne\\tf\\u007fg\\rh\"",
            "{json}"
        );
        assert!(is_valid_json(&json), "{json}");
        assert_eq!(
            json_string_content(&json).as_deref(),
            Some(hostile),
            "{json}"
        );

        // One character at a time, so a single missing arm cannot hide behind
        // the others in the combined string above.
        for (raw, escaped) in [
            ('\\', "\"\\\\\""),
            ('"', "\"\\\"\""),
            ('\u{1}', "\"\\u0001\""),
            ('\n', "\"\\n\""),
            ('\t', "\"\\t\""),
            ('\r', "\"\\r\""),
            // Legal raw under RFC 8259 — above `U+001F` — and escaped anyway,
            // because they are `Cc`. Pinned so the extra escaping is not
            // dropped as "not required" without the decision being made again.
            ('\u{7f}', "\"\\u007f\""),
            ('\u{9f}', "\"\\u009f\""),
        ] {
            let mut one = String::new();
            push_json_string(&mut one, &raw.to_string());
            assert_eq!(one, escaped, "{raw:?}");
            assert!(is_valid_json(&one), "{raw:?}: {one}");
            assert_eq!(
                json_string_content(&one).as_deref(),
                Some(raw.to_string().as_str()),
                "{raw:?}: {one}"
            );
        }
    }

    /// The same two arms, reached the way a caller reaches them: a backslash or
    /// a control character typed into the input is echoed back through `input`
    /// and through the finding's `ref_text`, so an unescaped one corrupts the
    /// whole envelope rather than one field.
    #[test]
    fn parsed_summary_json_escapes_hostile_input_characters() {
        for input in [
            "5 kg\\",
            "5 kg\u{1}",
            "5 kg\"",
            "5 kg\n",
            "5 kg\t",
            "5 kg\u{7f}",
        ] {
            let json = parsed_summary_json(&parse(input, None));
            assert!(is_valid_json(&json), "{input:?}: {json}");
            // The echoed `input` field decodes back to exactly what was typed.
            let tail = json.split_once("\"input\":").expect("an input field").1;
            let mut rest = tail;
            assert!(json_string(&mut rest), "{input:?}: {json}");
            let literal = &tail[..tail.len() - rest.len()];
            assert_eq!(
                json_string_content(literal).as_deref(),
                Some(input),
                "{input:?}: {json}"
            );
        }
    }

    /// Decodes a JSON string literal back to its Rust value, so the escaping
    /// above can be checked to round-trip rather than merely to parse.
    fn json_string_content(text: &str) -> Option<String> {
        let body = text.strip_prefix('"')?.strip_suffix('"')?;
        let mut out = String::new();
        let mut chars = body.chars();
        while let Some(ch) = chars.next() {
            if ch != '\\' {
                out.push(ch);
                continue;
            }
            match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => {
                    let digits: String = chars.by_ref().take(4).collect();
                    if digits.len() != 4 {
                        return None;
                    }
                    let code = u32::from_str_radix(&digits, 16).ok()?;
                    out.push(char::from_u32(code)?);
                }
                _ => return None,
            }
        }
        Some(out)
    }

    /// A range used to cross the boundary as a bare `{"kind":"range"}`.
    #[test]
    fn serializes_both_range_endpoints_as_nested_readings() {
        let parsed = parse("10 ± 0.5 mm", None);
        let json = parsed_summary_json(&parsed);
        assert!(is_valid_json(&json), "{json}");
        assert!(
            json.contains(
                "\"range\":{\"from\":{\"kind\":\"quantity\",\"value\":0.0095,\"unit\":\"m\",\"dimension\":\"length\"},\
\"to\":{\"kind\":\"quantity\",\"value\":0.0105,\"unit\":\"m\",\"dimension\":\"length\"}}"
            ),
            "{json}"
        );
    }

    /// Minimal structural JSON validator, enough to catch a bare `inf`/`NaN`
    /// token appearing where a number belongs.
    fn is_valid_json(text: &str) -> bool {
        let mut rest = text;
        json_value(&mut rest) && rest.trim().is_empty()
    }

    fn json_value(rest: &mut &str) -> bool {
        *rest = rest.trim_start();
        match rest.chars().next() {
            Some('{') => json_container(rest, '}', true),
            Some('[') => json_container(rest, ']', false),
            Some('"') => json_string(rest),
            Some('t') => json_literal(rest, "true"),
            Some('f') => json_literal(rest, "false"),
            Some('n') => json_literal(rest, "null"),
            Some(_) => json_number(rest),
            None => false,
        }
    }

    fn json_container(rest: &mut &str, close: char, keyed: bool) -> bool {
        *rest = &rest[1..];
        *rest = rest.trim_start();
        if rest.starts_with(close) {
            *rest = &rest[close.len_utf8()..];
            return true;
        }
        loop {
            if keyed {
                *rest = rest.trim_start();
                if !json_string(rest) {
                    return false;
                }
                *rest = rest.trim_start();
                if !rest.starts_with(':') {
                    return false;
                }
                *rest = &rest[1..];
            }
            if !json_value(rest) {
                return false;
            }
            *rest = rest.trim_start();
            if rest.starts_with(',') {
                *rest = &rest[1..];
                continue;
            }
            if rest.starts_with(close) {
                *rest = &rest[close.len_utf8()..];
                return true;
            }
            return false;
        }
    }

    fn json_string(rest: &mut &str) -> bool {
        if !rest.starts_with('"') {
            return false;
        }
        let mut chars = rest.char_indices().skip(1);
        while let Some((idx, ch)) = chars.next() {
            match ch {
                '\\' => {
                    if chars.next().is_none() {
                        return false;
                    }
                }
                '"' => {
                    *rest = &rest[idx + 1..];
                    return true;
                }
                ch if ch.is_control() => return false,
                _ => {}
            }
        }
        false
    }

    fn json_literal(rest: &mut &str, literal: &str) -> bool {
        if let Some(tail) = rest.strip_prefix(literal) {
            *rest = tail;
            true
        } else {
            false
        }
    }

    fn json_number(rest: &mut &str) -> bool {
        let end = rest
            .find(|ch: char| !matches!(ch, '0'..='9' | '-' | '+' | '.' | 'e' | 'E'))
            .unwrap_or(rest.len());
        let (number, tail) = rest.split_at(end);
        if number.is_empty() || number.parse::<f64>().is_err() {
            return false;
        }
        // Rust accepts "inf"/"NaN" in `parse::<f64>`, JSON does not.
        if !number.starts_with(|ch: char| ch.is_ascii_digit() || ch == '-') {
            return false;
        }
        *rest = tail;
        true
    }

    /// Every surface that answers "is this parse acceptable" answers with the
    /// same function.
    ///
    /// There were three answers and they disagreed: the JSON summary said `ok`
    /// whenever there was a `best`, the Rust adapter also demanded an empty
    /// skipped list and refused ambiguity outside `Forgiving`, and the browser
    /// adapter read `error` severity without ever looking at the strictness. A
    /// fourth surface added later fails here unless it delegates too.
    #[test]
    fn every_surface_agrees_on_whether_a_parse_is_acceptable() {
        let corpus = [
            "5 kg",
            "",
            "1,234",
            "about 20kg",
            "3 m 5 m",
            "1'234",
            "5 meterz",
            "180cm",
            "3pm Europe/Paris",
            "next friday",
            "qqqq",
        ];

        for strictness in [
            Strictness::Forgiving,
            Strictness::Confirm,
            Strictness::Strict,
        ] {
            for text in corpus {
                let ctx = ParseCtx {
                    strictness,
                    ..ParseCtx::default()
                };
                let parsed = parse(text, Some(ctx.clone()));
                let decided = accepts(&parsed);
                let label = format!("{text:?} under {strictness:?}");

                // The JSON summary the wasm and browser adapters read.
                let json = parsed_summary_json(&parsed);
                let serialized = json.starts_with("{\"ok\":true,");
                assert_eq!(serialized, decided, "{label}: JSON `ok` disagrees");

                // The Rust adapter that canonicalizes a field.
                let values = canonicalize_values(&[CanonicalizeRequest::new(
                    "field",
                    text,
                    Parser::unrestricted_with_context(ctx.clone()),
                )]);
                assert_eq!(values[0].ok, decided, "{label}: the adapter disagrees");
                assert_eq!(
                    values[0].canonical.is_some(),
                    decided,
                    "{label}: a refused field still carries a canonical value"
                );
                assert_eq!(
                    values[0].message.is_some(),
                    !decided,
                    "{label}: a refusal with nothing to say"
                );

                // And the narrow entry points reach the same function, so a
                // field parsed by kind is not judged by a different rule.
                for narrow in [
                    parse_quantity_fast(text, Some(ctx.clone())),
                    parse_number_fast(text, Some(ctx.clone())),
                ] {
                    let narrow_json = parsed_summary_json(&narrow);
                    assert_eq!(
                        narrow_json.starts_with("{\"ok\":true,"),
                        accepts(&narrow),
                        "{label}: a narrow entry point serializes a different `ok`"
                    );
                }
            }
        }
    }
}
