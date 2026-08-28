//! Temperature ranges, which read only when both ends carried the unit.
//!
//! `40 C to 85 C` parsed and `40 to 85 C` did not. A range borrows the right
//! endpoint's unit for a left endpoint that has none, and the borrow could not
//! see a temperature: temperature is a grammar here, not registry entries,
//! because an offset conversion is not a factor.

mod support;
use support::parse;

use unravel_nl::{Dimension, IssueCode};

fn range_of(input: &str) -> Option<(f64, f64, String)> {
    let best = parse(input, None).best?;
    let range = best.range.as_ref()?;
    Some((range.from.value?, range.to.value?, range.to.unit.clone()?))
}

fn close(actual: f64, expected: f64, input: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{input}: expected {expected}, got {actual}"
    );
}

/// The unit is borrowed from the right endpoint, temperature included.
#[test]
fn a_temperature_range_reads_with_the_unit_written_once() {
    for (input, from, to) in [
        ("40 to 85 C", 40.0, 85.0),
        ("20 to 30 C", 20.0, 30.0),
        ("40-85 C", 40.0, 85.0),
        ("40 to 85 °C", 40.0, 85.0),
        ("-40 to 85 C", -40.0, 85.0),
    ] {
        let (a, b, unit) = range_of(input).unwrap_or_else(|| panic!("{input:?} was not read"));
        close(a, from, input);
        close(b, to, input);
        assert_eq!(unit, "C", "{input:?}");
    }

    // Writing the unit on both ends still works, and agrees.
    assert_eq!(range_of("40 C to 85 C"), range_of("40 to 85 C"));
}

/// Each endpoint converts on its own, which is what makes K and F well defined.
#[test]
fn kelvin_and_fahrenheit_ranges_convert_each_end() {
    let (a, b, unit) = range_of("40 to 85 K").expect("a kelvin range");
    close(a, -233.15, "40 K");
    close(b, -188.15, "85 K");
    assert_eq!(unit, "C");

    let (a, b, _) = range_of("40 to 85 F").expect("a fahrenheit range");
    close(a, 4.444_444_444_444_445, "40 F");
    close(b, 29.444_444_444_444_443, "85 F");
}

/// A tolerance on a temperature is refused, because it is a difference.
///
/// `center ± delta` subtracts one absolute temperature from another and calls
/// the result a temperature. `-40 K ± 0.5` came out as -40.5 °C to -585.8 °C.
/// Celsius looked right only because it is the canonical unit and its offset is
/// zero, which is an accident rather than a rule — so the whole dimension is
/// refused rather than the one spelling that happens to survive.
#[test]
fn a_temperature_tolerance_is_refused_rather_than_computed() {
    for input in [
        "-40 K±0.5",
        "-40 C±0.5",
        "-40 F±0.5",
        "10 ± 0.5 C",
        "0.5 K±0.5",
    ] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input:?} computed a tolerance");
        assert!(
            !parsed.findings.skipped.is_empty(),
            "{input:?} was dropped with no finding"
        );
    }

    // Tolerances on every other dimension are untouched.
    for (input, from, to, unit) in [
        ("10 ± 0.5 mm", 0.0095, 0.0105, "m"),
        ("10 ± 0.5 kg", 9.5, 10.5, "kg"),
    ] {
        let (a, b, got) = range_of(input).unwrap_or_else(|| panic!("{input:?}"));
        close(a, from, input);
        close(b, to, input);
        assert_eq!(got, unit, "{input:?}");
    }
}

/// The unit lookup a range borrows from agrees with every other lookup.
///
/// `unit_suffix` read `unit.aliases` while the rest of the crate resolves a
/// unit by `unit_lookup_aliases`, which also carries the id. Exactly one unit
/// is written so that the difference shows — the coulomb, whose `C` is only an
/// id — and that was enough to lose the temperature range.
#[test]
fn a_range_can_borrow_a_unit_named_only_by_its_id() {
    let (from, to, unit) = range_of("40 to 85 C").expect("a range");
    assert_eq!((from, to, unit.as_str()), (40.0, 85.0, "C"));

    // The coulomb itself still reads, and still reports the collision.
    let coulomb = parse("7 C", None);
    assert_eq!(
        coulomb.best.as_ref().and_then(|best| best.dimension),
        Some(Dimension::Temperature)
    );
    assert!(
        coulomb
            .findings
            .ambiguities
            .iter()
            .any(|found| found.code == IssueCode::AmbiguousUnit)
    );
}
