//! The units real technical documents write, and what this crate does with them.
//!
//! These spellings were taken from five research articles rather than chosen,
//! which is the point: a sweep of symbols someone thought of measures the
//! imagination of whoever wrote the list. Frequencies below are occurrences
//! after a number across those five documents.

mod support;
use support::parse;

use unravel_nl::{Dimension, IssueCode};

fn close(actual: f64, expected: f64, input: &str) {
    assert!(
        (actual - expected).abs() <= expected.abs().max(1.0) * 1e-9,
        "{input}: expected {expected:e}, got {actual:e}"
    );
}

/// The angstrom, ninety occurrences and no registry entry at all.
///
/// Exactly 1e-10 m by definition, so the conversion is exact although the unit
/// is not SI.
#[test]
fn the_angstrom_is_a_length() {
    for (input, expected) in [("7 Å", 7e-10), ("60 Å", 60e-10), ("5.19 Å", 5.19e-10)] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_eq!(best.dimension, Some(Dimension::Length), "{input:?}");
        close(best.value.expect("a value"), expected, input);
    }
}

/// Prefixes above giga. Graphene's stiffness is quoted in TPa.
#[test]
fn prefixes_above_giga_are_read() {
    for (input, expected, dimension) in [
        ("1 TPa", 1e12, Dimension::Pressure),
        ("7 TW", 7e12, Dimension::Power),
    ] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_eq!(best.dimension, Some(dimension), "{input:?}");
        close(best.value.expect("a value"), expected, input);
    }
}

/// Plane angle, one hundred and fifty-four occurrences.
///
/// SI calls this dimensionless, but `°` and `rad` convert into one another, so
/// a reading that carries the conversion says more than one that drops the unit
/// for being dimensionless.
#[test]
fn plane_angle_reads_and_converts() {
    let degrees = parse("90°", None).best.expect("an angle");
    assert_eq!(degrees.dimension, Some(Dimension::Angle));
    close(
        degrees.value.expect("a value"),
        core::f64::consts::PI / 2.0,
        "90°",
    );

    let radians = parse("7 rad", None).best.expect("an angle");
    assert_eq!(radians.dimension, Some(Dimension::Angle));
    close(radians.value.expect("a value"), 7.0, "7 rad");
}

/// Density, written the way a materials paper writes it.
#[test]
fn density_reads_in_the_spellings_documents_use() {
    for (input, expected) in [
        ("2.52 g/cm³", 2520.0),
        ("2.52 g/cm3", 2520.0),
        ("1.76 g/cm^3", 1760.0),
        ("1000 kg/m3", 1000.0),
    ] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_eq!(best.dimension, Some(Dimension::Density), "{input:?}");
        close(best.value.expect("a value"), expected, input);
    }
}

/// Energy is refused, because there is no Dimension for it.
///
/// It was not refused before: did-you-mean read `7 J/m²` as 7 m/s2 and `7 kJ`
/// as 7000 m, by edit distance to `m/s2` and `km`.
#[test]
fn energy_is_refused_rather_than_guessed() {
    for (input, quantity) in [
        ("7 J", "energy"),
        ("7 kJ", "energy"),
        ("148.0 J/m²", "energy per area"),
        ("7 W/m²", "irradiance"),
    ] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input:?} produced a reading");
        assert!(parsed.suggestions.is_empty(), "{input:?} suggested a unit");

        let found = parsed
            .findings
            .skipped
            .iter()
            .find(|found| found.code == IssueCode::UnknownUnit)
            .unwrap_or_else(|| panic!("{input:?} did not say why: {:?}", parsed.findings));
        assert!(
            found.reason.contains(quantity),
            "{input:?}: {}",
            found.reason
        );
    }
}

/// Percent is refused, and the refusal says what is missing.
///
/// The value is not in doubt — `10%` is ten percent — but a Dimension asserts
/// that two readings carrying it are the same kind of quantity, and two bare
/// percentages are not: 13.30% of an area and 74.68% of a mass share a sign and
/// nothing else. A reading with no Dimension is not available either, because
/// the registry is scoped by `DimensionSet`, so a dimensionless reading is one
/// no caller could admit or refuse. Refusing at least says why.
#[test]
fn percent_is_refused_because_the_basis_is_not_in_the_text() {
    for input in ["10%", "10 %", "13.30%", "-7.5%", "74.68%"] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input:?} produced a reading");

        let found = parsed
            .findings
            .skipped
            .iter()
            .find(|found| found.code == IssueCode::UnknownUnit)
            .unwrap_or_else(|| panic!("{input:?} did not say why"));
        assert!(
            found.reason.contains("ratio"),
            "{input:?}: {}",
            found.reason
        );
        // One cause, one finding.
        assert_eq!(parsed.findings.skipped.len(), 1, "{input:?}");
    }

    // A stated basis is named as such, and is still not commensurable with a
    // bare percentage, so it is refused too rather than folded in with it.
    for (input, basis) in [("7 wt%", "weight"), ("7 vol%", "volume")] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input:?}");
        assert!(
            parsed
                .findings
                .skipped
                .iter()
                .any(|found| found.reason.contains(basis)),
            "{input:?} did not name its basis"
        );
    }
}
