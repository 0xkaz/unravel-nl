//! Inputs where a registry alias collides with an informal grammar.
//!
//! Two grammars read text the registry also reads: the compact duration idiom
//! (`5w` as five weeks) and the closed-up compound idiom (`1m80` as 1.8 m).
//! `parse` consults the registry first and the fast quantity dispatch consults
//! it last, so a collision used to mean the reading depended on which entry
//! point the caller picked — `5 W` was five watts through `parse` and five
//! weeks through `parse_quantity_fast`, and `5m3` was 5000 L through one and
//! 5.03 m through the other.
//!
//! The rule everywhere now is that a registry alias wins. Where the losing
//! reading was never plausible (`5 W` is not a week, and nobody writes the
//! compound idiom with a space) it is simply not read; where it is plausible
//! (`5m3` is shaped exactly like `1m80`) it is reported as an alternative with
//! an `AmbiguousUnit` finding rather than dropped.

use unravel_nl::{Dimension, IssueCode, Parsed, parse, parse_all, parse_quantity_fast};

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

/// The value, unit, and dimension of `best`, which is what every entry point
/// has to agree on. Confidence deliberately stays out: the grammars rank their
/// own readings, and the fast dispatch reaching a reading sooner is not a
/// disagreement about what the text says.
fn reading_of(parsed: &Parsed) -> Option<(f64, String, Option<Dimension>)> {
    parsed.best.as_ref().map(|best| {
        (
            best.value.expect("a value"),
            best.unit.clone().expect("a unit"),
            best.dimension,
        )
    })
}

fn assert_entry_points_agree(input: &str) -> Parsed {
    let broad = parse(input, None);
    let fast = parse_quantity_fast(input, None);
    assert_eq!(
        reading_of(&broad),
        reading_of(&fast),
        "parse and parse_quantity_fast disagree on {input:?}"
    );
    assert_eq!(
        broad.alternatives.len(),
        fast.alternatives.len(),
        "alternative count differs on {input:?}"
    );
    assert_eq!(
        broad
            .findings
            .ambiguities
            .iter()
            .map(|issue| issue.code)
            .collect::<Vec<_>>(),
        fast.findings
            .ambiguities
            .iter()
            .map(|issue| issue.code)
            .collect::<Vec<_>>(),
        "ambiguity codes differ on {input:?}"
    );
    fast
}

#[test]
fn watt_beats_the_week_idiom_through_every_entry_point() {
    for input in ["5 W", "5W"] {
        let parsed = assert_entry_points_agree(input);
        let best = parsed.best.as_ref().expect("a reading");
        assert_close(best.value.expect("a value"), 5.0);
        assert_eq!(best.unit.as_deref(), Some("W"));
        assert_eq!(best.dimension, Some(Dimension::Power));
        // The week reading is not a competing reading of `5 W`, so it is not
        // reported as one.
        assert!(parsed.alternatives.is_empty(), "{input:?}");
        assert!(parsed.findings.ambiguities.is_empty(), "{input:?}");
        assert!(parsed.findings.skipped.is_empty(), "{input:?}");

        let scanned = parse_all(input, None);
        assert_eq!(scanned.len(), 1, "{input:?}");
        let scanned = scanned[0].parsed.best.as_ref().expect("a reading");
        assert_eq!(scanned.unit.as_deref(), Some("W"), "{input:?}");
        assert_close(scanned.value.expect("a value"), 5.0);
    }
}

/// The single-letter tokens the compact duration grammar claims, checked
/// against the registry.
///
/// `W` is the only collision that changes the reading: the registry has watt,
/// the grammar has week. `D`, `H`, `M` and `S` resolve through the registry's
/// ASCII-case fallback to `d`, `h`, `m` and `s`, and for the time ones both
/// readings are the same quantity, so nothing needs deciding.
///
/// One case is knowingly outside this test: a lowercase `5w` is read as five
/// weeks by the fast dispatch and not read at all by `parse`, because
/// `InputFeatures::maybe_duration` does not list `w` and so `parse` never
/// reaches the duration grammar. That is a gate gap, not an alias collision —
/// the registry has no lowercase `w` — and it is left alone here.
#[test]
fn single_letter_duration_tokens_read_the_same_through_every_entry_point() {
    for token in ["W", "D", "H", "M", "S", "d", "h", "m", "s"] {
        for input in [format!("5 {token}"), format!("5{token}")] {
            assert_entry_points_agree(&input);
        }
    }
}

