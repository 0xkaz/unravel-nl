use unravel_nl::{
    AcceptOptions, CompletionKind, CustomUnit, Dimension, FuzzyProfile, FuzzyTerm, IssueCode, Kind,
    NumberFormat, ParseCtx, complete_readings, describe_parsed, describe_reading, parse,
};

#[test]
fn explicit_number_format_policy_resolves_decimal_ambiguity() {
    let comma = parse(
        "1,234 kg",
        Some(ParseCtx {
            number_format: NumberFormat::CommaDecimal,
            ..ParseCtx::default()
        }),
    );
    let comma_best = comma.best.expect("comma decimal");
    assert_eq!(comma_best.unit.as_deref(), Some("kg"));
    assert_close(comma_best.value.expect("value"), 1.234);
    assert!(comma.findings.ambiguities.is_empty());

    let dot = parse(
        "1,234 kg",
        Some(ParseCtx {
            number_format: NumberFormat::DotDecimal,
            ..ParseCtx::default()
        }),
    );
    let dot_best = dot.best.expect("dot decimal");
    assert_close(dot_best.value.expect("value"), 1234.0);
}

#[test]
fn completion_readings_fan_out_plain_numbers_to_expected_units() {
    let completions = complete_readings(
        "10",
        Some(ParseCtx {
            expected_dimension: Some(Dimension::Mass),
            ..ParseCtx::default()
        }),
    );

    assert!(completions.iter().any(|item| {
        item.reason == "best"
            && item.reading.kind == Kind::Number
            && item.reading.value == Some(10.0)
    }));
    assert!(completions.iter().any(|item| {
        item.reason == "unit_fanout"
            && item.reading.dimension == Some(Dimension::Mass)
            && item.text == "10 kg"
    }));
}

#[test]
fn acceptance_controls_reject_shapes_but_keep_candidates() {
    let parsed = parse(
        "between 5 and 10 kg",
        Some(ParseCtx {
            accept: AcceptOptions {
                ranges: false,
                ..AcceptOptions::default()
            },
            ..ParseCtx::default()
        }),
    );

    assert!(parsed.best.is_none());
    assert_eq!(parsed.alternatives.len(), 1);
    assert_eq!(parsed.alternatives[0].kind, Kind::Range);
    assert_eq!(parsed.findings.skipped[0].code, IssueCode::RejectedByPolicy);

    let conversion = parse(
        "72 in to cm",
        Some(ParseCtx {
            accept: AcceptOptions {
                conversions: false,
                ..AcceptOptions::default()
            },
            ..ParseCtx::default()
        }),
    );
    assert!(conversion.best.is_none());
    assert_eq!(conversion.alternatives[0].unit.as_deref(), Some("cm"));
}

#[test]
fn custom_units_can_carry_custom_kind_metadata() {
    let parsed = parse(
        "3 cases",
        Some(ParseCtx {
            custom_units: vec![
                CustomUnit::new("case", "item", &["case", "cases"], Dimension::Volume, 24.0)
                    .kind("package_count"),
            ],
            ..ParseCtx::default()
        }),
    );

    let best = parsed.best.expect("custom kind");
    assert_eq!(best.custom_kind.as_deref(), Some("package_count"));
    assert_eq!(best.unit.as_deref(), Some("item"));
    assert_close(best.value.expect("value"), 72.0);
}

#[test]
fn custom_fuzzy_profile_normalizes_terms_to_ranges() {
    let parsed = parse(
        "heavy",
        Some(ParseCtx {
            expected_dimension: Some(Dimension::Mass),
            fuzzy_profiles: vec![FuzzyProfile::new(
                "parcels",
                Dimension::Mass,
                "kg",
                &[FuzzyTerm::new("heavy", 20.0, 70.0)],
            )],
            ..ParseCtx::default()
        }),
    );

    let best = parsed.best.expect("fuzzy profile");
    assert_eq!(best.kind, Kind::Range);
    let range = best.range.expect("range");
    assert_close(range.from.value.expect("from"), 20.0);
    assert_close(range.to.value.expect("to"), 70.0);
}

#[test]
fn describe_views_expose_stable_resource_fields() {
    let parsed = parse("5 kg", None);
    let parsed_view = describe_parsed(&parsed);
    assert_eq!(parsed_view.object, "unravel.parsed");
    assert!(parsed_view.fields.iter().any(|field| field.name == "ok"));

    let reading_view = describe_reading(parsed.best.as_ref().expect("best"));
    assert_eq!(reading_view.object, "unravel.quantity");
    assert!(reading_view.summary.contains("kg"));
    assert!(
        reading_view
            .fields
            .iter()
            .any(|field| { field.name == "dimension" && field.value == Dimension::Mass.as_str() })
    );
}

#[test]
fn prefix_completion_api_remains_available() {
    let completions = unravel_nl::complete("10 met", None);
    assert_eq!(completions[0].kind, CompletionKind::Unit);
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}
