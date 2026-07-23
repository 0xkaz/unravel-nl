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
/// wrapping that envelope for the editor functions — which this
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

/// Parses one reading through the minimal [`Parser`] and returns JSON.
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
    parsed_summary_json(&Parser::default().parse(text))
}

/// Parses one reading through the minimal [`Parser`] with a locale hint.
///
/// Returns the same compact summary envelope as [`parse_json`] — `{ok, input,
/// best, issues}` — which is deliberately not the parse contract published by
/// [`parsed_output_schema_json`].
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json_with_locale(text: &str, locale: &str) -> String {
    let dimensions = crate::parser::minimal_dimensions();
    parsed_summary_json(
        &Parser::with_context(
            dimensions,
            ParseCtx {
                locale: parse_locale_tag(locale),
                expected_dimensions: dimensions,
                ..ParseCtx::default()
            },
        )
        .parse(text),
    )
}

/// Parses one reading through a dimension-scoped [`Parser`] and returns JSON.
///
/// Returns the same compact summary envelope as [`parse_json`] — `{ok, input,
/// best, issues}` — which is deliberately not the parse contract published by
/// [`parsed_output_schema_json`].
///
/// An `expected_dimension` tag that names nothing this build can read is
/// refused rather than parsed.
#[cfg(feature = "wasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm")))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    if unreadable_dimension_tag(expected_dimension) {
        return parsed_summary_json(&unreadable_dimension_tag_parsed(text, expected_dimension));
    }
    parsed_summary_json(&parse_wasm_parser(locale, expected_dimension, strictness).parse(text))
}

/// Extracts editor dimensions through the minimal [`Parser`] and returns JSON.
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
    parsed_matches_summary_json(text, &Parser::default().parse_dimensions_for_editor(text))
}

/// Extracts editor dimensions through a scoped [`Parser`] and returns JSON.
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
    if unreadable_dimension_tag(expected_dimension) {
        return unreadable_dimension_tag_matches_json(text, expected_dimension);
    }
    parsed_matches_summary_json(
        text,
        &parse_wasm_parser(locale, expected_dimension, strictness)
            .parse_dimensions_for_editor(text),
    )
}

#[cfg(feature = "wasm")]
fn parse_wasm_parser(locale: &str, expected_dimension: &str, strictness: &str) -> Parser {
    let mut ctx = parse_wasm_context(locale, expected_dimension, strictness);
    let dimensions = if ctx.expected_dimensions.is_empty() {
        crate::parser::minimal_dimensions()
    } else {
        ctx.expected_dimensions
    };
    ctx.expected_dimensions = dimensions;
    Parser::with_context(dimensions, ctx)
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
        expected_dimensions: parse_dimension_set_tag(expected_dimension),
        strictness: parse_strictness_tag(strictness),
        ..ParseCtx::default()
    }
}

/// Reads the `expected_dimension` tag, which may name several dimensions.
///
/// The boundary is a fixed `&str` signature, so a set is written as a
/// comma-separated list: `"length"`, `"length,area"`. An empty tag is the empty
/// set — no restriction — and an unrecognized name is dropped, as every tag
/// parser here drops what it cannot read. When *every* name is unreadable the
/// result would be the empty set, and the caller's entry point refuses the call
/// instead of running it unrestricted; see [`unreadable_dimension_tag`].
#[cfg(feature = "wasm")]
pub(crate) fn parse_dimension_set_tag(text: &str) -> DimensionSet {
    text.split(',')
        .filter_map(|tag| parse_dimension_tag(tag.trim()))
        .collect()
}

/// Whether a tag names dimensions but none this build can read.
///
/// Dropping *some* members of a list is documented and harmless: the ones that
/// were read still restrict the parse, so the field is narrower than it asked
/// for, never wider. Dropping *all* of them is the opposite — `"lenght"` would
/// leave the empty set, which is no restriction at all, so a single typo turns
/// a hard filter into none and `5 kg` comes back from a length field with
/// `ok:true` and nothing said. That failure is silent and it fails open, so the
/// entry points that take this tag refuse the call rather than answer it under
/// a policy they could not read.
#[cfg(feature = "wasm")]
pub(crate) fn unreadable_dimension_tag(tag: &str) -> bool {
    !tag.trim().is_empty() && parse_dimension_set_tag(tag).is_empty()
}

