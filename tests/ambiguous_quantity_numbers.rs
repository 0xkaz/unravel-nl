//! Two readings that used to be settled in silence.
//!
//! 1. A grouping-ambiguous number kept its ambiguity when bare (`1.234`) and
//!    lost it the moment a unit was attached (`1.234 kg`), so a strict caller
//!    accepted a guess at a factor of a thousand.
//! 2. `5 m3` read as a volume through [`parse`] and as 5 m + 3 cm through
//!    [`parse_quantity_fast`], because the compound-height parser splits on the
//!    first `m` and runs before the registry lookup in the fast path.

mod support;
use support::{parse, parse_number_fast, parse_quantity_fast};

use unravel_nl::{
    CanonicalizeRequest, Dimension, IssueCode, Kind, NumberFormat, ParseCtx, Parsed, Parser,
    Strictness, canonicalize_values,
};

fn ambiguous_number_findings(parsed: &Parsed) -> Vec<(&str, usize, usize)> {
    parsed
        .findings
        .ambiguities
        .iter()
        .filter(|issue| issue.code == IssueCode::AmbiguousNumber)
        .map(|issue| (issue.ref_text.as_str(), issue.span.start, issue.span.end))
        .collect()
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{label}: expected {expected}, got {actual}"
    );
}

/// The bare-number behaviour this fix had to match, kept here as the reference.
#[test]
fn bare_ambiguous_number_still_reports_both_readings() {
    let parsed = parse("1.234", None);
    assert_close(parsed.best.as_ref().unwrap().value.unwrap(), 1.234, "1.234");
    assert_eq!(parsed.alternatives.len(), 1);
    assert_close(parsed.alternatives[0].value.unwrap(), 1234.0, "1.234 alt");
    assert_eq!(ambiguous_number_findings(&parsed), vec![("1.234", 0, 5)]);

    assert_eq!(parse_number_fast("1.234", None).alternatives.len(), 1);
}

/// A unit answers which quantity is meant, not which number was written.
#[test]
fn a_unit_does_not_settle_the_grouping_question() {
    for (input, best_value, alternative_value, unit, dimension) in [
        ("1.234 kg", 1.234, 1234.0, "kg", Dimension::Mass),
        ("1,234 kg", 1234.0, 1.234, "kg", Dimension::Mass),
        ("1.234 hours", 4442.4, 4_442_400.0, "s", Dimension::Time),
        ("1.234 USD", 1.234, 1234.0, "USD", Dimension::Currency),
        ("1.234 m", 1.234, 1234.0, "m", Dimension::Length),
        ("12.345 kg", 12.345, 12345.0, "kg", Dimension::Mass),
        ("0.123 kg", 0.123, 123.0, "kg", Dimension::Mass),
        // The conversion is affine, so the competing reading is re-parsed
        // rather than scaled: 1234 °F is 667.78 °C, not 1000 × −17.09.
        (
            "1.234 F",
            -17.092_222_222_222_22,
            667.777_777_777_777_8,
            "C",
            Dimension::Temperature,
        ),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input}"));
            assert_close(best.value.unwrap(), best_value, input);
            assert_eq!(best.unit.as_deref(), Some(unit), "{input}");
            assert_eq!(best.dimension, Some(dimension), "{input}");

            assert_eq!(parsed.alternatives.len(), 1, "{input}: {parsed:#?}");
            let alternative = &parsed.alternatives[0];
            assert_close(alternative.value.unwrap(), alternative_value, input);
            assert_eq!(alternative.unit.as_deref(), Some(unit), "{input}");
            assert_eq!(alternative.dimension, Some(dimension), "{input}");
            assert!(
                alternative.confidence.unwrap() < best.confidence.unwrap(),
                "{input}: the competing reading must not outrank the chosen one"
            );

            let findings = ambiguous_number_findings(&parsed);
            assert_eq!(findings.len(), 1, "{input}: {parsed:#?}");
            let number_text = input.split(' ').next().expect("number token");
            assert_eq!(findings[0].0, number_text, "{input}");
            assert_eq!(
                (findings[0].1, findings[0].2),
                (0, number_text.len()),
                "{input}"
            );
        }
    }
}

/// The finding points at the number, not at the whole input.
#[test]
fn the_finding_spans_the_number_inside_the_quantity() {
    assert_eq!(
        ambiguous_number_findings(&parse("1.234 kg", None)),
        vec![("1.234", 0, 5)]
    );
    // A currency symbol shifts the number, and the span moves with it.
    assert_eq!(
        ambiguous_number_findings(&parse("¥1,234", None)),
        vec![("1,234", 2, 7)]
    );
}

