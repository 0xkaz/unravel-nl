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
pub fn parsed_output_schema_json() -> &'static str {
    PARSED_OUTPUT_SCHEMA_JSON
}

/// Returns the MCP parse-tool schema as a static JSON string.
pub fn mcp_tool_schema_json() -> &'static str {
    MCP_TOOL_SCHEMA_JSON
}

/// Parses one reading and returns a JSON string at the WASM/FFI boundary around [`parse`].
#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json(text: &str) -> String {
    parsed_summary_json(&parse(text, None))
}

/// Parses one reading with a locale hint and returns a JSON string at the WASM/FFI boundary around [`parse`].
#[cfg(feature = "wasm")]
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
#[cfg(feature = "wasm")]
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
#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json(text: &str) -> String {
    parsed_matches_summary_json(text, &parse_all(text, None))
}

/// Parses all readings with a locale hint and returns a JSON string at the WASM/FFI boundary around [`parse_all`].
#[cfg(feature = "wasm")]
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
#[cfg(feature = "wasm")]
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
#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_dimensions_for_editor_json(text: &str) -> String {
    parsed_matches_summary_json(text, &parse_dimensions_for_editor(text, None))
}

/// Parses editor dimensions with explicit context tags and returns a JSON string at the WASM/FFI boundary around [`parse_dimensions_for_editor`].
#[cfg(feature = "wasm")]
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
}
