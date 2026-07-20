//! Semantic coverage for `parse_number_fast`.
//!
//! Everything that referenced this entry point before asserted only that it
//! agrees with `parse(purpose = Number)`, which passes just as well when both
//! are wrong. Its documented contract — "grouping and decimal separators are
//! read according to [`ParseCtx::number_format`] and [`ParseCtx::locale`]" —
//! had no test with any locale or any explicit `NumberFormat`.

use unravel_nl::{IssueCode, Kind, Locale, NumberFormat, ParseCtx, parse_number_fast};

fn with_format(input: &str, number_format: NumberFormat) -> unravel_nl::Parsed {
    parse_number_fast(
        input,
        Some(ParseCtx {
            number_format,
            ..ParseCtx::default()
        }),
    )
}

/// An explicit format is the caller settling the question, so there is one
/// reading and no ambiguity.
#[test]
fn dot_decimal_reads_the_dot_as_a_decimal_point() {
    let parsed = with_format("1.234", NumberFormat::DotDecimal);
    let best = parsed.best.as_ref().expect("best reading");
    assert_eq!(best.kind, Kind::Number);
    assert_eq!(best.value, Some(1.234));
    assert_eq!(best.unit, None);
    assert_eq!(best.dimension, None);
    assert!(parsed.alternatives.is_empty(), "{:?}", parsed.alternatives);
    assert!(
        parsed.findings.ambiguities.is_empty(),
        "{:?}",
        parsed.findings.ambiguities
    );

    // The mirror case: with the dot as the decimal point, a comma is grouping.
    let grouped = with_format("1,234", NumberFormat::DotDecimal);
    assert_eq!(grouped.best.and_then(|best| best.value), Some(1234.0));
}

/// A declared format answers consistently no matter how many separators the
/// input happens to contain.
///
/// With the comma declared as the decimal separator, the dot can only group
/// digits. This used to hold when both characters were present (`1.234,56`) and
/// when the dot repeated (`1.234.567`), but a *single* dot fell through to the
/// plain path and was read as a decimal point, so `1.234` came back as 1.234
/// instead of 1234 — a factor of a thousand, silently, for a caller who had
/// explicitly declared a European format.
#[test]
fn comma_decimal_treats_every_dot_as_grouping() {
    for (input, expected) in [
        ("1.234", 1_234.0),
        ("1.234.567", 1_234_567.0),
        ("1.23", 123.0),
        ("1.234,56", 1234.56),
    ] {
        assert_eq!(
            with_format(input, NumberFormat::CommaDecimal)
                .best
                .and_then(|best| best.value),
            Some(expected),
            "{input}"
        );
    }

    // The mirror image: DotDecimal treats every comma as grouping.
    for (input, expected) in [
        ("1,234", 1_234.0),
        ("1,234,567", 1_234_567.0),
        ("1,23", 123.0),
        ("1,234.56", 1234.56),
    ] {
        assert_eq!(
            with_format(input, NumberFormat::DotDecimal)
                .best
                .and_then(|best| best.value),
            Some(expected),
            "{input}"
        );
    }

    // What each format settles: its own decimal separator stays a decimal point.
    let parsed = with_format("1,234", NumberFormat::CommaDecimal);
    let best = parsed.best.as_ref().expect("best reading");
    assert_eq!(best.value, Some(1.234));
    assert!(parsed.alternatives.is_empty(), "{:?}", parsed.alternatives);
    assert!(
        parsed.findings.ambiguities.is_empty(),
        "{:?}",
        parsed.findings.ambiguities
    );
}

#[test]
fn auto_reports_the_grouping_ambiguity_with_an_alternative() {
    for (input, best_value, alternative, reason) in [
        ("1.234", 1.234, 1234.0, "Dot"),
        ("1,234", 1234.0, 1.234, "Comma"),
    ] {
        let parsed = with_format(input, NumberFormat::Auto);
        assert_eq!(
            parsed.best.as_ref().and_then(|best| best.value),
            Some(best_value),
            "{input}"
        );
        assert_eq!(parsed.alternatives.len(), 1, "{input}");
        assert_eq!(parsed.alternatives[0].value, Some(alternative), "{input}");

        assert_eq!(parsed.findings.ambiguities.len(), 1, "{input}");
        let ambiguity = &parsed.findings.ambiguities[0];
        assert_eq!(ambiguity.code, IssueCode::AmbiguousNumber, "{input}");
        assert_eq!(ambiguity.ref_text, input, "{input}");
        assert_eq!(ambiguity.candidate_count, Some(2), "{input}");
        assert!(ambiguity.reason.starts_with(reason), "{input}");
    }

    // Auto is the default, so the bare call reports the same ambiguity.
    assert_eq!(parse_number_fast("1.234", None).alternatives.len(), 1);
}

/// **Documented but not implemented.**
///
/// `parse_number_fast`'s doc comment says separators are read according to
/// `ParseCtx::number_format` *and* `ParseCtx::locale`, but the number path only
/// ever consults `number_format`: every locale below — including a
/// comma-decimal one — produces the identical reading. Pinned so that either
/// the doc or the behaviour has to change deliberately.
#[test]
fn locale_alone_does_not_change_the_separator_reading() {
    for locale in [
        None,
        Some(Locale::En),
        Some(Locale::EnUs),
        Some(Locale::EnGb),
        Some(Locale::Ja),
        Some(Locale::Other("de-DE".to_owned())),
    ] {
        let parsed = parse_number_fast(
            "1.234",
            Some(ParseCtx {
                locale: locale.clone(),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(
            parsed.best.as_ref().and_then(|best| best.value),
            Some(1.234),
            "{locale:?}"
        );
        assert_eq!(parsed.alternatives.len(), 1, "{locale:?}");
        assert_eq!(parsed.alternatives[0].value, Some(1234.0), "{locale:?}");
    }
}

/// The entry point exists so a numeric field cannot come back holding a
/// measurement, a currency, a date, or a range.
#[test]
fn never_attaches_a_unit() {
    for input in ["5 kg", "5kg", "20°C", "¥1,234", "next friday", "3-5"] {
        let parsed = parse_number_fast(input, None);
        assert!(parsed.best.is_none(), "{input}: {:?}", parsed.best);
        assert!(parsed.alternatives.is_empty(), "{input}");
        // Refused, not dropped: the input is reported.
        assert_eq!(parsed.findings.skipped.len(), 1, "{input}");
        assert_eq!(
            parsed.findings.skipped[0].code,
            IssueCode::NoValue,
            "{input}"
        );
        assert_eq!(parsed.findings.skipped[0].ref_text, input, "{input}");
    }

    // A number it does read never carries a unit or a dimension either.
    for input in ["5", "-3.5", "1,234", "½"] {
        for reading in parse_number_fast(input, None)
            .best
            .iter()
            .chain(parse_number_fast(input, None).alternatives.iter())
        {
            assert_eq!(reading.kind, Kind::Number, "{input}");
            assert_eq!(reading.unit, None, "{input}");
            assert_eq!(reading.dimension, None, "{input}");
        }
    }
}
