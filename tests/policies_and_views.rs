use unravel_nl::{
    AcceptOptions, CompletionKind, CustomUnit, Dimension, DimensionSet, FuzzyProfile, FuzzyTerm,
    IssueCode, Kind, NumberFormat, ParseCtx, complete_readings, describe_parsed, describe_reading,
    parse,
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
            expected_dimensions: DimensionSet::from(Dimension::Mass),
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

/// `complete_readings` feeds the same picker `complete` does, and carries the
/// same three structural promises — none of which were pinned before.
///
/// 1. At most 24 candidates, the cap the doc comment states. It is reachable:
///    a bare number with enough custom units fans out past it.
/// 2. Ordered by score, highest first, since the picker shows the list as-is.
/// 3. The built-in unit fan-out is bounded at 12 candidates, so a bare number
///    cannot crowd the parse-derived readings out of the capped list.
#[test]
fn completion_readings_are_capped_ordered_and_bounded() {
    // Enough custom units to overflow the cap: 12 built-in fan-out candidates
    // plus 40 custom ones plus the parsed reading is well past 24.
    let many_units: Vec<CustomUnit> = (0..40)
        .map(|index| {
            CustomUnit::new(
                &format!("cu{index}"),
                "m",
                &[],
                Dimension::Length,
                1.0 + f64::from(index),
            )
        })
        .collect();
    let capped = complete_readings(
        "10",
        Some(ParseCtx {
            custom_units: many_units,
            ..ParseCtx::default()
        }),
    );
    assert_eq!(
        capped.len(),
        24,
        "the documented cap is both an upper bound and reachable: {capped:?}"
    );

    for text in [
        "10",
        "5 kg",
        "2 cups",
        "",
        "next friday",
        "between 5 and 10 kg",
    ] {
        let completions = complete_readings(text, None);
        assert!(
            completions.len() <= 24,
            "{text:?}: {} candidates exceeds the documented cap of 24",
            completions.len()
        );
        for pair in completions.windows(2) {
            assert!(
                pair[0].score >= pair[1].score,
                "{text:?}: candidates are not ordered by score, {} before {}",
                pair[0].score,
                pair[1].score
            );
        }
        assert!(
            completions
                .iter()
                .filter(|item| item.reason == "unit_fanout")
                .count()
                <= 12,
            "{text:?}: the built-in unit fan-out is bounded at 12: {completions:?}"
        );
    }

    // The fan-out bound is reached, not merely respected: a bare number with no
    // expected dimension offers the full 12 built-in units.
    assert_eq!(
        complete_readings("10", None)
            .iter()
            .filter(|item| item.reason == "unit_fanout")
            .count(),
        12,
    );
}

/// `REJECTED_BY_POLICY` names the two policies that emit it, and `Strictness`
/// is not one of them.
///
/// The variant's own documentation said "refused by the active `Strictness`
/// policy", which no path does: a strict refusal is reported under the code for
/// what it refused. Only `AcceptOptions` and `ParseCtx::expected_dimensions`
/// reach this code, and a caller that branches on it needs that to be true.
#[test]
fn rejected_by_policy_comes_from_acceptance_and_dimensions_not_strictness() {
    let rejection = |parsed: &unravel_nl::Parsed| {
        parsed
            .findings
            .skipped
            .iter()
            .map(|issue| issue.code)
            .collect::<Vec<_>>()
    };

    // Strictness refuses, but under the code for what it refused.
    let strict = |text: &str| {
        parse(
            text,
            Some(ParseCtx {
                strictness: unravel_nl::Strictness::Strict,
                ..ParseCtx::default()
            }),
        )
    };
    let approximate = strict("about 20kg");
    assert!(approximate.best.is_none());
    assert_eq!(rejection(&approximate), vec![IssueCode::Approximation]);
    let typo = strict("5 meterz");
    assert!(typo.best.is_none());
    assert_eq!(rejection(&typo), vec![IssueCode::TypoCorrected]);

    // The two policies that do emit it.
    let shape = parse(
        "between 5 and 10 kg",
        Some(ParseCtx {
            accept: AcceptOptions {
                ranges: false,
                ..AcceptOptions::default()
            },
            ..ParseCtx::default()
        }),
    );
    assert!(rejection(&shape).contains(&IssueCode::RejectedByPolicy));

    let domain = parse(
        "5 kg",
        Some(ParseCtx {
            expected_dimensions: DimensionSet::from(Dimension::Length),
            ..ParseCtx::default()
        }),
    );
    assert!(rejection(&domain).contains(&IssueCode::RejectedByPolicy));
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
            expected_dimensions: DimensionSet::from(Dimension::Mass),
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
fn described_ok_uses_the_public_acceptance_rule() {
    let parsed = parse(
        "2 cups",
        Some(ParseCtx {
            strictness: unravel_nl::Strictness::Confirm,
            ..ParseCtx::default()
        }),
    );
    assert!(parsed.best.is_some());
    let view = describe_parsed(&parsed);
    assert!(!unravel_nl::accepts(&parsed));
    assert!(
        view.fields
            .iter()
            .any(|field| field.name == "ok" && field.value == "false")
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
