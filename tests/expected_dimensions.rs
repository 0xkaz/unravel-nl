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
    canonicalize_values, complete, complete_readings, parse, parse_date_fast,
    parse_dimensions_for_editor, parse_number_fast, parse_quantity_fast,
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

/// Written closed up, `5m3` has a competing *length* reading, and declaring
/// lengths is what promotes it.
///
/// `5m3` is the cubic metre and also the `1m80` height idiom. The registry unit
/// leads when nothing is declared; under a length field the compound is the
/// only reading left, so it is promoted rather than refused along with the one
/// the field cannot hold. `5mA` has no compound reading: `A` is a unit symbol,
/// not a written lower-place count.
#[test]
fn an_in_domain_alternative_is_promoted_rather_than_refused_with_the_rest() {
    for (input, metres, refused) in [("5m3", 5.03, Dimension::Volume)] {
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

    for parsed in [
        parse("5mA", ctx(&[Dimension::Length])),
        parse_quantity_fast("5mA", ctx(&[Dimension::Length])),
    ] {
        assert!(parsed.best.is_none());
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.alternatives[0].dimension, Some(Dimension::Current));
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
    ] {
        assert!(parsed.best.is_none());
        assert_eq!(rejection(&parsed), None);
    }

    // The scan keeps the span and carries the refusal on it.
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
    for input in ["5 mM", "5 W", "5 mA", "5 m3", "5 kg", "3640"] {
        let empty = Some(ParseCtx::default());
        assert_eq!(parse(input, empty.clone()), parse(input, None), "{input}");
        assert_eq!(
            parse_quantity_fast(input, empty.clone()),
            parse_quantity_fast(input, None),
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
/// A bare number and a date have no dimension to be outside the
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

    let broad = parse("3640", ctx(&[Dimension::Area]));
    assert_eq!(broad.best.as_ref().expect("a number").kind, Kind::Number);
    assert_eq!(rejection(&broad), None);
}

/// The fuzzy-temperature grammar is opt-in, and only a declaration opts in.
///
/// `parse_fuzzy_temperature` asks whether the set *contains* `Temperature`, not
/// whether it *allows* it, which is deliberate and load-bearing: `hot` is an
/// English word long before it is a measurement, so reading it as 27–35 °C in
/// an undeclared field would be an invention. The empty set allows every
/// dimension and must still leave this grammar off, which is the difference the
/// two predicates make and the reason this test exists — relaxing `contains` to
/// `allows` otherwise passes the whole suite.
#[test]
fn the_fuzzy_temperature_grammar_stays_off_until_temperature_is_declared() {
    for input in ["it's hot", "it's cold", "今日は暑い"] {
        // Off: no declaration, and an empty declaration, are the same thing.
        for parsed in [parse(input, None), parse(input, ctx(&[]))] {
            assert!(parsed.best.is_none(), "{input}");
        }
        // Off: a declaration that is not this one does not opt in either.
        assert!(
            parse(input, ctx(&[Dimension::Length])).best.is_none(),
            "{input}"
        );

        // On: only where the caller said the field holds a temperature.
        for dimensions in [
            &[Dimension::Temperature][..],
            &[Dimension::Length, Dimension::Temperature][..],
        ] {
            let parsed = parse(input, ctx(dimensions));
            let best = parsed.best.as_ref().expect(input);
            assert_eq!(best.kind, Kind::Range, "{input}");
            let range = best.range.as_ref().expect(input);
            assert_eq!(range.from.unit.as_deref(), Some("C"), "{input}");
            assert_eq!(
                range.from.dimension,
                Some(Dimension::Temperature),
                "{input}"
            );
            assert_eq!(rejection(&parsed), None, "{input}");
        }
    }

    // And the reading it produces is a temperature, so a field that declared
    // some *other* domain could not have been given it anyway.
    let hot = parse("it's hot", ctx(&[Dimension::Temperature]));
    let range = hot.best.expect("a range").range.expect("endpoints");
    assert_close(range.from.value.expect("a low"), 27.0);
    assert_close(range.to.value.expect("a high"), 35.0);
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

/// The editor extractor extracts the same candidates whatever is declared, and
/// a declaration decides which of them are *read* rather than refused.
///
/// A declaration composes with the label the extractor infers, and the
/// composition is an intersection: it can only narrow what the label already
/// allowed. Declaring lengths does not turn `予算1234` — a budget, under no
/// dimension label at all — into one.
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

    // The candidates never change: the budget, the date, and the unlabelled
    // digits are no dimension under any declaration.
    for dimensions in [
        &[][..],
        &[Dimension::Length][..],
        &[Dimension::Length, Dimension::Area][..],
        &[Dimension::Area][..],
    ] {
        assert_eq!(
            texts(&parse_dimensions_for_editor(
                text,
                Some(japanese(dimensions))
            )),
            vec!["3m", "4m", "6帖", "3640"],
            "{dimensions:?}"
        );
    }

    // Undeclared, and with both domains declared, every one of them reads.
    for dimensions in [&[][..], &[Dimension::Length, Dimension::Area][..]] {
        let found = parse_dimensions_for_editor(text, Some(japanese(dimensions)));
        for reading in &found {
            assert!(reading.parsed.best.is_some(), "{}", reading.text);
            assert_eq!(rejection(&reading.parsed), None, "{}", reading.text);
        }
    }

    // Lengths only: the two widths and the labelled bare number read, and the
    // tatami area is refused rather than quietly missing.
    let lengths = parse_dimensions_for_editor(text, Some(japanese(&[Dimension::Length])));
    assert_eq!(
        lengths[0].parsed.best.as_ref().expect("a width").unit,
        Some("m".to_owned())
    );
    assert!(lengths[2].parsed.best.is_none());
    assert_eq!(
        rejection(&lengths[2].parsed),
        Some("dimension area is outside the expected dimensions: length")
    );
    // `寸法3640` is a millimetre length written without its unit, and a length
    // field takes it.
    assert_eq!(
        lengths[3].parsed.best.as_ref().expect("a number").value,
        Some(3640.0)
    );
    assert_eq!(
        lengths[3].parsed.alternatives[0].unit.as_deref(),
        Some("mm")
    );

    // Areas only: the two lengths are refused rather than quietly missing, the
    // tatami area is the only reading left standing, and the labelled bare
    // number — a length too — is refused with them rather than dropped.
    let areas = parse_dimensions_for_editor(text, Some(japanese(&[Dimension::Area])));
    for refused in [&areas[0], &areas[1], &areas[3]] {
        assert!(refused.parsed.best.is_none(), "{}", refused.text);
        assert_eq!(
            rejection(&refused.parsed),
            Some("dimension length is outside the expected dimensions: area"),
            "{}",
            refused.text
        );
    }
    // Refused, not lost: the millimetre reading it would have been is still on
    // the table.
    assert_eq!(areas[3].parsed.alternatives[0].unit.as_deref(), Some("mm"));
    assert_eq!(areas[3].parsed.alternatives[0].value, Some(3640.0));
    let area = areas[2].parsed.best.as_ref().expect("a tatami area");
    assert_eq!(area.dimension, Some(Dimension::Area));
    assert_eq!(area.unit.as_deref(), Some("m2"));
}

/// A declaration narrows the label; it never overrules it.
///
/// The label is the crate's own inference about the candidate, so a
/// contradicting declaration cannot promote a reading the label rules out: a
/// bare number under an *area* label is not a length because a length was
/// declared. Nothing is refused there either, since an area label offers no
/// reading the declaration could refuse — contrast `寸法3640`, whose millimetre
/// length it does refuse, out loud.
#[test]
fn a_declaration_composes_with_the_label_rather_than_replacing_it() {
    // A contradicting label: nothing was ever a dimension, so nothing matches
    // and nothing is refused — exactly as with no declaration at all.
    assert!(parse_dimensions_for_editor("面積3640", None).is_empty());
    assert!(parse_dimensions_for_editor("面積3640", ctx(&[Dimension::Length])).is_empty());

    // An unknown label is not a dimension label, and declaring one does not
    // make it into one.
    assert!(parse_dimensions_for_editor("予算1234", ctx(&[Dimension::Length])).is_empty());
    // Nor is an unlabelled bare number.
    assert!(parse_dimensions_for_editor("3640", ctx(&[Dimension::Length])).is_empty());

    // An agreeing label reads, under either declaration that contains it.
    for dimensions in [
        &[Dimension::Length][..],
        &[Dimension::Length, Dimension::Area][..],
    ] {
        let found = parse_dimensions_for_editor("寸法3640", ctx(dimensions));
        assert_eq!(found.len(), 1, "{dimensions:?}");
        assert_eq!(
            found[0].parsed.best.as_ref().expect("a number").value,
            Some(3640.0)
        );
    }
}

/// A labelled bare number the declaration refuses is reported, not dropped.
///
/// `寸法3640` is a length in millimetres. In a field that declared areas there
/// is no reading of it left — but the reading it refused is one the caller
/// typed, so it comes back as a match with `best: None` and the refusal on it,
/// which is what [`parse_dimensions_for_editor`]'s documentation promises and
/// what the no-silent-loss contract requires.
#[test]
fn a_labelled_bare_number_outside_the_declared_set_is_refused_rather_than_dropped() {
    // Undeclared, it reads.
    let free = parse_dimensions_for_editor("寸法3640", None);
    assert_eq!(free.len(), 1);
    assert_eq!(
        free[0].parsed.best.as_ref().expect("a number").value,
        Some(3640.0)
    );

    for (dimensions, reason) in [
        (
            &[Dimension::Area][..],
            "dimension length is outside the expected dimensions: area",
        ),
        (
            &[Dimension::Mass][..],
            "dimension length is outside the expected dimensions: mass",
        ),
    ] {
        let refused = parse_dimensions_for_editor("寸法3640", ctx(dimensions));
        assert_eq!(refused.len(), 1, "{dimensions:?}");
        assert_eq!(refused[0].text, "3640");
        assert!(refused[0].parsed.best.is_none(), "{dimensions:?}");
        assert_eq!(rejection(&refused[0].parsed), Some(reason));
        // The millimetre length it refused is kept rather than dropped.
        assert_eq!(refused[0].parsed.alternatives.len(), 1);
        assert_eq!(
            refused[0].parsed.alternatives[0].unit.as_deref(),
            Some("mm")
        );
        assert_eq!(refused[0].parsed.alternatives[0].value, Some(3640.0));
    }

    // A labelled *unit* was already refused this way; the bare number now
    // matches it instead of disappearing.
    let unit = parse_dimensions_for_editor("幅3m", ctx(&[Dimension::Area]));
    assert_eq!(unit.len(), 1);
    assert!(unit[0].parsed.best.is_none());
    assert_eq!(
        rejection(&unit[0].parsed),
        Some("dimension length is outside the expected dimensions: area")
    );
}

/// The extractor's own label inference is not a caller declaration.
///
/// The label decides which reading of a candidate is *kept*; it never refuses
/// one, because a refusal is a statement about what the caller declared. When
/// the caller declared nothing, the extractor must therefore answer exactly
/// what it answered before declarations existed — no promotion, no
/// `REJECTED_BY_POLICY`.
#[test]
fn the_label_hint_never_refuses_and_never_reranks() {
    // `1m2` is the registry's square metre next to a width label, and `5m3` its
    // cubic metre. Both have a competing *length* compound, and neither label
    // may promote it.
    for (input, value, unit, dimension) in [
        ("幅1m2", 1.0, "m2", Dimension::Area),
        ("幅5m3", 5000.0, "L", Dimension::Volume),
        ("高さ5ft2", 0.4645152, "m2", Dimension::Area),
    ] {
        let found = parse_dimensions_for_editor(input, None);
        assert_eq!(found.len(), 1, "{input}");
        let best = found[0].parsed.best.as_ref().expect(input);
        assert_close(best.value.expect(input), value);
        assert_eq!(best.unit.as_deref(), Some(unit), "{input}");
        assert_eq!(best.dimension, Some(dimension), "{input}");
        // Nothing the caller declared was refused, so nothing is reported as
        // refused.
        assert_eq!(rejection(&found[0].parsed), None, "{input}");
        assert!(
            found[0]
                .parsed
                .findings
                .skipped
                .iter()
                .all(|issue| issue.code != IssueCode::RejectedByPolicy),
            "{input}"
        );
    }

    // The empty set is the absence of the field here too, on inputs where the
    // label hint is what does the work.
    for input in [
        "幅1m2",
        "幅5m3",
        "幅3m",
        "寸法3640",
        "面積3640",
        "予算1234",
        "3640",
        "幅3m×奥行4m、予算1234、6帖、寸法3640",
    ] {
        assert_eq!(
            parse_dimensions_for_editor(input, Some(ParseCtx::default())),
            parse_dimensions_for_editor(input, None),
            "{input}"
        );
    }
}

/// A promotion says what happened, not the opposite of it.
///
/// `report_closed_compound_alternative` writes its ambiguity while the registry
/// unit is still the reading. When the declared dimensions then refuse that
/// unit, the sentence has to be retold: the flagship case of this feature would
/// otherwise ship a finding stating that the reading which was refused is the
/// one that was read.
#[test]
fn a_promoted_compound_is_described_as_the_reading_it_became() {
    const REGISTRY_READ: &str = "Written closed up, this is both a registry unit and a compound quantity; the registry unit was read.";
    const COMPOUND_READ: &str = "Written closed up, this is both a registry unit and a compound quantity; the registry unit is outside the expected dimensions, so the compound quantity was read.";
    const NEITHER_READ: &str = "Written closed up, this is both a registry unit and a compound quantity; the registry unit is outside the expected dimensions, and neither reading was accepted.";

    let ambiguity = |parsed: &Parsed| {
        parsed
            .findings
            .ambiguities
            .iter()
            .find(|issue| issue.code == IssueCode::AmbiguousUnit)
            .map(|issue| issue.reason.clone())
            .expect("an ambiguity")
    };

    for (input, metres) in [("5m3", 5.03), ("5ft2", 1.5748)] {
        // Undeclared, the registry unit is read, and the finding says so.
        let free = parse(input, None);
        assert_ne!(free.best.as_ref().expect(input).unit.as_deref(), Some("m"));
        assert_eq!(ambiguity(&free), REGISTRY_READ, "{input}");

        // Under lengths the compound is promoted, and the finding now describes
        // that, through every entry point that reports it.
        for parsed in [
            parse(input, ctx(&[Dimension::Length])),
            parse_quantity_fast(input, ctx(&[Dimension::Length])),
        ] {
            let best = parsed.best.as_ref().expect(input);
            assert_eq!(best.unit.as_deref(), Some("m"), "{input}");
            assert_close(best.value.expect(input), metres);
            assert_eq!(ambiguity(&parsed), COMPOUND_READ, "{input}");
        }
    }

    // With nothing left to promote, the finding says that instead of naming a
    // winner there is not.
    let refused = parse("5m3", ctx(&[Dimension::Area]));
    assert!(refused.best.is_none());
    assert_eq!(ambiguity(&refused), NEITHER_READ);
}
