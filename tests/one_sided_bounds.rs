//! The edge cases of a one-sided bound, pinned by name.
//!
//! A bound states one endpoint and leaves the other unstated. The cases that
//! decide whether that is read honestly are not the ordinary ones: a sign with
//! no number behind it, endpoints that descend, a missing unit, full-width
//! punctuation, more than one range in a line, and values at the edge of what
//! a float holds.
//!
//! Some of these pin behaviour that is absent rather than correct. They are
//! written as characterization tests and labelled where that is so, because an
//! unread input that nobody has written down is indistinguishable from one
//! nobody has noticed — and because a later fix should have to change a test
//! on purpose rather than quietly turn one green.

mod support;
use support::parse;

use unravel_nl::{Dimension, DimensionSet, IssueCode, Parsed, Parser};

fn upper_bound_of(parsed: &Parsed) -> Option<(Option<f64>, f64, String)> {
    let range = parsed.best.as_ref()?.range.as_ref()?;
    Some((
        range.from.value,
        range.to.value?,
        range.to.unit.clone().unwrap_or_default(),
    ))
}

fn unread(parsed: &Parsed) -> bool {
    parsed.best.is_none() && parsed.alternatives.is_empty()
}

/// A sign with nothing behind it is not a number, and not a bound.
#[test]
fn a_bare_sign_reads_as_nothing() {
    for input in ["-", "+", "- kg", "+ kg", "≤ -", "under -"] {
        let parsed = parse(input, None);
        assert!(unread(&parsed), "{input:?} produced a reading");
        assert!(
            !parsed.findings.skipped.is_empty(),
            "{input:?} said nothing about why"
        );
    }
}

/// Endpoints that descend are reported, never quietly reordered.
///
/// Whether the author swapped them or meant them is undecidable, so the
/// reading keeps the order the text wrote and the surprise becomes a finding.
#[test]
fn descending_endpoints_are_reported_and_left_in_place() {
    for (input, from, to) in [
        ("40 to 12 kN", 40_000.0, 12_000.0),
        ("from 10kg to 2kg", 10.0, 2.0),
    ] {
        let parsed = parse(input, None);
        let range = parsed
            .best
            .as_ref()
            .expect("a range")
            .range
            .as_ref()
            .expect("endpoints");

        assert_eq!(
            range.from.value,
            Some(from),
            "{input:?} moved its endpoints"
        );
        assert_eq!(range.to.value, Some(to), "{input:?} moved its endpoints");
        assert!(
            parsed
                .findings
                .ambiguities
                .iter()
                .any(|found| found.code == IssueCode::AmbiguousNumber),
            "{input:?} descended without saying so"
        );
    }
}

/// A one-sided bound with no unit is not read as a bare number.
///
/// `≤ 40` could be forty of anything. Inferring a unit would be the same
/// mistake as inventing the lower endpoint, so nothing is returned.
#[test]
fn a_bound_without_a_unit_is_not_given_one() {
    for input in ["≤ 40", "< 40", "under 40", "up to 40"] {
        let parsed = parse(input, None);
        assert!(unread(&parsed), "{input:?} invented a reading");
    }

    // A two-ended range of bare numbers is dimensionless, which is a reading
    // the text does support — no unit is being supplied here.
    let plain = parse("12 to 37", None);
    let (from, to, unit) = upper_bound_of(&plain).expect("a dimensionless range");
    assert_eq!(from, Some(12.0));
    assert_eq!(to, 37.0);
    assert!(unit.is_empty(), "a bare range acquired the unit {unit:?}");
}

/// Full-width and CJK comparators reach the bound grammar through normalization.
///
/// `≦` (U+2266) is how Japanese technical writing ordinarily says "at most",
/// and this library already normalizes full-width digits and reads the `以下`
/// suffix — so reading `≤` but not `≦` was an inconsistency, not a policy. The
/// mapping lives in the normalization table with the other rewrites rather
/// than inside the bound grammar, so it is one declared rule.
#[test]
fn full_width_comparators_normalize_to_their_ascii_spellings() {
    for input in ["≦ 40 C", "<= 40 C", "≤ 40 C", "< 40 C", "＜40 C"] {
        let (from, to, unit) =
            upper_bound_of(&parse(input, None)).unwrap_or_else(|| panic!("{input:?} unread"));
        assert_eq!(from, None, "{input:?} invented a lower bound");
        assert!((to - 40.0).abs() < 1e-9, "{input:?} got {to}");
        assert_eq!(unit, "C", "{input:?}");
    }

    // The suffix form and a CJK compatibility unit still compose with it.
    let (_, to, unit) = upper_bound_of(&parse("２５．４ｍｍ以下", None)).expect("a bound");
    assert!((to - 0.0254).abs() < 1e-9, "got {to}");
    assert_eq!(unit, "m");

    let (_, to, unit) = upper_bound_of(&parse("≦40㎜", None)).expect("a bound");
    assert!((to - 0.04).abs() < 1e-9, "got {to}");
    assert_eq!(unit, "m");
}

