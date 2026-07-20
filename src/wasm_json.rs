use crate::*;

/// Identifies the version of the JSON parsing contract.
pub const CONTRACT_VERSION: &str = "unravel-nl.parse.v1";

/// Returns the current JSON parsing contract version.
pub fn contract_version() -> &'static str {
    CONTRACT_VERSION
}

/// Returns the parse-input schema as a static JSON string.
pub fn parse_input_schema_json() -> &'static str {
    PARSE_INPUT_SCHEMA_JSON
}

/// Returns the parsed-output schema as a static JSON string.
///
/// This schema describes the full parse contract — the shape of a [`Parsed`]
/// value — and requires `input`, `best`, `alternatives`, `suggestions`, and
/// `findings` with `additionalProperties: false`.
///
/// **The WASM/FFI `*_json` functions in this crate do not emit this shape.**
/// They emit a deliberately compact summary envelope — `{ok, input, best,
/// issues}` for the single-value functions, and an array of span objects
/// wrapping that envelope for the `parse_all` and editor functions — which this
/// schema rejects. Treat this schema as the documented contract for
/// callers that build the JSON from [`Parsed`] themselves, not as a validator
/// for the WASM output.
pub fn parsed_output_schema_json() -> &'static str {
    PARSED_OUTPUT_SCHEMA_JSON
}

/// Returns the MCP parse-tool schema as a static JSON string.
///
/// The declared `outputSchema` is [`parsed_output_schema_json`], the full parse
/// contract. **The WASM/FFI `*_json` functions do not produce that shape**, so
/// a validating MCP client cannot be wired straight from those functions to
/// this tool declaration; a host that uses this schema must serialize a
/// [`Parsed`] into the contract shape itself.
pub fn mcp_tool_schema_json() -> &'static str {
    MCP_TOOL_SCHEMA_JSON
}

/// Parses one reading and returns a JSON string at the WASM/FFI boundary around [`parse`].
///
/// # Envelope
///
/// The returned object is a compact summary envelope with exactly the keys
/// `ok`, `input`, `best`, and `issues` — for example
/// `{"ok":true,"input":"about 20kg","best":{"kind":"quantity","value":20,"unit":"kg","dimension":"mass"},"issues":[{"code":"APPROXIMATION","severity":"warning","rank":30,"ref_text":"about"}]}`.
///
/// This is deliberately **not** the parse contract published by
/// [`parsed_output_schema_json`], which requires `input`, `best`,
/// `alternatives`, `suggestions`, and `findings` and forbids extra properties.
/// The envelope trades `alternatives`, `suggestions`, and the structured
/// [`Findings`] split for a flat ranked `issues` list plus an `ok` flag, which
/// is what a UI boundary usually wants. Do not validate it against that schema;
/// use the Rust [`Parsed`] value when you need the full contract.
///
/// Numbers in the envelope are rounded to six decimal places for display, so
/// `2 cups` is emitted as `0.473176` where the Rust reading holds
/// `0.473176473`. Treat the envelope as a presentation format: do arithmetic on
/// the Rust value, not on the JSON.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json(text: &str) -> String {
    parsed_summary_json(&parse(text, None))
}

/// Parses one reading with a locale hint and returns a JSON string at the WASM/FFI boundary around [`parse`].
///
/// Returns the same compact summary envelope as [`parse_json`] — `{ok, input,
/// best, issues}` — which is deliberately not the parse contract published by
/// [`parsed_output_schema_json`].
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json_with_locale(text: &str, locale: &str) -> String {
    parsed_summary_json(&parse(
        text,
        Some(ParseCtx {
            locale: parse_locale_tag(locale),
            ..ParseCtx::default()
        }),
    ))
}

/// Parses one reading with explicit context tags and returns a JSON string at the WASM/FFI boundary around [`parse`].
///
/// Returns the same compact summary envelope as [`parse_json`] — `{ok, input,
/// best, issues}` — which is deliberately not the parse contract published by
/// [`parsed_output_schema_json`].
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    parsed_summary_json(&parse(
        text,
        Some(parse_wasm_context(locale, expected_dimension, strictness)),
    ))
}