#[test]
fn single_letter_duration_tokens_keep_their_readings() {
    for (input, expected_value, expected_unit) in [
        ("5 D", 432_000.0, "s"),
        ("5d", 432_000.0, "s"),
        ("5 H", 18_000.0, "s"),
        ("5h", 18_000.0, "s"),
        ("5 M", 5.0, "m"),
        ("5m", 5.0, "m"),
        ("5 S", 5.0, "s"),
        ("5s", 5.0, "s"),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input:?}"));
            assert_close(best.value.expect("a value"), expected_value);
            assert_eq!(best.unit.as_deref(), Some(expected_unit), "{input:?}");
            assert!(parsed.alternatives.is_empty(), "{input:?}");
            assert!(parsed.findings.ambiguities.is_empty(), "{input:?}");
        }
    }
}

#[test]
fn closed_up_registry_alias_leads_and_reports_the_compound_reading() {
    for (input, value, unit, dimension, alternative) in [
        ("5m3", 5000.0, "L", Dimension::Volume, 5.03),
        ("5ft2", 0.464_515_2, "m2", Dimension::Area, 1.5748),
    ] {
        let parsed = assert_entry_points_agree(input);
        let best = parsed.best.as_ref().expect("a reading");
        assert_close(best.value.expect("a value"), value);
        assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
        assert_eq!(best.dimension, Some(dimension), "{input:?}");

        // The compound reading is plausible for text of this shape, so it is
        // reported rather than dropped.
        assert_eq!(parsed.alternatives.len(), 1, "{input:?}");
        let competing = &parsed.alternatives[0];
        assert_close(competing.value.expect("a value"), alternative);
        assert_eq!(competing.unit.as_deref(), Some("m"), "{input:?}");
        assert_eq!(competing.dimension, Some(Dimension::Length), "{input:?}");
        assert!(
            competing.confidence < best.confidence,
            "the alternative outranks the reading it lost to on {input:?}"
        );

        assert_eq!(parsed.findings.ambiguities.len(), 1, "{input:?}");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousUnit,
            "{input:?}"
        );
        assert_eq!(
            parsed.findings.ambiguities[0].candidate_count,
            Some(2),
            "{input:?}"
        );
        assert_eq!(parsed.findings.ambiguities[0].span.text, input);
        assert!(parsed.findings.skipped.is_empty(), "{input:?}");

        // `parse_all` is built on the fast dispatch and inherits both.
        let scanned = parse_all(input, None);
        assert_eq!(scanned.len(), 1, "{input:?}");
        let scanned = &scanned[0].parsed;
        assert_close(
            scanned.best.as_ref().expect("a reading").value.expect("v"),
            value,
        );
        assert_eq!(scanned.alternatives.len(), 1, "{input:?}");
        assert_eq!(
            scanned.findings.ambiguities[0].code,
            IssueCode::AmbiguousUnit,
            "{input:?}"
        );
    }
}

/// The spaced form has one reading, not two: the compound idiom never puts a
/// space before its unit, so there is nothing to report.
#[test]
fn spaced_registry_alias_reports_nothing() {
    for (input, value, unit) in [("5 m3", 5000.0, "L"), ("5 ft2", 0.464_515_2, "m2")] {
        let parsed = assert_entry_points_agree(input);
        let best = parsed.best.as_ref().expect("a reading");
        assert_close(best.value.expect("a value"), value);
        assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
        assert!(parsed.alternatives.is_empty(), "{input:?}");
        assert!(parsed.findings.ambiguities.is_empty(), "{input:?}");
        assert!(parsed.findings.skipped.is_empty(), "{input:?}");
    }
}

/// Every documented compound, unchanged: no new alternative, no new finding.
#[test]
fn documented_compounds_are_untouched() {
    for (input, value, unit) in [
        ("1m80", 1.8, "m"),
        ("180cm", 1.8, "m"),
        ("5mm", 0.005, "m"),
        ("5ft 11", 1.8034, "m"),
        ("5 ft 11", 1.8034, "m"),
        ("5ft", 1.524, "m"),
        ("3 yd 2 ft", 3.3528, "m"),
        ("2 lb 3 oz", 0.992_233_309_375, "kg"),
        ("2h30", 9000.0, "s"),
        ("1h30m", 5400.0, "s"),
        ("1h", 3600.0, "s"),
        ("20 min", 1200.0, "s"),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input:?}"));
            assert_close(best.value.expect("a value"), value);
            assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
            assert!(parsed.alternatives.is_empty(), "{input:?}");
            assert!(parsed.findings.ambiguities.is_empty(), "{input:?}");
            assert!(parsed.findings.skipped.is_empty(), "{input:?}");
        }
    }

    // Neither idiom reads this one, and the registry has no `m80cm`: it stays
    // unread rather than acquiring a guess.
    for parsed in [parse("1m80cm", None), parse_quantity_fast("1m80cm", None)] {
        assert!(parsed.best.is_none());
        assert_eq!(parsed.findings.skipped.len(), 1);
        assert_eq!(parsed.findings.skipped[0].code, IssueCode::NoValue);
    }
}
