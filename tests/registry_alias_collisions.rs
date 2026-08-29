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

mod support;
use support::{parse, parse_quantity_fast};

use unravel_nl::{Dimension, DimensionSet, IssueCode, Parsed, Parser, unit_definitions};

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
    }
}

/// The single-letter tokens the compact duration grammar claims, checked
/// against the registry.
///
/// `W` is the only collision that changes the reading: the registry has watt,
/// the grammar has week. `D`, `M` and `S` resolve through the registry's
/// ASCII-case fallback to `d`, `m` and `s`.
///
/// `H` and `S` used to be listed here as cases where "both readings are the
/// same quantity, so nothing needs deciding". That was wrong, and it is the
/// premise this test was built on: `H` is the henry and `S` the siemens, and
/// ASCII-case folding cannot tell either from the hour and the second. Their
/// readings are now withheld from `best` — see
/// `case_folded_symbol_of_an_unmeasured_quantity_is_not_decided` below.
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
        ("5h", 18_000.0, "s"),
        ("5 M", 5.0, "m"),
        ("5m", 5.0, "m"),
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

/// A case-sensitive alias keeps its own meaning: `mM` is millimolar, not `mm`.
///
/// The guards that hand text to the registry used to run their lookup on an
/// ASCII-lowercased copy, so they could only ever protect aliases that are
/// already lowercase. `5 mM` lowercased to `5 mm`, matched the metres-and-
/// centimetres shape, and came back from the fast dispatch as five millimetres
/// while `parse` read millimolar.
#[test]
fn case_sensitive_aliases_keep_their_own_reading() {
    for (input, value, unit, dimension, alternatives) in [
        ("5 mM", 5.0, "mol/m3", Dimension::Concentration, 0),
        ("5mM", 5.0, "mol/m3", Dimension::Concentration, 0),
        ("5 mA", 0.005, "A", Dimension::Current, 0),
        // A unit symbol is not a lower-place count. Reading `A` as the number
        // word `a = 1` invented a centimetre that was never written.
        ("5mA", 0.005, "A", Dimension::Current, 0),
        // The lowercase neighbours these collide with are unchanged.
        ("5 mm", 0.005, "m", Dimension::Length, 0),
        ("5mm", 0.005, "m", Dimension::Length, 0),
    ] {
        let parsed = assert_entry_points_agree(input);
        let best = parsed.best.as_ref().expect("a reading");
        assert_close(best.value.expect("a value"), value);
        assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
        assert_eq!(best.dimension, Some(dimension), "{input:?}");
        assert_eq!(parsed.alternatives.len(), alternatives, "{input:?}");
        assert_eq!(parsed.findings.ambiguities.len(), alternatives, "{input:?}");
        assert!(parsed.findings.skipped.is_empty(), "{input:?}");
    }
}

