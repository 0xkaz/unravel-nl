//! Pins the `IssueCode` variants that no Rust test asserted.
//!
//! `Empty`, `UnitAssumed`, and `AmbiguousUnit` are all reachable from `parse`
//! at this commit, but were only ever checked from `tests/wasm_node_smoke.mjs`
//! (which needs a `wasm-pack` build) or not at all. `UnknownUnit` is documented
//! as reserved and is never constructed; a sweep guards that claim.

use unravel_nl::{
    Dimension, Findings, IssueCode, IssueSeverity, Kind, Locale, ParseCtx, Parsed, Skipped, Span,
    parse, parse_all, ranked_findings,
};

#[test]
fn reports_empty_for_blank_input() {
    let parsed = parse("", None);
    assert!(parsed.best.is_none());
    assert_eq!(parsed.findings.skipped.len(), 1);
    let skipped = &parsed.findings.skipped[0];
    assert_eq!(skipped.code, IssueCode::Empty);
    assert_eq!(skipped.ref_text, "");
    assert_eq!((skipped.span.start, skipped.span.end), (0, 0));

    // Whitespace-only input is the same finding, but the span is retargeted to
    // the end of the input the caller passed in rather than to its start, so an
    // editor highlights the caret position after the blanks.
    let blank = parse("   ", None);
    assert!(blank.best.is_none());
    assert_eq!(blank.findings.skipped.len(), 1);
    let skipped = &blank.findings.skipped[0];
    assert_eq!(skipped.code, IssueCode::Empty);
    assert_eq!((skipped.span.start, skipped.span.end), (3, 3));
    assert_eq!(skipped.span.text, "");
}

#[test]
fn reports_unit_assumed_when_a_dimension_is_expected() {
    for ctx in [
        ParseCtx {
            expected_dimension: Some(Dimension::Length),
            ..ParseCtx::default()
        },
        ParseCtx {
            expect: Some(Kind::Quantity),
            ..ParseCtx::default()
        },
    ] {
        let parsed = parse("3640", Some(ctx.clone()));
        let ambiguity = parsed
            .findings
            .ambiguities
            .iter()
            .find(|found| found.code == IssueCode::UnitAssumed)
            .unwrap_or_else(|| panic!("no UNIT_ASSUMED for {ctx:?}"));
        assert_eq!(ambiguity.ref_text, "3640");
        assert_eq!(ambiguity.candidate_count, Some(2));

        // The hint does not force the reading: the unitless number still wins
        // and the assumed millimetre length is offered alongside it.
        let best = parsed.best.as_ref().expect("best reading");
        assert_eq!(best.kind, Kind::Number);
        assert_eq!(best.unit, None);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.alternatives[0].unit.as_deref(), Some("mm"));
        assert_eq!(parsed.alternatives[0].dimension, Some(Dimension::Length));
    }

    // Without a hint there is nothing to assume, and no finding.
    let bare = parse("3640", None);
    assert!(bare.findings.ambiguities.is_empty(), "{:?}", bare.findings);
}