/// Parses all readings and returns a JSON string at the WASM/FFI boundary around [`parse_all`].
///
/// # Envelope
///
/// The returned JSON is an array of match objects, each with the keys `start`,
/// `end`, `byteStart`, `byteEnd`, `charStart`, `charEnd`, `text`, and `parsed`,
/// where `parsed` is the compact summary envelope described on [`parse_json`].
/// This is deliberately **not** the parse contract published by
/// [`parsed_output_schema_json`]; that schema describes a single [`Parsed`]
/// value and rejects both the array wrapper and the envelope inside it.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json(text: &str) -> String {
    parsed_matches_summary_json(text, &parse_all(text, None))
}

/// Parses all readings with a locale hint and returns a JSON string at the WASM/FFI boundary around [`parse_all`].
///
/// # Envelope
///
/// The returned JSON is an array of match objects, each with the keys `start`,
/// `end`, `byteStart`, `byteEnd`, `charStart`, `charEnd`, `text`, and `parsed`,
/// where `parsed` is the compact summary envelope described on [`parse_json`].
/// This is deliberately **not** the parse contract published by
/// [`parsed_output_schema_json`]; that schema describes a single [`Parsed`]
/// value and rejects both the array wrapper and the envelope inside it.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json_with_locale(text: &str, locale: &str) -> String {
    parsed_matches_summary_json(
        text,
        &parse_all(
            text,
            Some(ParseCtx {
                locale: parse_locale_tag(locale),
                ..ParseCtx::default()
            }),
        ),
    )
}

/// Parses all readings with explicit context tags and returns a JSON string at the WASM/FFI boundary around [`parse_all`].
///
/// # Envelope
///
/// The returned JSON is an array of match objects, each with the keys `start`,
/// `end`, `byteStart`, `byteEnd`, `charStart`, `charEnd`, `text`, and `parsed`,
/// where `parsed` is the compact summary envelope described on [`parse_json`].
/// This is deliberately **not** the parse contract published by
/// [`parsed_output_schema_json`]; that schema describes a single [`Parsed`]
/// value and rejects both the array wrapper and the envelope inside it.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    parsed_matches_summary_json(
        text,
        &parse_all(
            text,
            Some(parse_wasm_context(locale, expected_dimension, strictness)),
        ),
    )
}

/// Parses editor dimension readings and returns a JSON string at the WASM/FFI boundary around [`parse_dimensions_for_editor`].
///
/// # Envelope
///
/// The returned JSON is an array of match objects, each with the keys `start`,
/// `end`, `byteStart`, `byteEnd`, `charStart`, `charEnd`, `text`, and `parsed`,
/// where `parsed` is the compact summary envelope described on [`parse_json`].
/// This is deliberately **not** the parse contract published by
/// [`parsed_output_schema_json`]; that schema describes a single [`Parsed`]
/// value and rejects both the array wrapper and the envelope inside it.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_dimensions_for_editor_json(text: &str) -> String {
    parsed_matches_summary_json(text, &parse_dimensions_for_editor(text, None))
}

/// Parses editor dimensions with explicit context tags and returns a JSON string at the WASM/FFI boundary around [`parse_dimensions_for_editor`].
///
/// # Envelope
///
/// The returned JSON is an array of match objects, each with the keys `start`,
/// `end`, `byteStart`, `byteEnd`, `charStart`, `charEnd`, `text`, and `parsed`,
/// where `parsed` is the compact summary envelope described on [`parse_json`].
/// This is deliberately **not** the parse contract published by
/// [`parsed_output_schema_json`]; that schema describes a single [`Parsed`]
/// value and rejects both the array wrapper and the envelope inside it.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_dimensions_for_editor_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    let mut ctx = parse_wasm_context(locale, expected_dimension, strictness);
    ctx.purpose = ParsePurpose::DimensionEditor;
    parsed_matches_summary_json(text, &parse_dimensions_for_editor(text, Some(ctx)))
}