/// The whole registry, every alias in four cases and both shapes: `parse` and
/// `parse_quantity_fast` name the same value, unit, and dimension.
///
/// A single alias read differently by two entry points is a bug the sweep
/// catches wherever it appears, rather than only where someone thought to look.
/// Confidence and provenance are deliberately not compared — the two dispatches
/// reach the same reading through different grammars and rank it differently,
/// which is a known and separate matter.
///
#[test]
fn every_registry_alias_reads_the_same_through_both_entry_points() {
    let mut checked = 0_usize;
    let mut disagreements = Vec::new();
    for unit in unravel_nl::unit_definitions() {
        for alias in unit.aliases {
            let capitalized = {
                let mut chars = alias.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            };
            for variant in [
                (*alias).to_owned(),
                alias.to_uppercase(),
                alias.to_lowercase(),
                capitalized,
            ] {
                for input in [format!("5 {variant}"), format!("5{variant}")] {
                    checked += 1;
                    let broad = reading_of(&parse(&input, None));
                    let fast = reading_of(&parse_quantity_fast(&input, None));
                    if broad != fast {
                        disagreements.push(format!("{input:?}: parse {broad:?}, fast {fast:?}"));
                    }
                }
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "{} of {checked} inputs disagree:\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
    // The registry is not empty and the sweep really ran over it.
    assert!(checked > 2_000, "only {checked} inputs swept");
}

#[test]
fn compact_week_alias_reaches_both_quantity_entry_points() {
    for input in ["5 w", "5w"] {
        let broad = reading_of(&parse(input, None));
        let fast = reading_of(&parse_quantity_fast(input, None));
        assert_eq!(broad, fast, "{input:?}");
        assert_eq!(broad.map(|reading| reading.0), Some(5.0 * 7.0 * 86_400.0));
    }
}

/// A symbol that names an unmeasured quantity does not get to decide `best`
/// just because folding its case reaches something measurable.
///
/// `5 H` is five hours if the `H` is a sloppy hour and five henries if it is
/// the SI symbol, and this registry has no dimension for inductance — so it
/// cannot tell the two apart, and it used to answer as though it could, with
/// no finding at all. The hour reading is still a reading someone may want, so
/// it is offered as an alternative rather than dropped; what changed is that
/// reading `best` no longer hands it over silently.
#[test]
fn case_folded_symbol_of_an_unmeasured_quantity_is_not_decided() {
    for (input, quantity, alternative) in [
        ("5 H", "inductance", 18_000.0),
        ("5H", "inductance", 18_000.0),
        ("5 S", "electric conductance", 5.0),
        ("5S", "electric conductance", 5.0),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            assert!(parsed.best.is_none(), "{input:?} kept a best reading");

            let kept = parsed
                .alternatives
                .first()
                .unwrap_or_else(|| panic!("{input:?} lost the folded reading"));
            assert_close(kept.value.expect("a value"), alternative);
            assert_eq!(kept.unit.as_deref(), Some("s"), "{input:?}");

            let found = parsed
                .findings
                .ambiguities
                .iter()
                .find(|found| found.code == IssueCode::AmbiguousUnit)
                .unwrap_or_else(|| panic!("{input:?} reported no ambiguity"));
            assert!(
                found.reason.contains(quantity),
                "{input:?}: {}",
                found.reason
            );
        }
    }
}

/// A correctly spelled symbol is never a misspelling of an unrelated unit.
///
/// Each input below is a real SI symbol for a quantity with no dimension here.
/// Did-you-mean matching used to resolve them anyway, by edit distance to
/// something unrelated: the hertz became the hour, the lumen and the candela
/// became the metre, the mole became the mil, the steradian became the stone,
/// and the decibel became the day. None of those is a competing reading worth
/// offering, so the reading is dropped outright and the symbol is reported.
#[test]
fn correctly_spelled_symbol_is_not_typo_corrected_into_another_quantity() {
    for (input, quantity) in [
        ("7 Hz", "frequency"),
        ("7 lm", "luminous flux"),
        ("7 cd", "luminous intensity"),
        ("7 mol", "amount of substance"),
        ("7 Wb", "magnetic flux"),
        ("7 sr", "solid angle"),
        ("7 dB", "logarithmic level"),
    ] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            assert!(parsed.best.is_none(), "{input:?} kept a best reading");
            assert!(
                parsed.alternatives.is_empty(),
                "{input:?} kept an alternative"
            );
            assert!(parsed.suggestions.is_empty(), "{input:?} kept a suggestion");
            assert!(
                !parsed
                    .findings
                    .ambiguities
                    .iter()
                    .any(|found| found.code == IssueCode::TypoCorrected),
                "{input:?} still claims a spelling correction"
            );

            let found = parsed
                .findings
                .skipped
                .iter()
                .find(|found| found.code == IssueCode::UnknownUnit)
                .unwrap_or_else(|| panic!("{input:?} reported no unknown unit"));
            assert!(
                found.reason.contains(quantity),
                "{input:?}: {}",
                found.reason
            );
        }
    }
}