/// Normalizing a comparator does not move the spans a caller highlights with.
///
/// This is the half that makes the rewrite safe to do at all: `＜` is three
/// bytes and `<` is one, so a span computed on the normalized text and handed
/// back unchanged would point into the middle of the original. Findings and
/// editor matches both have to address the untouched input.
#[test]
fn normalized_comparators_keep_spans_on_the_original_input() {
    // An unread full-width bound reports a span covering the original bytes.
    let parsed = parse("≧ 5 mm", None);
    let skipped = parsed.findings.skipped.first().expect("a finding");
    assert_eq!(skipped.span.start, 0);
    assert_eq!(skipped.span.end, "≧ 5 mm".len());
    assert_eq!(skipped.span.text, "≧ 5 mm");

    // An editor match inside a longer line addresses the original too.
    let line = "bore ２５．４ mm and wall 10 mm";
    let matches = Parser::unrestricted().parse_dimensions_for_editor(line);
    let first = matches.first().expect("a match");
    assert_eq!(&line[first.start..first.end], "２５．４ mm");
}

/// Lower-bound comparators stay unread, in every spelling.
///
/// There is no lower-bound grammar at all, so normalizing `≧` to `≥` maps the
/// character without making the input readable. Pinned as characterization:
/// the day a lower bound is read, this test should be the one that says so.
#[test]
fn lower_bound_comparators_are_still_unread() {
    for input in [
        "≧ 5 mm",
        "＞5mm",
        ">= 5 mm",
        "≥ 5 mm",
        "> 5 mm",
        "min 12 mm",
        "over 5 mm",
    ] {
        let parsed = parse(input, None);
        assert!(
            unread(&parsed),
            "{input:?} now reads — a lower bound grammar arrived, so update this test"
        );
        assert!(
            !parsed.findings.skipped.is_empty(),
            "{input:?} was dropped without a finding"
        );
    }
}

/// More than one range in a line is not read as one range.
///
/// This parser reads a whole input as a single value; picking one range out of
/// a line holding two would be choosing for the caller. Both are refused
/// together rather than the first being returned and the second dropped.
#[test]
fn several_ranges_in_one_input_are_refused_together() {
    for input in ["1-2 m and 3-4 m", "1-2 m, 3-4 m", "≤ 2 m and ≤ 4 m"] {
        let parsed = parse(input, None);
        assert!(unread(&parsed), "{input:?} picked one range out of several");
        assert!(
            !parsed.findings.skipped.is_empty(),
            "{input:?} dropped them silently"
        );
    }
}

/// Values at the edges, including the zero the bound no longer invents.
#[test]
fn extreme_and_zero_values_stay_exact() {
    for (input, value) in [("0 kg", 0.0), ("-0 kg", -0.0), ("-40 C", -40.0)] {
        let best = parse(input, None).best.expect(input);
        assert_eq!(best.value, Some(value), "{input:?}");
    }

    // A stated zero is a stated endpoint, and must not be confused with the
    // unstated one: `≤0 kg` says zero, `under 40 kg` says nothing.
    let stated = upper_bound_of(&parse("≤0 kg", None)).expect("a bound");
    assert_eq!(stated.0, None, "the open end is still open");
    assert_eq!(stated.1, 0.0, "the stated end is zero");

    // Exponent notation is not read at all, so an overflowing literal never
    // reaches the bound grammar. Pinned so the day it is read, this is seen.
    for input in ["1e308 kg", "1e400 kg", "under 1e308 kg"] {
        assert!(unread(&parse(input, None)), "{input:?} now reads");
    }
}

/// The two contracts are measured apart, never mixed.
///
/// The same input is a different question under a length-and-area parser than
/// under an unrestricted one, so neither answer is evidence about the other.
/// What must hold in both is that nothing outside the configured registry
/// leaks in — including as an alternative.
#[test]
fn a_scoped_parser_offers_nothing_from_outside_its_registry() {
    let building = Parser::japanese_building();
    let unrestricted = Parser::unrestricted();

    // Length and area are in scope for both, and agree.
    for input in ["25.4 mm", "10mm以下", "6帖"] {
        assert_eq!(
            building.parse(input).best.and_then(|best| best.dimension),
            unrestricted
                .parse(input)
                .best
                .and_then(|best| best.dimension),
            "{input:?}"
        );
    }

    // Out of scope for the building parser: refused, with nothing offered.
    for input in ["7 kN", "under 40 kg", "7 C", "5 H"] {
        let parsed = building.parse(input);
        assert!(parsed.best.is_none(), "{input:?} was read out of scope");
        assert!(
            parsed.alternatives.is_empty(),
            "{input:?} offered an out-of-registry alternative"
        );
        assert!(
            !parsed.findings.skipped.is_empty() || !parsed.findings.ambiguities.is_empty(),
            "{input:?} was refused silently"
        );
    }

    // A withheld symbol reports the same refusal under a scoped registry, and
    // still declines to name a reading the caller did not ask for.
    let scoped = Parser::new(DimensionSet::of(&[Dimension::Length])).parse("5 H");
    assert!(scoped.best.is_none());
    assert!(scoped.alternatives.is_empty());
}