#[cfg(feature = "wasm")]
pub(crate) fn parse_locale_tag(text: &str) -> Option<Locale> {
    match text {
        "" => None,
        "ja" | "ja-JP" => Some(Locale::Ja),
        "en" => Some(Locale::En),
        "en-US" => Some(Locale::EnUs),
        "en-GB" | "en-UK" => Some(Locale::EnGb),
        other => Some(Locale::Other(other.to_owned())),
    }
}

#[cfg(feature = "wasm")]
pub(crate) fn parse_wasm_context(
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> ParseCtx {
    ParseCtx {
        locale: parse_locale_tag(locale),
        expected_dimension: parse_dimension_tag(expected_dimension),
        strictness: parse_strictness_tag(strictness),
        ..ParseCtx::default()
    }
}

#[cfg(feature = "wasm")]
pub(crate) fn parse_dimension_tag(text: &str) -> Option<Dimension> {
    match text {
        "" => None,
        "length" => Some(Dimension::Length),
        "area" => Some(Dimension::Area),
        "mass" => Some(Dimension::Mass),
        "time" => Some(Dimension::Time),
        "volume" => Some(Dimension::Volume),
        "currency" => Some(Dimension::Currency),
        "temperature" => Some(Dimension::Temperature),
        "speed" => Some(Dimension::Speed),
        "data" => Some(Dimension::Data),
        "data_rate" => Some(Dimension::DataRate),
        "flow_rate" => Some(Dimension::FlowRate),
        "concentration" => Some(Dimension::Concentration),
        "acceleration" => Some(Dimension::Acceleration),
        "force" => Some(Dimension::Force),
        "torque" => Some(Dimension::Torque),
        "pressure" => Some(Dimension::Pressure),
        "power" => Some(Dimension::Power),
        "charge" => Some(Dimension::Charge),
        "voltage" => Some(Dimension::Voltage),
        "current" => Some(Dimension::Current),
        "resistance" => Some(Dimension::Resistance),
        "illuminance" => Some(Dimension::Illuminance),
        "radiation_equivalent_dose" => Some(Dimension::RadiationEquivalentDose),
        "radioactivity" => Some(Dimension::Radioactivity),
        _ => None,
    }
}

#[cfg(feature = "wasm")]
pub(crate) fn parse_strictness_tag(text: &str) -> Strictness {
    match text {
        "confirm" => Strictness::Confirm,
        "strict" => Strictness::Strict,
        _ => Strictness::Forgiving,
    }
}

/// Covers the surface that only exists in the shipped WASM build.
///
/// `make build-wasm` ships exactly `--features wasm`, and until this module
/// existed none of it had a Rust test: the tag parsers and the eight `*_json`
/// entry points were exercised only by `tests/wasm_node_smoke.mjs`, which needs
/// a `wasm-pack` build and is not part of `make check`. The tag parsers fail
/// *silently* — an unrecognized dimension tag yields `None` and an unrecognized
/// strictness tag yields `Forgiving` — so a typo there changes which readings
/// are accepted without any error surfacing.
#[cfg(all(test, feature = "wasm"))]
mod wasm_tests {
    use super::*;

    /// Every `Dimension` variant, written out rather than derived, so that
    /// adding a variant without teaching `parse_dimension_tag` about it fails
    /// here instead of silently becoming an ignored tag.
    const ALL_DIMENSIONS: [Dimension; 24] = [
        Dimension::Length,
        Dimension::Area,
        Dimension::Mass,
        Dimension::Time,
        Dimension::Volume,
        Dimension::Currency,
        Dimension::Temperature,
        Dimension::Speed,
        Dimension::Data,
        Dimension::DataRate,
        Dimension::FlowRate,
        Dimension::Concentration,
        Dimension::Acceleration,
        Dimension::Force,
        Dimension::Torque,
        Dimension::Pressure,
        Dimension::Power,
        Dimension::Charge,
        Dimension::Voltage,
        Dimension::Current,
        Dimension::Resistance,
        Dimension::Illuminance,
        Dimension::RadiationEquivalentDose,
        Dimension::Radioactivity,
    ];

    /// `parse_dimension_tag` hand-mirrors `Dimension::as_str`, and a missing arm
    /// is not an error — it is `None`, which quietly drops the caller's
    /// expected-dimension hint.
    #[test]
    fn every_dimension_tag_round_trips_through_as_str() {
        for dimension in ALL_DIMENSIONS {
            assert_eq!(
                parse_dimension_tag(dimension.as_str()),
                Some(dimension),
                "tag {:?} does not round-trip",
                dimension.as_str()
            );
        }

        // Distinct variants must not collapse onto one tag.
        let mut tags: Vec<&str> = ALL_DIMENSIONS.iter().map(|d| d.as_str()).collect();
        tags.sort_unstable();
        let unique = {
            let mut unique = tags.clone();
            unique.dedup();
            unique
        };
        assert_eq!(tags, unique, "two dimensions share a tag");

        assert_eq!(parse_dimension_tag(""), None);
        assert_eq!(parse_dimension_tag("nonsense"), None);
        // Tags are matched exactly: no case folding, no separator tolerance.
        assert_eq!(parse_dimension_tag("Length"), None);
        assert_eq!(parse_dimension_tag("data-rate"), None);
    }

    #[test]
    fn maps_known_strictness_tags_and_falls_back_by_intent() {
        assert_eq!(parse_strictness_tag("confirm"), Strictness::Confirm);
        assert_eq!(parse_strictness_tag("strict"), Strictness::Strict);
        assert_eq!(parse_strictness_tag("forgiving"), Strictness::Forgiving);

        // The fallback is deliberate and total: an empty tag means "unset", and
        // an unrecognized one — including a typo such as "stict" — is *not* an
        // error, it silently downgrades to the most permissive policy. This is
        // asserted so the silence is a decision on record rather than an
        // accident, and so a future change to error out has to update this test.
        for unknown in ["", "stict", "STRICT", "Confirm", "nonsense"] {
            assert_eq!(
                parse_strictness_tag(unknown),
                Strictness::Forgiving,
                "{unknown:?}"
            );
        }
    }

    #[test]
    fn maps_locale_tags() {
        assert_eq!(parse_locale_tag(""), None);
        assert_eq!(parse_locale_tag("ja"), Some(Locale::Ja));
        assert_eq!(parse_locale_tag("ja-JP"), Some(Locale::Ja));
        assert_eq!(parse_locale_tag("en"), Some(Locale::En));
        assert_eq!(parse_locale_tag("en-US"), Some(Locale::EnUs));
        assert_eq!(parse_locale_tag("en-GB"), Some(Locale::EnGb));
        assert_eq!(parse_locale_tag("en-UK"), Some(Locale::EnGb));
        // Anything else is carried through verbatim rather than dropped.
        assert_eq!(
            parse_locale_tag("fr-CA"),
            Some(Locale::Other("fr-CA".to_owned()))
        );
        assert_eq!(
            parse_locale_tag("nonsense"),
            Some(Locale::Other("nonsense".to_owned()))
        );
    }

    #[test]
    fn builds_a_context_from_all_three_tags() {
        let ctx = parse_wasm_context("ja-JP", "area", "strict");
        assert_eq!(ctx.locale, Some(Locale::Ja));
        assert_eq!(ctx.expected_dimension, Some(Dimension::Area));
        assert_eq!(ctx.strictness, Strictness::Strict);

        // Every tag empty is the same as the default context.
        assert_eq!(parse_wasm_context("", "", ""), ParseCtx::default());

        // Unrecognized tags are absorbed, not reported.
        let sloppy = parse_wasm_context("", "lenght", "stict");
        assert_eq!(sloppy.expected_dimension, None);
        assert_eq!(sloppy.strictness, Strictness::Forgiving);
    }

    #[test]
    fn parse_json_emits_the_summary_envelope() {
        let json = parse_json("about 20kg");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"ok\":true"), "{json}");
        assert!(json.contains("\"input\":\"about 20kg\""), "{json}");
        assert!(json.contains("\"unit\":\"kg\""), "{json}");
        assert!(json.contains("\"dimension\":\"mass\""), "{json}");
        assert!(json.contains("\"code\":\"APPROXIMATION\""), "{json}");

        // A refused input still produces a well-formed envelope carrying why.
        let failed = parse_json("3pm Europe/Paris");
        assert!(is_valid_json(&failed), "{failed}");
        assert!(failed.contains("\"ok\":false"), "{failed}");
        assert!(failed.contains("\"best\":null"), "{failed}");
        assert!(
            failed.contains("\"code\":\"TIMEZONE_UNSUPPORTED\""),
            "{failed}"
        );
    }

    #[test]
    fn parse_json_with_locale_applies_the_locale() {
        let json = parse_json_with_locale("5尺3寸", "ja");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"ok\":true"), "{json}");
        assert!(json.contains("\"dimension\":\"length\""), "{json}");

        // The locale reaches the parser: the imperial cup wins under en-GB and
        // the US cup under en-US, and the envelope shows which. The values are
        // the six-decimal renderings of 0.473176473 L and 0.56826125 L —
        // `format_number` rounds at this boundary, so the envelope is not a
        // full-precision transport. The unrounded winners are pinned in
        // `tests/issue_codes.rs`.
        let us = parse_json_with_locale("2 cups", "en-US");
        let gb = parse_json_with_locale("2 cups", "en-GB");
        assert!(us.contains("\"value\":0.473176"), "{us}");
        assert!(gb.contains("\"value\":0.568261"), "{gb}");
    }

    #[test]
    fn parse_json_with_context_applies_dimension_and_strictness() {
        let json = parse_json_with_context("3640", "", "length", "forgiving");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"code\":\"UNIT_ASSUMED\""), "{json}");

        // Without the dimension tag there is nothing to assume a unit from.
        let bare = parse_json_with_context("3640", "", "", "forgiving");
        assert!(is_valid_json(&bare), "{bare}");
        assert!(!bare.contains("UNIT_ASSUMED"), "{bare}");

        // A misspelled dimension tag behaves exactly like no tag at all — the
        // silent failure this module exists to make visible.
        let typo = parse_json_with_context("3640", "", "lenght", "forgiving");
        assert_eq!(typo, bare);
    }

    #[test]
    fn parse_all_json_carries_the_position_triple() {
        let json = parse_all_json("3m and 20kg");
        assert!(is_valid_json(&json), "{json}");
        for key in [
            "\"start\":",
            "\"end\":",
            "\"byteStart\":",
            "\"byteEnd\":",
            "\"charStart\":",
            "\"charEnd\":",
            "\"text\":",
            "\"parsed\":",
        ] {
            assert!(json.contains(key), "{key} missing from {json}");
        }
        assert!(json.starts_with('['), "{json}");

        assert_eq!(parse_all_json(""), "[]");
    }

    #[test]
    fn parse_all_json_with_locale_reports_char_offsets_past_multibyte_text() {
        let json = parse_all_json_with_locale("3m×4m のLDK", "ja");
        assert!(is_valid_json(&json), "{json}");
        for key in ["\"start\":", "\"byteStart\":", "\"charStart\":"] {
            assert!(json.contains(key), "{key} missing from {json}");
        }
        // The char offsets must differ from the byte offsets once a multi-byte
        // character precedes a match, which is the whole point of shipping both.
        let matches = parse_all("3m×4m のLDK", None);
        let last = matches.last().expect("a match");
        assert!(last.start > 0);
        assert!(
            byte_to_char_offset("3m×4m のLDK", last.start) < last.start,
            "{last:?}"
        );
    }

    #[test]
    fn parse_all_json_with_context_applies_the_tags() {
        let json = parse_all_json_with_context("幅3640 高さ2400", "ja", "length", "forgiving");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"charStart\":"), "{json}");
    }

    #[test]
    fn parse_dimensions_for_editor_json_extracts_labelled_lengths() {
        let json = parse_dimensions_for_editor_json("幅3m 奥行4m");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"dimension\":\"length\""), "{json}");
        assert!(json.contains("\"charStart\":"), "{json}");

        // An unlabelled bare number is not a dimension, so nothing is extracted.
        assert_eq!(parse_dimensions_for_editor_json("3640"), "[]");
    }

    #[test]
    fn parse_dimensions_for_editor_json_with_context_applies_the_tags() {
        let json =
            parse_dimensions_for_editor_json_with_context("幅3m 奥行4m", "ja", "length", "strict");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"dimension\":\"length\""), "{json}");
    }

    /// Minimal structural JSON check — enough to catch an unescaped string, an
    /// unbalanced container, or a bare `inf`/`NaN` where a number belongs.
    /// A copy of the checker in `json_out.rs`, kept local so that module's
    /// test-only helpers stay private to it.
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
        if !number.starts_with(|ch: char| ch.is_ascii_digit() || ch == '-') {
            return false;
        }
        *rest = tail;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_schema_and_mcp_helpers() {
        assert_eq!(contract_version(), "unravel-nl.parse.v1");
        assert!(parse_input_schema_json().contains("\"text\""));
        assert!(parse_input_schema_json().contains("\"strictness\""));
        assert!(parse_input_schema_json().contains("\"timezone\""));
        assert!(parsed_output_schema_json().contains("\"findings\""));
        assert!(parsed_output_schema_json().contains("\"currency\""));
        assert!(parsed_output_schema_json().contains("\"temperature\""));
        assert!(parsed_output_schema_json().contains("\"recurrence\""));
        assert!(parsed_output_schema_json().contains("\"timezone\""));
        assert!(mcp_tool_schema_json().contains("unravel_nl_parse"));
        assert!(mcp_tool_schema_json().contains("inputSchema"));
        assert!(mcp_tool_schema_json().contains("outputSchema"));
        assert_eq!(
            IssueCode::TimezoneUnsupported.as_str(),
            "TIMEZONE_UNSUPPORTED"
        );
    }

    /// The WASM/FFI envelope is deliberately not the published parse contract,
    /// as the docs on the schema accessors and the `*_json` functions now say.
    #[test]
    fn wasm_envelope_is_not_the_published_output_schema() {
        let envelope = parsed_summary_json(&parse("about 20kg", None));
        for key in ["\"ok\":", "\"input\":", "\"best\":", "\"issues\":"] {
            assert!(envelope.contains(key), "envelope should carry {key}");
        }
        // The contract's required members are absent from the envelope.
        for key in ["\"alternatives\"", "\"suggestions\"", "\"findings\""] {
            assert!(!envelope.contains(key), "envelope should not carry {key}");
        }
        // ...yet the schema requires them and forbids the envelope's extras.
        let schema = parsed_output_schema_json();
        assert!(schema.contains(
            "\"required\": [\"input\", \"best\", \"alternatives\", \"suggestions\", \"findings\"]"
        ));
        assert!(schema.contains("\"additionalProperties\": false"));
        assert!(!schema.contains("\"ok\""));
        // And the MCP declaration still points its outputSchema at that contract.
        assert!(mcp_tool_schema_json().contains("parsed-output.json"));
    }

    /// The input schema's prose must match the behaviour documented on
    /// [`ParseCtx`].
    #[test]
    fn input_schema_prose_matches_behaviour() {
        let schema = parse_input_schema_json();
        assert!(schema.contains("Reserved for adapter layers and currently ignored by the parser"));
        assert!(schema.contains("This is a hard filter, not a hint"));
        assert!(schema.contains("This does not constrain parsing"));
    }
}