/// The readings that a caller writing a technical document actually wanted.
///
/// These are the symbols the registry had no entry for, so the alias lookup
/// fell through to a case-folded or did-you-mean match and returned a
/// different physical quantity: `7 kN` was 3.6 m/s, `7 MPa` was 7 m/s, and
/// `7 mg` was 7 m. An exact registry entry is what makes the intended reading
/// win, because exact matching runs before either fallback.
#[test]
fn si_prefixed_symbols_read_as_their_own_quantity() {
    for (input, value, unit, dimension) in [
        ("7 kN", 7000.0, "N", Dimension::Force),
        ("7 MN", 7_000_000.0, "N", Dimension::Force),
        ("7 MPa", 7_000_000.0, "Pa", Dimension::Pressure),
        ("7 GPa", 7_000_000_000.0, "Pa", Dimension::Pressure),
        ("7 N/mm2", 7_000_000.0, "Pa", Dimension::Pressure),
        ("7 bar", 700_000.0, "Pa", Dimension::Pressure),
        ("7 kW", 7000.0, "W", Dimension::Power),
        ("7 MW", 7_000_000.0, "W", Dimension::Power),
        ("7 mV", 0.007, "V", Dimension::Voltage),
        ("7 kV", 7000.0, "V", Dimension::Voltage),
        ("7 ms", 0.007, "s", Dimension::Time),
        ("7 mg", 0.000_007, "kg", Dimension::Mass),
        ("7 mm2", 0.000_007, "m2", Dimension::Area),
        ("7 cm2", 0.000_7, "m2", Dimension::Area),
    ] {
        let parsed = parse(input, None);
        let best = parsed
            .best
            .as_ref()
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_close(best.value.expect("a value"), value);
        assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
        assert_eq!(best.dimension, Some(dimension), "{input:?}");
        assert!(
            !parsed
                .findings
                .ambiguities
                .iter()
                .any(|found| found.code == IssueCode::TypoCorrected),
            "{input:?} was read by spelling correction, not by the registry"
        );
    }
}

/// The lowercase spellings these entries could have shadowed.
///
/// `kN` and `kn` are a kilonewton and a knot, `mm` and `mM` a millimetre and a
/// millimolar. Adding the uppercase entry must not move the lowercase reading,
/// and the checks below fail if it does.
#[test]
fn added_entries_leave_their_lowercase_neighbours_alone() {
    for (input, unit, dimension) in [
        ("7 kn", "m/s", Dimension::Speed),
        ("7 mm", "m", Dimension::Length),
        ("7 mM", "mol/m3", Dimension::Concentration),
        ("7 m", "m", Dimension::Length),
        ("7 s", "s", Dimension::Time),
        ("7 g", "kg", Dimension::Mass),
        ("7 N", "N", Dimension::Force),
        ("7 Pa", "Pa", Dimension::Pressure),
        ("7 W", "W", Dimension::Power),
        ("7 V", "V", Dimension::Voltage),
    ] {
        let parsed = parse(input, None);
        let best = parsed
            .best
            .as_ref()
            .unwrap_or_else(|| panic!("{input:?} was not read"));
        assert_eq!(best.unit.as_deref(), Some(unit), "{input:?}");
        assert_eq!(best.dimension, Some(dimension), "{input:?}");
    }
}

/// A symbol two registry quantities both answer to is reported, not settled
/// quietly.
///
/// `C` is the coulomb in the registry and Celsius to the temperature grammar.
/// The temperature grammar wins and probably should — `7 C` is a temperature
/// far more often than a charge — but the coulomb used to vanish without a
/// finding, so nothing told a caller that its symbol names two quantities.
/// Celsius therefore keeps `best` and the coulomb is named beside it.
#[test]
fn symbol_shared_by_two_measured_quantities_is_reported() {
    let parsed = parse("7 C", None);

    let best = parsed.best.as_ref().expect("the temperature reading");
    assert_eq!(best.dimension, Some(Dimension::Temperature));

    let competing = parsed.alternatives.first().expect("the coulomb reading");
    assert_eq!(competing.dimension, Some(Dimension::Charge));
    assert_close(competing.value.expect("a value"), 7.0);

    let found: Vec<_> = parsed
        .findings
        .ambiguities
        .iter()
        .filter(|found| found.code == IssueCode::AmbiguousUnit)
        .collect();
    assert_eq!(found.len(), 1, "one finding, filed once");
    assert_eq!(found[0].ref_text, "C");

    // Lowercase `c` is not the coulomb's symbol, so there is nothing to report.
    let lowercase = parse("7 c", None);
    assert_eq!(
        lowercase.best.as_ref().and_then(|best| best.dimension),
        Some(Dimension::Temperature)
    );
    assert!(lowercase.alternatives.is_empty());
    assert!(lowercase.findings.ambiguities.is_empty());
}