/// Every ambiguous shape, against a spread of grammars that read a number.
#[test]
fn every_ambiguous_shape_keeps_its_finding_under_every_unit() {
    for number in [
        "1.234", "12.345", "123.456", "0.123", "1,234", "12,345", "0,123",
    ] {
        for unit in ["kg", "g", "m", "cm", "mm", "L", "m2", "m3", "hours", "USD"] {
            let input = format!("{number} {unit}");
            for parsed in [parse(&input, None), parse_quantity_fast(&input, None)] {
                assert!(parsed.best.is_some(), "{input}: {parsed:#?}");
                assert_eq!(parsed.alternatives.len(), 1, "{input}: {parsed:#?}");
                assert_eq!(
                    ambiguous_number_findings(&parsed).len(),
                    1,
                    "{input}: {parsed:#?}"
                );
            }
        }
    }
}

/// Shapes the input itself settles stay silent, unit or no unit.
#[test]
fn settled_shapes_report_nothing_with_a_unit() {
    for input in [
        "1.23 kg",
        "1.2345 kg",
        "1234.567 kg",
        "1.234,56 kg",
        "1,234.56 kg",
        "1.234.567 kg",
        "5 kg",
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            assert!(parsed.best.is_some(), "{input}: {parsed:#?}");
            assert!(
                ambiguous_number_findings(&parsed).is_empty(),
                "{input}: {parsed:#?}"
            );
            assert!(parsed.alternatives.is_empty(), "{input}: {parsed:#?}");
        }
    }
}

/// A declared format is the caller answering the question; nothing is left to
/// report, for quantities exactly as for bare numbers.
///
/// Only [`parse`] is checked for the resulting *value*: the fallback grammar
/// `parse_quantity_fast` reaches for `1.234 kg` does not consult the declared
/// format, an asymmetry [`NumberFormat::CommaDecimal`] already documents. What
/// matters here is that neither entry point reports an ambiguity once the
/// caller has declared a format.
#[test]
fn a_declared_number_format_settles_the_quantity_silently() {
    for (format, expected) in [
        (NumberFormat::DotDecimal, 1.234),
        (NumberFormat::CommaDecimal, 1234.0),
    ] {
        let ctx = ParseCtx {
            number_format: format,
            ..ParseCtx::default()
        };
        let general = parse("1.234 kg", Some(ctx.clone()));
        assert_close(
            general.best.as_ref().unwrap().value.unwrap(),
            expected,
            "1.234 kg",
        );

        for parsed in [general, parse_quantity_fast("1.234 kg", Some(ctx.clone()))] {
            assert!(parsed.alternatives.is_empty(), "{format:?}: {parsed:#?}");
            assert!(
                ambiguous_number_findings(&parsed).is_empty(),
                "{format:?}: {parsed:#?}"
            );
        }
    }
}

