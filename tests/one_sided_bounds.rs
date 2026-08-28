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

use unravel_nl::{Dimension, DimensionSet, IssueCode, Parsed, Parser, humanize};

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

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
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
    // A refused full-width bound points at the marker, in the original bytes.
    // The unit is deliberately unknown (`ｘ`), so these stay unread whatever the
    // bound grammars can do, and the span is the only thing under test.
    for (input, marker) in [("≧ ５ ｘ", "≧"), ("２５．４ｘ以上", "以上")] {
        let parsed = parse(input, None);
        let skipped = parsed.findings.skipped.first().expect("a finding");
        assert_eq!(
            input.get(skipped.span.start..skipped.span.end),
            Some(marker),
            "{input:?} span {}..{} does not cover {marker:?} in the original",
            skipped.span.start,
            skipped.span.end
        );
        assert_eq!(skipped.span.text, marker, "{input:?}");
    }

    // An editor match inside a longer line addresses the original too.
    let line = "bore ２５．４ mm and wall 10 mm";
    let matches = Parser::unrestricted().parse_dimensions_for_editor(line);
    let first = matches.first().expect("a match");
    assert_eq!(&line[first.start..first.end], "２５．４ mm");
}

/// A lower bound reads as a bound, in every spelling.
///
/// This replaces the test that pinned these as unread. That one carried its own
/// instruction — "the day a lower bound is read, this test should be the one
/// that says so" — and it was: adding the grammar turned it red, which is what
/// a characterization test is for.
///
/// The shape mirrors the upper bound exactly. `≥ 5 mm` states the lower end and
/// leaves the upper unstated, where `≤ 5 mm` does the reverse, so a caller
/// handles one kind of answer rather than two.
#[test]
fn a_lower_bound_reads_with_its_upper_end_unstated() {
    for input in [
        "≥ 5 mm",
        "> 5 mm",
        ">= 5 mm",
        "≧ 5 mm",
        "＞5mm",
        "over 5 mm",
        "at least 5 mm",
        "more than 5 mm",
        "above 5 mm",
        "no less than 5 mm",
        "5 mm minimum",
        "5mm以上",
        "5mm超",
        "5mmを超える",
    ] {
        let parsed = parse(input, None);
        let best = parsed
            .best
            .as_ref()
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        let range = best.range.as_ref().expect("a range");

        assert_close(range.from.value.expect("the stated bound"), 0.005);
        assert_eq!(range.from.unit.as_deref(), Some("m"), "{input:?}");
        assert_eq!(range.to.value, None, "{input:?} invented an upper bound");
        assert_eq!(range.to.unit.as_deref(), Some("m"), "{input:?}");
    }
}

/// The two directions are mirror images, and both survive a round trip.
///
/// `humanize` renders the open end away rather than naming it, and the text it
/// produces is a spelling each grammar accepts, so the pair closes.
#[test]
fn both_bound_directions_round_trip_through_humanize() {
    for (input, rendered_as) in [
        ("≥ 5 mm", "at least 0.005 m"),
        ("5mm以上", "at least 0.005 m"),
        ("min 12 mm", "at least 0.012 m"),
        ("under 40 kg", "up to 40 kg"),
        ("40 C max", "up to 40 °C"),
    ] {
        let first = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?}"));
        let rendered = humanize(&first, None);
        assert_eq!(rendered, rendered_as, "{input:?}");

        let second = parse(&rendered, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} rendered as {rendered:?}, which did not parse"));
        let (a, b) = (
            first.range.as_ref().expect("a range"),
            second.range.as_ref().expect("a range"),
        );
        assert_eq!(a.from.value, b.from.value, "{input:?} -> {rendered:?}");
        assert_eq!(a.to.value, b.to.value, "{input:?} -> {rendered:?}");
        assert_eq!(a.from.unit, b.from.unit, "{input:?} -> {rendered:?}");
    }
}

/// English `max` and `min` are read on both sides.
///
/// Adding `min` and `minimum` for the lower bound while the upper side still
/// could not read `max 40 C` would have left this able to read one half of a
/// tolerance table. `max` is a suffix marker where `min` is not, because `min`
/// is also the minute.
#[test]
fn max_and_min_are_read_on_both_sides() {
    for (input, from, to) in [
        ("max 40 kg", None, Some(40.0)),
        ("max. 40 kg", None, Some(40.0)),
        ("40 kg max", None, Some(40.0)),
        ("40 kg maximum", None, Some(40.0)),
        ("no more than 40 kg", None, Some(40.0)),
        ("min 40 kg", Some(40.0), None),
        ("min. 40 kg", Some(40.0), None),
        ("40 kg minimum", Some(40.0), None),
        ("no less than 40 kg", Some(40.0), None),
    ] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        let range = best.range.as_ref().expect("a range");
        assert_eq!(range.from.value, from, "{input:?}");
        assert_eq!(range.to.value, to, "{input:?}");
    }
}

/// A lower bound the grammar cannot reach is still refused by name.
///
/// `5mm以上60mm以下` used to be the example here, and is now read by
/// `Grammar::TwoSidedBoundRange`. What is left is text where the bound is
/// stated plainly and the *unit* is the part nothing can resolve — and there
/// the refusal names the marker rather than saying only that nothing matched,
/// which is the difference between "cannot parse this" and "does not implement
/// this".
#[test]
fn an_unreachable_lower_bound_is_refused_by_name() {
    for (input, marker) in [("２５．４ｘ以上", "以上"), ("5ｘ以上", "以上")] {
        let parsed = parse(input, None);
        assert!(unread(&parsed), "{input:?} produced a reading");
        assert!(
            parsed.suggestions.is_empty(),
            "{input:?} still suggests a spelling correction"
        );
        assert!(
            parsed
                .findings
                .skipped
                .iter()
                .any(|found| found.reason.contains(marker)),
            "{input:?} did not name the bound: {:?}",
            parsed.findings.skipped
        );
    }
}