/// A competing reading is only offered from a domain the caller asked for.
///
/// A parser scoped to temperature has no interest in the coulomb, so the
/// ambiguity is not reported to it: the reading it would name is one this
/// parser is configured not to return.
#[test]
fn competing_alias_respects_the_configured_registry() {
    let parsed = Parser::new(DimensionSet::of(&[Dimension::Temperature])).parse("7 C");

    assert_eq!(
        parsed.best.as_ref().and_then(|best| best.dimension),
        Some(Dimension::Temperature)
    );
    assert!(parsed.alternatives.is_empty());
    assert!(parsed.findings.ambiguities.is_empty());
}

/// Withholding files its finding once, however many times dispatch nests.
///
/// `finalize_parsed` documents that it may run more than once over the same
/// result, and both withholding branches write rather than test, so without a
/// fixed point the refusal would be filed again on each pass.
#[test]
fn withheld_symbols_report_exactly_one_finding() {
    for input in ["5 H", "5 S", "7 Hz", "7 lm", "7 mol"] {
        for parsed in [parse(input, None), parse_quantity_fast(input, None)] {
            let filed = parsed
                .findings
                .ambiguities
                .iter()
                .map(|found| found.code)
                .chain(parsed.findings.skipped.iter().map(|found| found.code))
                .filter(|code| matches!(code, IssueCode::UnknownUnit | IssueCode::AmbiguousUnit))
                .count();
            assert_eq!(filed, 1, "{input:?} filed {filed} refusals");
            assert!(parsed.alternatives.len() <= 1, "{input:?}");
        }
    }
}

/// Every SI-prefixed spelling of a common base reads as that base, scaled.
///
/// This registry resolves by table lookup and has no prefix machinery, which is
/// fine until the table has holes. It had 57, across nine bases. A missing
/// entry is not a missing reading: the lookup falls through to did-you-mean,
/// which answers with a different quantity — `590 nm` came back as 1092680 m,
/// having read the nanometre as the nautical mile, and `7 fg` as 2.1336 m by
/// way of the foot.
///
/// The check is on the value, not only the dimension. `nm` resolved to the
/// nautical mile, which is a Length, so a dimension-only check called it
/// correct while it was wrong by a factor of 1.85e12.
#[test]
fn si_prefixed_spellings_read_as_their_base_scaled() {
    // base symbol, canonical unit, dimension, value of one base unit in it
    let bases: &[(&str, &str, Dimension, f64)] = &[
        ("m", "m", Dimension::Length, 1.0),
        ("g", "kg", Dimension::Mass, 1e-3),
        ("s", "s", Dimension::Time, 1.0),
        ("A", "A", Dimension::Current, 1.0),
        ("V", "V", Dimension::Voltage, 1.0),
        ("W", "W", Dimension::Power, 1.0),
        ("N", "N", Dimension::Force, 1.0),
        ("Pa", "Pa", Dimension::Pressure, 1.0),
        ("L", "L", Dimension::Volume, 1.0),
    ];
    let prefixes: &[(&str, f64)] = &[
        ("f", 1e-15),
        ("n", 1e-9),
        ("μ", 1e-6),
        ("m", 1e-3),
        ("c", 1e-2),
        ("k", 1e3),
        ("M", 1e6),
        ("G", 1e9),
    ];

    for (base, canonical, dimension, base_value) in bases {
        for (prefix, scale) in prefixes {
            let input = format!("7 {prefix}{base}");
            let expected = 7.0 * scale * base_value;

            let best = parse(&input, None)
                .best
                .unwrap_or_else(|| panic!("{input:?} was not read"));
            assert_eq!(best.dimension, Some(*dimension), "{input:?}");
            assert_eq!(best.unit.as_deref(), Some(*canonical), "{input:?}");

            let got = best.value.expect("a value");
            assert!(
                (got - expected).abs() <= expected.abs() * 1e-9,
                "{input:?}: expected {expected:e}, got {got:e}"
            );
        }
    }
}

/// The picometre is deliberately absent, because `1 pm` is the afternoon.
///
/// Same call as the bare `F` and `T`: a rare correct reading is not worth a
/// common lost one. Pinned so that adding `pm` has to be a decision.
#[test]
fn the_picometre_yields_to_the_clock() {
    let parsed = parse("1 pm", None);
    let best = parsed.best.as_ref().expect("a clock reading");
    assert_eq!(best.dimension, Some(Dimension::Time));
    assert!(
        !unit_definitions().iter().any(|unit| unit.id == "pm"),
        "the picometre was added; decide about `1 pm` before pinning this"
    );
}
