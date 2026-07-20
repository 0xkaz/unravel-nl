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
