//! Scientific notation, which technical documents write and this could not read.
//!
//! `1.2e5` was unread even as a bare number: every numeric path stopped at the
//! `e`, and the splitter that separates a number from its unit handed `1.2` to
//! one and `e5 Pa` to the other. Both had to learn the same shape, which is why
//! there are tests here for the number alone and the number with a unit.

mod support;
use support::parse;

use unravel_nl::{Dimension, NumberFormat, ParseCtx};

fn value_of(input: &str) -> Option<f64> {
    parse(input, None).best.and_then(|best| best.value)
}

#[test]
fn a_bare_exponent_is_one_number() {
    for (input, expected) in [
        ("1.2e5", 120_000.0),
        ("1.2E5", 120_000.0),
        ("1.2e+5", 120_000.0),
        ("2.5e-3", 0.0025),
        ("1e3", 1000.0),
        ("-1.2e5", -120_000.0),
        ("6.02e23", 6.02e23),
        ("1e0", 1.0),
        ("0e5", 0.0),
    ] {
        assert_eq!(value_of(input), Some(expected), "{input:?}");
    }
}

#[test]
fn an_exponent_keeps_its_unit() {
    for (input, expected, unit, dimension) in [
        ("1.2e5 Pa", 120_000.0, "Pa", Dimension::Pressure),
        ("2.5e-3 ohm", 0.0025, "Ω", Dimension::Resistance),
        ("1e3 mm", 1.0, "m", Dimension::Length),
        ("2.5e-3 m", 0.0025, "m", Dimension::Length),
        ("1.5e2 kN", 150_000.0, "N", Dimension::Force),
    ] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_eq!(best.value, Some(expected), "{input:?}");
        assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
        assert_eq!(best.dimension, Some(dimension), "{input:?}");
    }
}

/// The exponent is reassembled and parsed as one literal, not multiplied out.
///
/// `1e308` is exactly representable; `mantissa * 10f64.powi(308)` is not, and
/// reads it as 1.0000000000000006e308. Getting this wrong invents digits the
/// document did not write, which is the failure this library exists to avoid.
#[test]
fn an_exponent_is_rounded_once_not_multiplied_out() {
    assert_eq!(value_of("1e308"), Some(1e308));
    assert_eq!(value_of("-1e308"), Some(-1e308));
    assert_eq!(value_of("1.7976931348623157e308"), Some(f64::MAX));
    assert_eq!(value_of("5e-324"), Some(5e-324));
}

/// A number this type cannot hold is refused at both ends.
///
/// Overflow to infinity and underflow to zero are the same failure wearing
/// different clothes: both report a value the text does not state. `1e-400` is
/// not zero.
#[test]
fn a_number_too_large_or_too_small_to_hold_is_refused() {
    for input in [
        "1e400",
        "1e309",
        "-1e400",
        "1e-400",
        "1e-400 kg",
        "1e400 kg",
    ] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input:?} reported a value");
        assert!(
            !parsed.findings.skipped.is_empty(),
            "{input:?} was dropped with no finding"
        );
    }

    // A stated zero is still zero; only a non-zero mantissa collapsing to zero
    // is the failure.
    assert_eq!(value_of("0e-400"), Some(0.0));
}

/// The `e` has to sit between a digit and an exponent.
///
/// No unit in this registry starts with `e`, but that is a fact about the
/// registry, not a licence to treat any `e` as an exponent marker.
#[test]
fn a_stray_e_is_not_an_exponent() {
    for input in ["5e", "e5", "1.2e", "abc e5", "5 e5", "1.2e+", "1.2e-"] {
        assert_eq!(value_of(input), None, "{input:?} was read as an exponent");
    }
}

/// The mantissa still honours the caller's number format.
#[test]
fn the_mantissa_respects_the_declared_number_format() {
    let comma = ParseCtx {
        number_format: NumberFormat::CommaDecimal,
        ..ParseCtx::default()
    };
    assert_eq!(
        parse("1,2e5", Some(comma)).best.and_then(|best| best.value),
        Some(120_000.0)
    );

    let dot = ParseCtx {
        number_format: NumberFormat::DotDecimal,
        ..ParseCtx::default()
    };
    assert_eq!(
        parse("1.2e5", Some(dot)).best.and_then(|best| best.value),
        Some(120_000.0)
    );
}