/// Each range endpoint is asked separately, and named separately.
///
/// The range itself is not duplicated: enumerating both readings of both ends
/// would produce four candidate ranges, most of which nobody wrote. This
/// matches how an ambiguous *date* endpoint is already reported.
#[test]
fn range_endpoints_report_their_own_ambiguous_numbers() {
    for input in ["1.234-2.345 kg", "1.234-2.345"] {
        let parsed = parse(input, None);
        let range = parsed
            .best
            .as_ref()
            .and_then(|best| best.range.clone())
            .unwrap_or_else(|| panic!("{input}: no range"));
        assert_close(range.from.value.unwrap(), 1.234, input);
        assert_close(range.to.value.unwrap(), 2.345, input);
        assert_eq!(
            ambiguous_number_findings(&parsed),
            vec![("1.234", 0, 5), ("2.345", 6, 11)],
            "{input}"
        );
    }

    // Endpoints written identically get one finding each, and each finding
    // spans the endpoint it names. Locating an endpoint by the first occurrence
    // of its text put both findings on the left one and left the right end
    // unaddressed.
    let expectations = [
        ("1.234-1.234", vec![("1.234", 0, 5), ("1.234", 6, 11)]),
        ("1.234-1.234 kg", vec![("1.234", 0, 5), ("1.234", 6, 11)]),
        ("1.234 to 1.234", vec![("1.234", 0, 5), ("1.234", 9, 14)]),
        (
            "between 1.234 and 1.234",
            vec![("1.234", 8, 13), ("1.234", 18, 23)],
        ),
        (
            "from 1.234 to 1.234",
            vec![("1.234", 5, 10), ("1.234", 14, 19)],
        ),
        ("1.234〜1.234", vec![("1.234", 0, 5), ("1.234", 8, 13)]),
        ("1.234..1.234", vec![("1.234", 0, 5), ("1.234", 7, 12)]),
    ];
    for (input, expected) in expectations {
        let parsed = parse(input, None);
        let range = parsed
            .best
            .as_ref()
            .and_then(|best| best.range.clone())
            .unwrap_or_else(|| panic!("{input}: no range"));
        assert_close(range.from.value.unwrap(), 1.234, input);
        assert_close(range.to.value.unwrap(), 1.234, input);
        let findings = ambiguous_number_findings(&parsed);
        assert_eq!(findings, expected, "{input}");
        // Every span really covers the text it claims, and no two findings
        // point at the same fragment.
        for (text, start, end) in &findings {
            assert_eq!(&&input[*start..*end], text, "{input}");
        }
        assert_ne!(findings[0].1, findings[1].1, "{input}");
    }

    // Endpoints the shape settles report nothing.
    let settled = parse("1.5-2.5 kg", None);
    assert!(settled.best.is_some(), "{settled:#?}");
    assert!(
        ambiguous_number_findings(&settled).is_empty(),
        "{settled:#?}"
    );
}

/// The point of the strict validator: refuse what the parser had to guess at.
#[test]
fn strict_canonicalization_refuses_a_guessed_grouping() {
    let ctx = ParseCtx {
        strictness: Strictness::Strict,
        ..ParseCtx::default()
    };
    let results = canonicalize_values(&[
        CanonicalizeRequest::new(
            "weight",
            "1.234 kg",
            Parser::with_context(Dimension::Mass.into(), ctx.clone()),
        ),
        CanonicalizeRequest::new(
            "clear",
            "1.5 kg",
            Parser::with_context(Dimension::Mass.into(), ctx.clone()),
        ),
    ]);

    assert!(!results[0].ok, "{:#?}", results[0]);
    assert!(results[0].canonical.is_none());
    assert!(
        results[0]
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("AMBIGUOUS_NUMBER"),
        "{:#?}",
        results[0]
    );

    assert!(results[1].ok, "{:#?}", results[1]);
    assert_close(
        results[1].canonical.as_ref().unwrap().value.unwrap(),
        1.5,
        "1.5 kg",
    );
}

/// `5 m3` is a volume through every door.
///
/// The compound-height idiom is written closed up (`1m80`), so a space before a
/// token the registry knows is not that idiom. There is no competing reading to
/// surface here: nobody writes 5 m + 3 cm as `5 m3`.
#[test]
fn a_spaced_registry_unit_is_not_a_compound_height() {
    for (input, value, unit, dimension) in [
        ("5 m3", 5000.0, "L", Dimension::Volume),
        ("5 ft2", 0.464_515_2, "m2", Dimension::Area),
        ("5 m2", 5.0, "m2", Dimension::Area),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input}"));
            assert_eq!(best.kind, Kind::Quantity, "{input}");
            assert_close(best.value.unwrap(), value, input);
            assert_eq!(best.unit.as_deref(), Some(unit), "{input}");
            assert_eq!(best.dimension, Some(dimension), "{input}");
            assert!(parsed.alternatives.is_empty(), "{input}: {parsed:#?}");
        }
    }
}

/// The compound-height readings the README documents, unchanged.
#[test]
fn closed_up_and_multi_token_compounds_still_read_as_before() {
    for (input, value) in [
        ("1m80", 1.8),
        ("180cm", 1.8),
        ("180 cm", 1.8),
        ("5ft 11", 1.8034),
        ("5 ft 11", 1.8034),
        ("5 ft", 1.524),
        ("5 m", 5.0),
        ("3 yd 2 ft", 3.3528),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input}"));
            assert_close(best.value.unwrap(), value, input);
            assert_eq!(best.unit.as_deref(), Some("m"), "{input}");
            assert_eq!(best.dimension, Some(Dimension::Length), "{input}");
        }
    }

    let mass = parse("2 lb 3 oz", None).best.expect("compound mass");
    assert_close(mass.value.unwrap(), 0.992_233_309_375_000_1, "2 lb 3 oz");
}
