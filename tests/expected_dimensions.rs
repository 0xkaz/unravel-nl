//! `ParseCtx::expected_dimensions` as a hard filter on every entry point.
//!
//! Every cross-entry-point disagreement this crate has had was a collision
//! between measurement domains — `mM` against `mm`, `W` against a week, `mA`
//! against a length compound, `m3` against the `1m80` height idiom. Inside one
//! domain there were none. Declaring the domains a field accepts is therefore
//! not a hint: it is what makes those collisions structurally impossible, and
//! this file is the proof that it binds wherever a caller can enter.
//!
//! Refusal is loud. A reading the declared set does not accept never comes back
//! as `best`, but it is not dropped either: it moves to `alternatives` and is
//! reported as `REJECTED_BY_POLICY`, because a value the caller typed and the
//! parser read is not something the no-silent-loss contract lets us discard.

use unravel_nl::{
    CanonicalizeRequest, Dimension, DimensionSet, IssueCode, Kind, Locale, ParseCtx, Parsed,
    canonicalize_values, complete, complete_readings, parse, parse_all, parse_date_fast,
    parse_dimensions_for_editor, parse_number_fast, parse_quantity_fast, parse_recurrence_fast,
};

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}

fn ctx(dimensions: &[Dimension]) -> Option<ParseCtx> {
    Some(ParseCtx {
        expected_dimensions: DimensionSet::of(dimensions),
        ..ParseCtx::default()
    })
}

fn rejection(parsed: &Parsed) -> Option<&str> {
    parsed
        .findings
        .skipped
        .iter()
        .find(|issue| issue.code == IssueCode::RejectedByPolicy)
        .map(|issue| issue.reason.as_str())
}

/// The four historical collisions, refused outright once lengths are declared.
///
/// `5 mM` is a concentration, `5 W` a power, `5 mA` a current, `5 m3` a volume.
/// None of them can reach `best` in a length field any more, through either
/// entry point that reads a quantity.
#[test]
fn cross_domain_collisions_are_refused_under_a_length_field() {
    for (input, dimension, reason) in [
        (
            "5 mM",
            Dimension::Concentration,
            "dimension concentration is outside the expected dimensions: length",
        ),
        (
            "5 W",
            Dimension::Power,
            "dimension power is outside the expected dimensions: length",
        ),
        (
            "5 mA",
            Dimension::Current,
            "dimension current is outside the expected dimensions: length",
        ),
        (
            "5 m3",
            Dimension::Volume,
            "dimension volume is outside the expected dimensions: length",
        ),
    ] {
        for parsed in [
            parse(input, ctx(&[Dimension::Length])),
            parse_quantity_fast(input, ctx(&[Dimension::Length])),
        ] {
            assert!(parsed.best.is_none(), "{input}");
            // Refused, not lost: the reading is still on the table.
            assert_eq!(parsed.alternatives.len(), 1, "{input}");
            assert_eq!(parsed.alternatives[0].dimension, Some(dimension), "{input}");
            assert_eq!(rejection(&parsed), Some(reason), "{input}");
        }

        // Adding areas does not admit any of them either.
        let pair = parse(input, ctx(&[Dimension::Length, Dimension::Area]));
        assert!(pair.best.is_none(), "{input}");
        assert_eq!(
            rejection(&pair),
            Some(reason.replace("length", "length, area").as_str()),
            "{input}"
        );

        // Unrestricted, every one of them still reads exactly as before.
        let free = parse(input, None);
        assert_eq!(
            free.best.expect(input).dimension,
            Some(dimension),
            "{input}"
        );
    }
}

/// Written closed up, two of the four have a competing *length* reading, and
/// declaring lengths is what promotes it.
///
/// `5mA` is the registry's milliamp and also `5 m` + `1 cm`-style compound;
/// `5m3` is the cubic metre and also the `1m80` height idiom. The registry unit
/// leads when nothing is declared; under a length field the compound is the
/// only reading left, so it is promoted rather than refused along with the one
/// the field cannot hold.
#[test]
fn an_in_domain_alternative_is_promoted_rather_than_refused_with_the_rest() {
    for (input, metres, refused) in [
        ("5mA", 5.01, Dimension::Current),
        ("5m3", 5.03, Dimension::Volume),
    ] {
        for parsed in [
            parse(input, ctx(&[Dimension::Length])),
            parse_quantity_fast(input, ctx(&[Dimension::Length])),
        ] {
            let best = parsed.best.as_ref().expect(input);
            assert_eq!(best.dimension, Some(Dimension::Length), "{input}");
            assert_eq!(best.unit.as_deref(), Some("m"), "{input}");
            assert_close(best.value.expect(input), metres);
            // The promotion is still a choice, so it is still reported.
            assert!(rejection(&parsed).is_some(), "{input}");
            assert!(
                parsed
                    .alternatives
                    .iter()
                    .any(|reading| reading.dimension == Some(refused)),
                "{input}"
            );
        }
    }
}

