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
/// Numbers in the envelope carry the full `f64` value, not a display rounding:
/// `2 cups` is emitted as `0.473176473`, and a value below the old six-decimal
/// rounding — `0.0000001 m` — survives instead of collapsing to `0`. Values are
/// written in plain decimal notation (never exponent form), so `JSON.parse`
/// accepts every finite magnitude.
///
/// A `range` reading also carries a `range` object with `from` and `to`, each
/// a nested reading with the same fields a top-level reading has — without it
/// the envelope would report `ok:true` with no finding while both endpoints
/// were gone.
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
        // the US cup under en-US, and the envelope shows which. The envelope is
        // a machine transport, so it carries the full `f64` — 0.473176473 L and
        // 0.56826125 L — rather than a six-decimal display rounding. The same
        // winners are pinned in `tests/issue_codes.rs`.
        //
        // The whole envelope is compared rather than a substring of the number:
        // `contains("\"value\":0.473176")` is a *prefix* of the true value and
        // would pass just as happily against a six-decimal rounding.
        let us = parse_json_with_locale("2 cups", "en-US");
        let gb = parse_json_with_locale("2 cups", "en-GB");
        assert_eq!(
            us,
            "{\"ok\":true,\"input\":\"2 cups\",\"best\":{\"kind\":\"quantity\",\
             \"value\":0.473176473,\"unit\":\"L\",\"dimension\":\"volume\"},\
             \"issues\":[{\"code\":\"AMBIGUOUS_UNIT\",\"severity\":\"warning\",\
             \"rank\":55,\"ref_text\":\"cups\"}]}"
        );
        assert_eq!(
            gb,
            "{\"ok\":true,\"input\":\"2 cups\",\"best\":{\"kind\":\"quantity\",\
             \"value\":0.56826125,\"unit\":\"L\",\"dimension\":\"volume\"},\
             \"issues\":[{\"code\":\"AMBIGUOUS_UNIT\",\"severity\":\"warning\",\
             \"rank\":55,\"ref_text\":\"cups\"}]}"
        );
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

    /// The tags must reach the parser, not merely be accepted. Asserting the
    /// envelope is well-formed and carries `charStart` would pass with the tags
    /// thrown away, so the assertion is the *difference* the tags make: with an
    /// expected dimension the bare millimetre numbers gain `UNIT_ASSUMED`, and
    /// without one they do not.
    #[test]
    fn parse_all_json_with_context_applies_the_tags() {
        let tagged = parse_all_json_with_context("幅3640 高さ2400", "ja", "length", "forgiving");
        let untagged = parse_all_json("幅3640 高さ2400");
        assert!(is_valid_json(&tagged), "{tagged}");
        assert!(is_valid_json(&untagged), "{untagged}");
        assert!(tagged.contains("\"charStart\":"), "{tagged}");

        assert_eq!(
            tagged.matches("\"code\":\"UNIT_ASSUMED\"").count(),
            2,
            "{tagged}"
        );
        assert!(!untagged.contains("UNIT_ASSUMED"), "{untagged}");
        assert_ne!(tagged, untagged);

        // A misspelled dimension tag is absorbed silently and behaves exactly
        // like no tag at all.
        assert_eq!(
            parse_all_json_with_context("幅3640 高さ2400", "ja", "lenght", "forgiving"),
            untagged
        );
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

    /// Labelled, unambiguous, exactly-typed lengths are the case where the tags
    /// have nothing left to change: for `幅3m 奥行4m` the tagged output is
    /// byte-identical to the untagged one. That is worth pinning — a tag must
    /// not perturb input it has no business perturbing — but it is *not*
    /// evidence that the tags are read at all; see
    /// `parse_dimensions_for_editor_json_with_context_applies_the_tags` for
    /// that.
    #[test]
    fn parse_dimensions_for_editor_json_with_context_leaves_exact_matches_alone() {
        let json =
            parse_dimensions_for_editor_json_with_context("幅3m 奥行4m", "ja", "length", "strict");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"dimension\":\"length\""), "{json}");
        assert_eq!(json, parse_dimensions_for_editor_json("幅3m 奥行4m"));
    }

    /// Inputs where each tag demonstrably changes the output.
    #[test]
    fn parse_dimensions_for_editor_json_with_context_applies_the_tags() {
        // The dimension tag is a filter here, unlike in `parse`: a width label
        // survives `length` and is dropped entirely under `mass`.
        let as_length =
            parse_dimensions_for_editor_json_with_context("幅3640", "ja", "length", "forgiving");
        assert!(is_valid_json(&as_length), "{as_length}");
        assert!(
            as_length.contains("\"code\":\"UNIT_ASSUMED\""),
            "{as_length}"
        );
        assert_eq!(
            parse_dimensions_for_editor_json_with_context("幅3640", "ja", "mass", "forgiving"),
            "[]"
        );

        // The strictness tag is read too: an approximate quantity is kept under
        // `forgiving` and refused under `strict`.
        let forgiving =
            parse_dimensions_for_editor_json_with_context("幅約3m", "ja", "length", "forgiving");
        assert!(is_valid_json(&forgiving), "{forgiving}");
        assert!(
            forgiving.contains("\"code\":\"APPROXIMATION\""),
            "{forgiving}"
        );
        assert_eq!(
            parse_dimensions_for_editor_json_with_context("幅約3m", "ja", "length", "strict"),
            "[]"
        );
        // And the untagged call is the forgiving one, so `strict` really is the
        // tag doing the work.
        assert_eq!(parse_dimensions_for_editor_json("幅約3m"), forgiving);
    }

    /// A range crossing the boundary used to arrive as a bare
    /// `{"kind":"range"}`: both endpoints were dropped while the envelope still
    /// said `ok:true` with an empty `issues` list, so a JS caller had no way to
    /// see the loss. Every endpoint value and unit must survive.
    #[test]
    fn parse_json_carries_both_range_endpoints() {
        // input, from-value, to-value, unit
        let cases: [(&str, &str, &str, &str); 4] = [
            ("10 ± 0.5 mm", "0.0095", "0.0105", "m"),
            ("100-120㎡", "100", "120", "m2"),
            ("between 5 and 10 kg", "5", "10", "kg"),
            ("2〜3日", "172800", "259200", "s"),
        ];
        for (input, from, to, unit) in cases {
            let json = parse_json(input);
            assert!(is_valid_json(&json), "{input}: {json}");
            assert!(json.contains("\"kind\":\"range\""), "{input}: {json}");
            assert!(json.contains("\"range\":{\"from\":"), "{input}: {json}");
            assert!(json.contains(",\"to\":"), "{input}: {json}");
            assert!(
                json.contains(&format!("\"value\":{from},")),
                "{input}: {json}"
            );
            assert!(
                json.contains(&format!("\"value\":{to},")),
                "{input}: {json}"
            );
            // Both endpoints must carry the unit, not just the container.
            assert_eq!(
                json.matches(&format!("\"unit\":\"{unit}\"")).count(),
                2,
                "{input}: {json}"
            );
            // The endpoints are full readings, not a bare pair of numbers.
            assert!(json.contains("\"dimension\":"), "{input}: {json}");
        }
    }

    /// The envelope used to run every value through `format_number`, which
    /// rounds to six decimals — a real quantity below that threshold became
    /// exactly `0` with no finding at all.
    #[test]
    fn parse_json_keeps_values_below_the_old_rounding_threshold() {
        let json = parse_json("0.0000001 m");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"value\":0.0000001"), "{json}");
        assert!(!json.contains("\"value\":0,"), "{json}");
        assert!(!json.contains("\"value\":0}"), "{json}");
        // Plain decimal notation only: `1e-7` still parses in Rust but is the
        // shape a strict JSON consumer is most likely to trip over.
        assert!(!json.contains("e-"), "{json}");
        assert_eq!(
            parse("0.0000001 m", None).best.and_then(|best| best.value),
            Some(0.0000001)
        );
    }

    /// `2 cups` holds 0.473176473 L; the envelope used to ship 0.473176.
    #[test]
    fn parse_json_serializes_cups_at_full_precision() {
        let json = parse_json_with_locale("2 cups", "en-US");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"value\":0.473176473"), "{json}");
        let best = parse("2 cups", None).best.expect("a reading");
        assert!(json.contains(&format!("\"value\":{}", best.value.expect("a value"))));
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