/// A bound marker is never a misspelling of a unit.
///
/// `5mm以上` came back as an area of 5e-6 m2 once, because `mm以上` is a short
/// hop from a square millimetre by edit distance. It reads as a bound now, so
/// what this checks is the thing that would break quietly if the guard went:
/// the reading is a length, and no spelling correction was involved.
#[test]
fn a_bound_marker_is_not_corrected_into_a_unit() {
    for input in ["5mm以上", "5mm超", "40C以上", "12kg以上"] {
        let parsed = parse(input, None);
        assert!(
            !parsed
                .findings
                .ambiguities
                .iter()
                .any(|found| found.code == IssueCode::TypoCorrected),
            "{input:?} was read by spelling correction"
        );
        assert!(parsed.suggestions.is_empty(), "{input:?}");

        let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input:?}"));
        let range = best.range.as_ref().expect("a range");
        assert_ne!(
            range.from.dimension,
            Some(Dimension::Area),
            "{input:?} was read as an area"
        );
    }
}

/// The minute survives the lower-bound markers.
///
/// `min` is a bound only as a prefix — `min 12 mm`. As a suffix it is the
/// minute, and listing it there refused `5 min` as a bound, which is why the
/// prefix and suffix markers are kept in separate lists.
#[test]
fn the_minute_is_not_mistaken_for_a_minimum() {
    for (input, seconds) in [("5 min", 300.0), ("5 minutes", 300.0), ("90 min", 5400.0)] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?}"));
        assert_eq!(best.dimension, Some(Dimension::Time), "{input:?}");
        assert_eq!(best.value, Some(seconds), "{input:?}");
    }

    // Upper bounds are untouched by any of this.
    for input in ["5mm以下", "5mm未満", "5mmまで", "≦ 40 C", "under 40 kg"] {
        assert!(
            parse(input, None).best.is_some(),
            "{input:?} lost its upper bound"
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

    // Exponent notation reads now, so a literal at the edge of what an f64
    // holds does reach the bound grammar. What must not happen is a number the
    // text does not contain: overflow to infinity and underflow to zero are
    // both refused rather than reported.
    assert_eq!(
        parse("1e308 kg", None).best.expect("1e308 kg").value,
        Some(1e308)
    );
    let bounded = upper_bound_of(&parse("under 1e308 kg", None)).expect("a bound");
    assert_eq!(bounded.0, None);
    assert_eq!(bounded.1, 1e308);

    for input in ["1e400 kg", "1e309", "1e-400 kg"] {
        let parsed = parse(input, None);
        assert!(
            unread(&parsed),
            "{input:?} reported a number it cannot hold"
        );
        assert!(!parsed.findings.skipped.is_empty(), "{input:?}");
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

/// A range whose two ends are each written as a bound.
///
/// `5mm以上60mm以下` is how a Japanese technical document ordinarily writes
/// "between 5 and 60 mm". It is not a `X to Y` range and not a single bound, so
/// it went unread while each half parsed on its own.
#[test]
fn a_range_written_as_two_bounds_reads_both_ends() {
    for (input, from, to, unit) in [
        ("5mm以上60mm以下", 0.005, 0.06, "m"),
        ("5mm以上60mm未満", 0.005, 0.06, "m"),
        ("12kg以上37kg以下", 12.0, 37.0, "kg"),
        ("≥5mm ≤60mm", 0.005, 0.06, "m"),
        ("≥ 5 mm ≤ 60 mm", 0.005, 0.06, "m"),
        ("min 5 mm max 60 mm", 0.005, 0.06, "m"),
    ] {
        let best = parse(input, None)
            .best
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        let range = best.range.as_ref().expect("a range");

        assert_close(range.from.value.expect("a lower end"), from);
        assert_close(range.to.value.expect("an upper end"), to);
        assert_eq!(range.from.unit.as_deref(), Some(unit), "{input:?}");
        assert_eq!(range.to.unit.as_deref(), Some(unit), "{input:?}");
    }

    // It agrees with the same statement written as an ordinary range.
    let spelled = parse("5 to 60 mm", None).best.expect("a range");
    let bounded = parse("5mm以上60mm以下", None).best.expect("a range");
    let (a, b) = (
        spelled.range.as_ref().expect("a range"),
        bounded.range.as_ref().expect("a range"),
    );
    assert_eq!(a.from.value, b.from.value);
    assert_eq!(a.to.value, b.to.value);
}

/// Two bounds that disagree are not repaired, and neither end is invented.
#[test]
fn two_bounds_that_disagree_are_reported_not_reordered() {
    // Descending: kept as written, and reported.
    let parsed = parse("60mm以上5mm以下", None);
    let range = parsed
        .best
        .as_ref()
        .expect("a range")
        .range
        .as_ref()
        .expect("endpoints");
    assert_close(range.from.value.expect("a value"), 0.06);
    assert_close(range.to.value.expect("a value"), 0.005);
    assert!(
        parsed
            .findings
            .ambiguities
            .iter()
            .any(|found| found.code == IssueCode::AmbiguousNumber),
        "a descending pair was not reported"
    );

    // Ends measuring different things are not a range at all.
    for input in ["5mm以上60kg以下", "5kg以上60mm以下"] {
        assert!(unread(&parse(input, None)), "{input:?} crossed dimensions");
    }

    // A single bound still leaves its other end unstated rather than filling it.
    let single = parse("5mm以上", None).best.expect("a bound");
    assert_eq!(single.range.as_ref().expect("a range").to.value, None);
}