/// Every entry point answers the same way, which is the whole point.
#[test]
fn every_entry_point_enforces_the_declared_domains() {
    let dimensions = ctx(&[Dimension::Length]);
    let input = "5 W";

    for parsed in [
        parse(input, dimensions.clone()),
        parse_quantity_fast(input, dimensions.clone()),
    ] {
        assert!(parsed.best.is_none());
        assert!(rejection(&parsed).is_some());
    }

    // The narrow entry points that cannot read a power at all report the same
    // absence they always did, rather than a refusal of something unread.
    for parsed in [
        parse_number_fast(input, dimensions.clone()),
        parse_date_fast(input, dimensions.clone()),
        parse_recurrence_fast(input, dimensions.clone()),
    ] {
        assert!(parsed.best.is_none());
        assert_eq!(rejection(&parsed), None);
    }

    // The scan keeps the span and carries the refusal on it.
    let scanned = parse_all(input, dimensions.clone());
    assert_eq!(scanned.len(), 1);
    assert_eq!(scanned[0].text, "5 W");
    assert!(scanned[0].parsed.best.is_none());
    assert!(rejection(&scanned[0].parsed).is_some());

    let editor = parse_dimensions_for_editor(input, dimensions.clone());
    assert_eq!(editor.len(), 1);
    assert!(editor[0].parsed.best.is_none());
    assert!(rejection(&editor[0].parsed).is_some());

    // A picker must not offer what the field cannot hold.
    assert!(complete_readings(input, dimensions.clone()).is_empty());
    assert!(
        complete("wa", dimensions.clone())
            .iter()
            .all(|item| item.dimension == Some(Dimension::Length))
    );

    let canonical = canonicalize_values(&[CanonicalizeRequest::new("width", input, dimensions)]);
    assert!(!canonical[0].ok);
    assert!(canonical[0].canonical.is_none());
    assert!(
        canonical[0]
            .message
            .as_ref()
            .expect("a message")
            .contains("[REJECTED_BY_POLICY]")
    );
}

/// An empty set is exactly the absence of the field, everywhere.
#[test]
fn an_empty_set_is_no_restriction_at_all() {
    for input in [
        "5 mM",
        "5 W",
        "5 mA",
        "5 m3",
        "5 kg",
        "3640",
        "every monday",
    ] {
        let empty = Some(ParseCtx::default());
        assert_eq!(parse(input, empty.clone()), parse(input, None), "{input}");
        assert_eq!(
            parse_quantity_fast(input, empty.clone()),
            parse_quantity_fast(input, None),
            "{input}"
        );
        assert_eq!(
            parse_all(input, empty.clone()).len(),
            parse_all(input, None).len(),
            "{input}"
        );
        assert_eq!(
            parse_dimensions_for_editor(input, empty).len(),
            parse_dimensions_for_editor(input, None).len(),
            "{input}"
        );
    }
}

/// A reading with no measurement domain is not refused by one.
///
/// A bare number, a date, and a schedule have no dimension to be outside the
/// declared set, and refusing them would say something the declaration does not
/// say. It is also what keeps `parse_number_fast` usable in a dimensioned
/// field, and what keeps a labelled bare number readable by the editor.
#[test]
fn dimensionless_readings_survive_a_declared_set() {
    let dimensions = ctx(&[Dimension::Length]);

    let number = parse_number_fast("3640", dimensions.clone());
    let best = number.best.as_ref().expect("a number");
    assert_eq!(best.kind, Kind::Number);
    assert_eq!(best.value, Some(3640.0));
    assert_eq!(rejection(&number), None);
    // Length among the declared domains is what offers the millimetre reading.
    assert_eq!(number.alternatives[0].unit.as_deref(), Some("mm"));

    let recurrence = parse_recurrence_fast("every monday", dimensions.clone());
    assert_eq!(
        recurrence.best.as_ref().expect("a rule").kind,
        Kind::Recurrence
    );
    assert_eq!(rejection(&recurrence), None);

    let broad = parse("3640", ctx(&[Dimension::Area]));
    assert_eq!(broad.best.as_ref().expect("a number").kind, Kind::Number);
    assert_eq!(rejection(&broad), None);
}

/// A range carries its dimensions on its endpoints, so that is where it is
/// judged. Reading the range itself would see `dimension: None` and let a mass
/// interval straight into a length field.
#[test]
fn a_range_is_judged_by_its_endpoints() {
    let parsed = parse("5-10 kg", ctx(&[Dimension::Length]));
    assert!(parsed.best.is_none());
    assert_eq!(
        rejection(&parsed),
        Some("dimension mass is outside the expected dimensions: length")
    );
    assert_eq!(parsed.alternatives[0].kind, Kind::Range);

    let allowed = parse("5-10 kg", ctx(&[Dimension::Mass]));
    assert_eq!(allowed.best.as_ref().expect("a range").kind, Kind::Range);
    assert_eq!(rejection(&allowed), None);
}

/// The editor extractor keeps the set it always accepted when nothing is
/// declared, and narrows it — loudly — when something is.
#[test]
fn the_editor_extractor_keeps_its_contract_and_narrows_on_request() {
    let text = "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640";
    let japanese = |dimensions: &[Dimension]| ParseCtx {
        locale: Some(Locale::Ja),
        expected_dimensions: DimensionSet::of(dimensions),
        ..ParseCtx::default()
    };

    let texts = |matches: &[unravel_nl::ParsedMatch]| {
        matches
            .iter()
            .map(|found| found.text.clone())
            .collect::<Vec<_>>()
    };

    // Undeclared: exactly what it has always extracted.
    assert_eq!(
        texts(&parse_dimensions_for_editor(text, Some(japanese(&[])))),
        vec!["3m", "4m", "6帖", "3640"]
    );

    // Areas only: the two lengths are refused rather than quietly missing, and
    // the tatami area is the only reading left standing.
    let areas = parse_dimensions_for_editor(text, Some(japanese(&[Dimension::Area])));
    assert_eq!(texts(&areas), vec!["3m", "4m", "6帖"]);
    for refused in &areas[..2] {
        assert!(refused.parsed.best.is_none(), "{}", refused.text);
        assert_eq!(
            rejection(&refused.parsed),
            Some("dimension length is outside the expected dimensions: area")
        );
    }
    let area = areas[2].parsed.best.as_ref().expect("a tatami area");
    assert_eq!(area.dimension, Some(Dimension::Area));
    assert_eq!(area.unit.as_deref(), Some("m2"));
}