/// The refusal a `*_with_context` entry point returns for such a tag.
#[cfg(feature = "wasm")]
pub(crate) fn unreadable_dimension_tag_parsed(text: &str, tag: &str) -> Parsed {
    let mut parsed = parsed_shell(text, &ParseCtx::default());
    parsed.findings.skipped.push(skipped_with_span(
        text,
        &format!(
            "expected_dimension names no readable dimension: {tag:?}; the call was refused rather than parsed with no restriction at all"
        ),
        IssueCode::RejectedByPolicy,
        span(text),
    ));
    parsed
}

/// The same refusal, in the array envelope the scanning entry points return.
///
/// One match spanning the whole input, because an empty array is exactly the
/// silence this refuses to answer with.
#[cfg(feature = "wasm")]
pub(crate) fn unreadable_dimension_tag_matches_json(text: &str, tag: &str) -> String {
    parsed_matches_summary_json(
        text,
        &[ParsedMatch {
            start: 0,
            end: text.len(),
            text: text.to_owned(),
            parsed: unreadable_dimension_tag_parsed(text, tag),
        }],
    )
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
        assert_eq!(ctx.expected_dimensions, DimensionSet::from(Dimension::Area));
        assert_eq!(ctx.strictness, Strictness::Strict);

        // Every tag empty is the same as the default context.
        assert_eq!(parse_wasm_context("", "", ""), ParseCtx::default());

        // Unrecognized tags are absorbed, not reported.
        let sloppy = parse_wasm_context("", "lenght", "stict");
        assert_eq!(sloppy.expected_dimensions, DimensionSet::new());
        assert_eq!(sloppy.strictness, Strictness::Forgiving);

        // Every name the published input schema offers is one this reads back,
        // so the contract and the boundary cannot drift apart.
        for dimension in ALL_DIMENSIONS {
            let name = dimension.as_str();
            assert_eq!(parse_dimension_tag(name), Some(dimension), "{name}");
            assert!(parse_input_schema_json().contains(name), "{name}");
        }

        // Several dimensions are written as a comma-separated list, and an
        // unreadable member is dropped without taking the rest with it.
        assert_eq!(
            parse_wasm_context("", "length, area", "").expected_dimensions,
            DimensionSet::of(&[Dimension::Length, Dimension::Area])
        );
        assert_eq!(
            parse_wasm_context("", "length,lenght", "").expected_dimensions,
            DimensionSet::from(Dimension::Length)
        );
    }

    #[test]
    fn parse_json_emits_the_summary_envelope() {
        let json = parse_json("about 20m");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"ok\":true"), "{json}");
        assert!(json.contains("\"input\":\"about 20m\""), "{json}");
        assert!(json.contains("\"unit\":\"m\""), "{json}");
        assert!(json.contains("\"dimension\":\"length\""), "{json}");
        assert!(json.contains("\"code\":\"APPROXIMATION\""), "{json}");

        // A refused input still produces a well-formed envelope carrying why.
        let failed = parse_json_with_context("3pm Europe/Paris", "", "time", "");
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
        let us = parse_json_with_context("2 cups", "en-US", "volume", "");
        let gb = parse_json_with_context("2 cups", "en-GB", "volume", "");
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

        // Without a dimension tag, the WASM boundary uses the same minimal
        // length-and-area registry as `Parser::default`.
        let bare = parse_json_with_context("3640", "", "", "forgiving");
        assert!(is_valid_json(&bare), "{bare}");
        assert!(bare.contains("UNIT_ASSUMED"), "{bare}");
        assert_eq!(bare, json);

        // A misspelled dimension tag is refused, not absorbed: behaving like no
        // tag at all would turn a hard filter into none, and a length field
        // handed `5 kg` would answer `ok:true` with nothing said.
        let typo = parse_json_with_context("3640", "", "lenght", "forgiving");
        assert_ne!(typo, bare);
        assert_eq!(
            typo,
            "{\"ok\":false,\"input\":\"3640\",\"best\":null,\"issues\":[\
             {\"code\":\"REJECTED_BY_POLICY\",\"severity\":\"error\",\"rank\":90,\
             \"ref_text\":\"3640\"}]}"
        );
        assert_eq!(
            parse_json_with_context("5 kg", "", "lenght", ""),
            "{\"ok\":false,\"input\":\"5 kg\",\"best\":null,\"issues\":[\
             {\"code\":\"REJECTED_BY_POLICY\",\"severity\":\"error\",\"rank\":90,\
             \"ref_text\":\"5 kg\"}]}"
        );
        // Only the all-unreadable tag is refused. A list that keeps a readable
        // member still parses, under the members that were read.
        assert_eq!(
            parse_json_with_context("3640", "", "length,lenght", "forgiving"),
            parse_json_with_context("3640", "", "length", "forgiving")
        );
    }

    #[test]
    fn parse_dimensions_for_editor_json_extracts_labelled_lengths() {
        let json = parse_dimensions_for_editor_json("幅3m 奥行4m");
        assert!(is_valid_json(&json), "{json}");
        assert!(json.contains("\"dimension\":\"length\""), "{json}");
        assert!(json.starts_with('['), "{json}");
        // Every key of the span envelope, including the byte/char position
        // triple, which callers index the original string with.
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
        // The char offsets must differ from the byte offsets once a multi-byte
        // character precedes a match, which is the whole point of shipping both.
        let matches = parse_dimensions_for_editor("幅3m 奥行4m", None);
        let last = matches.last().expect("a match");
        assert!(last.start > 0);
        assert!(
            byte_to_char_offset("幅3m 奥行4m", last.start) < last.start,
            "{last:?}"
        );

        // An unlabelled bare number is not a dimension, so nothing is extracted.
        assert_eq!(parse_dimensions_for_editor_json("3640"), "[]");
        assert_eq!(parse_dimensions_for_editor_json(""), "[]");
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
        // survives `length`, and under `mass` the millimetre length it would
        // have been is refused — out loud, keeping its span, rather than
        // vanishing from the results.
        let as_length =
            parse_dimensions_for_editor_json_with_context("幅3640", "ja", "length", "forgiving");
        assert!(is_valid_json(&as_length), "{as_length}");
        assert!(
            as_length.contains("\"code\":\"UNIT_ASSUMED\""),
            "{as_length}"
        );
        assert_eq!(
            parse_dimensions_for_editor_json_with_context("幅3640", "ja", "mass", "forgiving"),
            "[{\"start\":3,\"end\":7,\"byteStart\":3,\"byteEnd\":7,\"charStart\":1,\
             \"charEnd\":5,\"text\":\"3640\",\"parsed\":{\"ok\":false,\"input\":\"3640\",\
             \"best\":null,\"issues\":[{\"code\":\"REJECTED_BY_POLICY\",\
             \"severity\":\"error\",\"rank\":90,\"ref_text\":\"3640\"}]}}]"
        );
        // An unreadable tag is refused rather than run unrestricted.
        assert_eq!(
            parse_dimensions_for_editor_json_with_context("幅3640", "ja", "lenght", "forgiving"),
            "[{\"start\":0,\"end\":7,\"byteStart\":0,\"byteEnd\":7,\"charStart\":0,\
             \"charEnd\":5,\"text\":\"幅3640\",\"parsed\":{\"ok\":false,\"input\":\"幅3640\",\
             \"best\":null,\"issues\":[{\"code\":\"REJECTED_BY_POLICY\",\
             \"severity\":\"error\",\"rank\":90,\"ref_text\":\"幅3640\"}]}}]"
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
    /// see the loss. Every endpoint value and unit must survive, and it must
    /// survive on the endpoint it belongs to: the whole `range` object is
    /// compared as one ordered fragment, because checking for the two values
    /// separately passes just as happily when `from` and `to` are swapped.
    #[test]
    fn parse_json_carries_both_range_endpoints() {
        // input, from-value, to-value, unit, dimension
        let cases: [(&str, &str, &str, &str, &str); 4] = [
            ("10 ± 0.5 mm", "0.0095", "0.0105", "m", "length"),
            ("100-120㎡", "100", "120", "m2", "area"),
            ("between 5 and 10 kg", "5", "10", "kg", "mass"),
            ("2〜3日", "172800", "259200", "s", "time"),
        ];
        for (input, from, to, unit, dimension) in cases {
            let json = parse_json_with_context(input, "", dimension, "");
            assert!(is_valid_json(&json), "{input}: {json}");
            assert!(json.contains("\"kind\":\"range\""), "{input}: {json}");
            // The endpoints are full readings, not a bare pair of numbers, and
            // the lower bound is `from`.
            let expected = format!(
                "\"range\":{{\"from\":{{\"kind\":\"quantity\",\"value\":{from},\"unit\":\"{unit}\",\
                 \"dimension\":\"{dimension}\"}},\"to\":{{\"kind\":\"quantity\",\"value\":{to},\
                 \"unit\":\"{unit}\",\"dimension\":\"{dimension}\"}}}}"
            );
            assert!(json.contains(&expected), "{input}: {json}\nwant {expected}");
            // Both endpoints carry the unit, not just the container.
            assert_eq!(
                json.matches(&format!("\"unit\":\"{unit}\"")).count(),
                2,
                "{input}: {json}"
            );
        }
    }

    /// The envelope used to run every value through `format_number`, which
    /// rounds to six decimals — a real quantity below that threshold became
    /// exactly `0` with no finding at all.
    #[test]
    fn parse_json_keeps_values_below_the_old_rounding_threshold() {
        let json = parse_json("0.0000001 m");
        assert!(is_valid_json(&json), "{json}");
        // The whole envelope, not a substring of the number: `contains` on a
        // numeric token matches any *longer* number that starts with the same
        // digits, so `"value":0.0000001` would also pass against a value of
        // `0.00000012` — and `"value":0` against every value in this test.
        assert_eq!(
            json,
            "{\"ok\":true,\"input\":\"0.0000001 m\",\"best\":{\"kind\":\"quantity\",\
             \"value\":0.0000001,\"unit\":\"m\",\"dimension\":\"length\"},\"issues\":[]}"
        );
        // Plain decimal notation only: `1e-7` still parses in Rust but is the
        // shape a strict JSON consumer is most likely to trip over.
        assert!(!json.contains("e-"), "{json}");
        assert_eq!(
            parse("0.0000001 m", None).best.and_then(|best| best.value),
            Some(0.0000001)
        );
    }

    /// `2 cups` holds 0.473176473 L; the envelope used to ship 0.473176.
    ///
    /// The value assertions are delimiter-terminated for the same reason the
    /// test above compares the whole envelope: `contains("\"value\":0.473176")`
    /// is a prefix of the true value and passes against the old rounding.
    #[test]
    fn parse_json_serializes_cups_at_full_precision() {
        let json = parse_json_with_context("2 cups", "en-US", "volume", "");
        assert!(is_valid_json(&json), "{json}");
        assert!(
            json.contains("\"value\":0.473176473,\"unit\":\"L\""),
            "{json}"
        );
        assert!(!json.contains("\"value\":0.473176,"), "{json}");
        let best = Parser::new(Dimension::Volume.into())
            .parse("2 cups")
            .best
            .expect("a reading");
        assert!(
            json.contains(&format!("\"value\":{},", best.value.expect("a value"))),
            "{json}"
        );
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
        assert!(schema.contains("applied before grammar dispatch"));
        assert!(schema.contains("independent of expected_dimensions"));
    }

    /// The published input schema must accept exactly what the code accepts.
    ///
    /// A substring assertion is what let the schema keep declaring the old
    /// singular `expected_dimension` with an `enum` of single names, which
    /// rejects the `"length,area"` that `docs/wasm.md` and the TypeScript
    /// declarations tell callers to send. A validator applying the published
    /// contract would have refused the documented value.
    #[test]
    fn input_schema_declares_the_dimension_set_the_parser_reads() {
        let schema = parse_input_schema_json();
        assert!(schema.contains("\"expected_dimensions\": {"), "{schema}");
        assert!(schema.contains("\"registry_dimensions\": {"), "{schema}");
        // The singular property, and the `enum` that forbade the list form, are
        // gone rather than merely joined by a second property.
        assert!(!schema.contains("\"expected_dimension\": {"), "{schema}");
        assert!(
            schema.contains("comma-separated list such as \\\"length,area\\\""),
            "{schema}"
        );

        // Every dimension is offered by the schema.
        for dimension in ALL_DIMENSIONS {
            let name = dimension.as_str();
            assert!(
                schema.contains(&format!("{name}|")) || schema.contains(&format!("|{name})")),
                "{name}"
            );
        }
        // And the shape it declares admits a list, not just one name.
        assert!(schema.contains("( *, *("), "{schema}");
    }
}