/// The cup is the crate's locale-dependent unit, and which one wins is decided
/// by [`ParseCtx::locale`]. Nothing else pins that mapping, so a change to the
/// ranking would otherwise silently convert a British recipe with US cups.
#[test]
fn reports_ambiguous_unit_and_picks_the_cup_by_locale() {
    for (locale, winner) in [
        (None, 0.473_176_473),
        (Some(Locale::En), 0.473_176_473),
        (Some(Locale::EnUs), 0.473_176_473),
        (Some(Locale::EnGb), 0.568_261_25),
    ] {
        let parsed = parse(
            "2 cups",
            Some(ParseCtx {
                locale: locale.clone(),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.as_ref().expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("L"), "{locale:?}");
        assert_eq!(best.dimension, Some(Dimension::Volume), "{locale:?}");
        assert_eq!(best.value, Some(winner), "{locale:?}");

        // All three cups stay on the table; the parser does not commit.
        assert_eq!(parsed.alternatives.len(), 2, "{locale:?}");
        let mut offered: Vec<f64> = parsed
            .alternatives
            .iter()
            .filter_map(|reading| reading.value)
            .chain(best.value)
            .collect();
        offered.sort_by(f64::total_cmp);
        assert_eq!(
            offered,
            vec![0.473_176_473, 0.5, 0.568_261_25],
            "{locale:?}"
        );

        assert_eq!(parsed.findings.ambiguities.len(), 1, "{locale:?}");
        let ambiguity = &parsed.findings.ambiguities[0];
        assert_eq!(ambiguity.code, IssueCode::AmbiguousUnit, "{locale:?}");
        assert_eq!(ambiguity.ref_text, "cups", "{locale:?}");
        assert_eq!(ambiguity.candidate_count, Some(3), "{locale:?}");
    }
}

/// `IssueCode::UnknownUnit` is documented as reserved — "no parse produces this
/// code today". This sweep is what forces that doc to be updated if a grammar
/// ever starts emitting it.
#[test]
fn unknown_unit_is_never_emitted() {
    for input in unknown_unit_sweep_corpus() {
        for ctx in [
            None,
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
            Some(ParseCtx {
                expected_dimension: Some(Dimension::Length),
                ..ParseCtx::default()
            }),
        ] {
            assert_no_unknown_unit(&parse(&input, ctx.clone()), &input);
            for found in parse_all(&input, ctx) {
                assert_no_unknown_unit(&found.parsed, &input);
            }
        }
    }
}

fn assert_no_unknown_unit(parsed: &Parsed, input: &str) {
    for skipped in &parsed.findings.skipped {
        assert_ne!(skipped.code, IssueCode::UnknownUnit, "{input:?}");
    }
    for ambiguity in &parsed.findings.ambiguities {
        assert_ne!(ambiguity.code, IssueCode::UnknownUnit, "{input:?}");
    }
    for approximation in &parsed.findings.approximations {
        assert_ne!(approximation.code, IssueCode::UnknownUnit, "{input:?}");
    }
}

/// Inputs biased towards unit-like tokens the registry does not know, which is
/// exactly where an `UNKNOWN_UNIT` would appear if anything emitted it.
fn unknown_unit_sweep_corpus() -> Vec<String> {
    let mut corpus: Vec<String> = Vec::new();
    for unit in [
        "zorkmid", "xyzzy", "furlong", "smoot", "meterz", "kgg", "kg", "㍇", "cups", "$", "€",
        "°X", "qq", "",
    ] {
        for number in ["", "5", "-3", "1,234", "1.234", "0"] {
            corpus.push(format!("{number} {unit}"));
            corpus.push(format!("{number}{unit}"));
            corpus.push(format!("about {number} {unit}"));
            corpus.push(format!("{number} {unit} to m"));
            corpus.push(format!("{number}-{number} {unit}"));
        }
    }
    for extra in [
        "5 blorks and 3 kg",
        "幅3640ミリ",
        "3pm Europe/Paris",
        "every second tuesday of the month",
        "5尺3寸",
        "９９９ｍｍ",
        "1e400",
        "\u{feff}5 kg",
    ] {
        corpus.push(extra.to_owned());
    }
    corpus
}

/// The `(severity, rank, recoverable)` classification of every `IssueCode`.
///
/// This is a stable contract, not an implementation detail: all three fields
/// are surfaced through `ranked_findings` — and from there through the JSON
/// envelope and the web adapters — so a UI branches on them to decide what to
/// block on, what to sort to the top, and whether a usable reading survives.
/// Moving a code to a different severity class, a different rank tier, or
/// flipping its recoverability changes what callers render, so every one of the
/// thirteen codes is pinned here rather than only the handful a live parse
/// happens to produce today.
#[test]
fn every_issue_code_keeps_its_severity_rank_and_recoverability() {
    // code, severity, rank, recoverable
    let table: [(IssueCode, IssueSeverity, u16, bool); 13] = [
        (IssueCode::Empty, IssueSeverity::Error, 100, false),
        (IssueCode::NoValue, IssueSeverity::Error, 100, false),
        (IssueCode::UnknownUnit, IssueSeverity::Error, 80, true),
        (
            IssueCode::TimezoneUnsupported,
            IssueSeverity::Error,
            90,
            true,
        ),
        (
            IssueCode::RecurrenceUnsupported,
            IssueSeverity::Error,
            90,
            true,
        ),
        (IssueCode::RejectedByPolicy, IssueSeverity::Error, 90, true),
        (IssueCode::TypoCorrected, IssueSeverity::Warning, 65, true),
        (IssueCode::AmbiguousNumber, IssueSeverity::Warning, 55, true),
        (IssueCode::AmbiguousDate, IssueSeverity::Warning, 55, true),
        (IssueCode::AmbiguousUnit, IssueSeverity::Warning, 55, true),
        (
            IssueCode::AmbiguousCurrency,
            IssueSeverity::Warning,
            55,
            true,
        ),
        (IssueCode::UnitAssumed, IssueSeverity::Info, 40, true),
        (IssueCode::Approximation, IssueSeverity::Warning, 30, true),
    ];

    // Every variant appears exactly once, so a new code cannot be added without
    // being classified here.
    let mut codes: Vec<&str> = table.iter().map(|row| row.0.as_str()).collect();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), table.len());

    for (code, severity, rank, recoverable) in table {
        // A finding carrying the code is pushed through the real flattening
        // path, which is where a UI reads these three fields from.
        let mut parsed = parse("5 kg", None);
        parsed.findings = Findings::default();
        parsed.findings.skipped.push(Skipped {
            code,
            ref_text: "x".to_owned(),
            reason: "pinned".to_owned(),
            span: Span {
                start: 0,
                end: 1,
                text: "5".to_owned(),
            },
        });

        let issues = ranked_findings(&parsed);
        assert_eq!(issues.len(), 1, "{code:?}");
        assert_eq!(issues[0].code, code, "{code:?}");
        assert_eq!(issues[0].severity, severity, "{code:?}");
        assert_eq!(issues[0].rank, rank, "{code:?}");
        assert_eq!(issues[0].recoverable, recoverable, "{code:?}");
        // The rank band the doc comment on `RankedIssue::rank` promises.
        assert!((30..=100).contains(&issues[0].rank), "{code:?}");
    }
}
