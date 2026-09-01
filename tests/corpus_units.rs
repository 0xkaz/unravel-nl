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

/// Percent reads as a ratio, with the value the document wrote.
///
/// This reverses an earlier judgement here, and the reason it was wrong is
/// worth keeping. The argument for refusing was that a dimension asserts two
/// readings are the same kind of quantity, and 13.30% of an area and 74.68% of
/// a mass are not. But a dimension names the *kind* of quantity, never the
/// referent: `5 m` of pipe and `5 m` of cable are both lengths and nothing here
/// claimed otherwise. By that argument Length would have to go too.
///
/// The value stays as written — `10%` is ten percent, not 0.1 — because
/// converting to a fraction invites a caller to multiply it by whatever is to
/// hand, and what it is a fraction of is not in the reading.
#[test]
fn percent_reads_as_a_ratio_with_the_written_value() {
    for (input, expected) in [
        ("10%", 10.0),
        ("10 %", 10.0),
        ("13.30%", 13.3),
        ("-7.5%", -7.5),
        ("1.5%", 1.5),
        ("60%", 60.0),
    ] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_eq!(best.dimension, Some(Dimension::Ratio), "{input:?}");
        assert_eq!(best.unit.as_deref(), Some("%"), "{input:?}");
        close(best.value.expect("a value"), expected, input);
    }

    // A bound over a percentage reads like any other bound.
    let bounded = parse("≥ 25%", None).best.expect("a bound");
    let range = bounded.range.as_ref().expect("a range");
    close(range.from.value.expect("a value"), 25.0, "≥ 25%");
    assert_eq!(range.to.value, None);
}

/// A percentage that states its basis is not folded into a bare one.
///
/// `wt%` and `vol%` of the same material are different numbers, and recording
/// either as a plain `%` would drop the part that says which. They are refused
/// with the basis named, until there is somewhere to put it.
#[test]
fn a_percentage_with_a_stated_basis_is_not_flattened() {
    for (input, basis) in [
        ("7 wt%", "weight"),
        ("7 vol%", "volume"),
        ("7 at%", "atom count"),
    ] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input:?} was read as a bare ratio");
        assert!(
            parsed
                .findings
                .skipped
                .iter()
                .any(|found| found.code == IssueCode::UnknownUnit && found.reason.contains(basis)),
            "{input:?} did not name its basis"
        );
    }
}
