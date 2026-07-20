//! Semantic coverage for `parse_number_fast`.
//!
//! Everything that referenced this entry point before asserted only that it
//! agrees with `parse(purpose = Number)`, which passes just as well when both
//! are wrong. The tests here pin the separator contract itself: what each
//! explicit `NumberFormat` reads and refuses, when `Auto` reports an ambiguity,
//! that `ParseCtx::locale` does not enter into it, and that no reading this
//! entry point returns ever carries a unit.

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

    // Grouping the dot badly is not a number in this format, so it is refused
    // and reported rather than regrouped: `1.5` must not become 15, and
    // `1.2.3` must not become 123.
    for input in ["1.5", "1.23", "1.2.3", "1.0.0", ".5", "12.34.567"] {
        let parsed = with_format(input, NumberFormat::CommaDecimal);
        assert!(parsed.best.is_none(), "{input}: {:?}", parsed.best);
        assert!(!parsed.findings.skipped.is_empty(), "{input} lost silently");
    }

    // The mirror image: DotDecimal treats every comma as grouping, and applies
    // the same validation, including the Indian 2-2-3 shape.
    for (input, expected) in [
        ("1,234", 1_234.0),
        ("1,234,567", 1_234_567.0),
        ("12,34,567", 1_234_567.0),
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

    for input in ["1,5", "1,23", "1,2,3"] {
        let parsed = with_format(input, NumberFormat::DotDecimal);
        assert!(parsed.best.is_none(), "{input}: {:?}", parsed.best);
        assert!(!parsed.findings.skipped.is_empty(), "{input} lost silently");
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

/// A declared format wins over the shape of the input, including when both
/// separators are present and the input is written the *other* way round.
///
/// `1,234.56` is an anglophone number and `1.234,56` a European one, and
/// `NumberFormat::Auto` reads both by taking the rightmost separator as the
/// decimal point. A caller who declares a format is not asking for that
/// inference: under `CommaDecimal` the dot can only group, so `1,234.56` has a
/// comma where a group separator cannot sit and is refused — reported, not
/// silently regrouped into 1234.56. The mirror holds for `1.234,56` under
/// `DotDecimal`. Every other mixed-separator case in this file happens to be
/// one where the declared format and `Auto` agree, so without these two the
/// branch could ignore `number_format` entirely and still pass.
#[test]
fn a_declared_format_outranks_the_rightmost_separator_inference() {
    for (input, format) in [
        ("1,234.56", NumberFormat::CommaDecimal),
        ("1.234,56", NumberFormat::DotDecimal),
        ("1'234.56", NumberFormat::CommaDecimal),
        ("12,34.56", NumberFormat::CommaDecimal),
    ] {
        let parsed = with_format(input, format);
        assert!(
            parsed.best.is_none(),
            "{input} under {format:?} was read as {:?}",
            parsed.best.as_ref().and_then(|best| best.value)
        );
        assert!(parsed.alternatives.is_empty(), "{input} under {format:?}");
        // Refused, not dropped.
        assert_eq!(parsed.findings.skipped.len(), 1, "{input} under {format:?}");
        assert_eq!(
            parsed.findings.skipped[0].code,
            IssueCode::NoValue,
            "{input} under {format:?}"
        );
    }

    // `Auto` is the reading the declared formats above must *not* fall back to.
    for input in ["1,234.56", "1.234,56"] {
        assert_eq!(
            with_format(input, NumberFormat::Auto)
                .best
                .and_then(|best| best.value),
            Some(1234.56),
            "{input}"
        );
    }
}

#[test]
fn auto_reports_the_grouping_ambiguity_with_an_alternative() {
    // The reason is display prose, so the whole sentence is pinned rather than
    // its first word: `starts_with("Dot")` passed against any sentence that
    // merely began with the separator's name.
    for (input, best_value, alternative, reason) in [
        (
            "1.234",
            1.234,
            1234.0,
            "Dot can be read as a thousands separator or a decimal separator.",
        ),
        (
            "1,234",
            1234.0,
            1.234,
            "Comma can be read as a thousands separator or a decimal separator.",
        ),
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
        assert_eq!(ambiguity.reason, reason, "{input}");
    }

    // Auto is the default, so the bare call reports the same ambiguity.
    assert_eq!(parse_number_fast("1.234", None).alternatives.len(), 1);
}

/// The separator reading comes from `ParseCtx::number_format` alone.
///
/// `ParseCtx::locale` is not consulted by the number path: every locale below —
/// including a comma-decimal one — produces the identical reading and the
/// identical alternative for `1.234`. This is what `parse_number_fast` and
/// `NumberFormat::Auto` now document, and it is pinned here so that a change
/// to either the docs or the behaviour has to be deliberate.
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

    // A number it does read never carries a unit or a dimension either. The
    // expected reading count is spelled out so that an input which stopped
    // parsing fails here instead of vacuously satisfying an empty loop —
    // `1,234` is the grouping-ambiguous one, so it has an alternative.
    for (input, expected_readings) in [("5", 1), ("-3.5", 1), ("1,234", 2), ("½", 1)] {
        let parsed = parse_number_fast(input, None);
        let readings: Vec<_> = parsed
            .best
            .iter()
            .chain(parsed.alternatives.iter())
            .collect();
        assert_eq!(readings.len(), expected_readings, "{input}: {readings:?}");
        for reading in readings {
            assert_eq!(reading.kind, Kind::Number, "{input}");
            assert_eq!(reading.unit, None, "{input}");
            assert_eq!(reading.dimension, None, "{input}");
            assert!(reading.value.is_some(), "{input}");
        }
    }
}
