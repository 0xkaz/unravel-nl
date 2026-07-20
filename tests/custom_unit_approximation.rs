//! Regressions for approximate readings that reached callers with no finding.
//!
//! The crate promises that anything skipped, ambiguous, or approximate is
//! reported in `Findings` rather than quietly dropped, and that an empty
//! `Findings` means the whole input was consumed with no guesswork. A unit
//! definition that declares its own conversion approximate — a `CustomUnit`
//! built with `.approximate(true)`, or a registry unit such as `month` or
//! `year` — used to produce `approximate: Some(true)` with a completely empty
//! `Findings`, which is that guarantee broken.

use unravel_nl::*;

fn smoot_ctx(approximate: bool) -> ParseCtx {
    ParseCtx {
        custom_units: vec![
            CustomUnit::new(
                "smoot",
                "m",
                &["smoot", "smoots"],
                Dimension::Length,
                1.7018,
            )
            .approximate(approximate),
        ],
        ..Default::default()
    }
}

fn no_findings(findings: &Findings) -> bool {
    findings.skipped.is_empty()
        && findings.ambiguities.is_empty()
        && findings.approximations.is_empty()
}

fn assert_reported_approximate(parsed: &Parsed, ref_text: &str) {
    let best = parsed.best.as_ref().expect("a reading");
    assert_eq!(best.approximate, Some(true), "{ref_text}");
    let approximations = &parsed.findings.approximations;
    assert_eq!(approximations.len(), 1, "{ref_text}: {approximations:?}");
    assert_eq!(approximations[0].code, IssueCode::Approximation);
    assert_eq!(approximations[0].ref_text, ref_text);
    assert_eq!(approximations[0].span.text, ref_text);
    assert!(
        !parsed.findings.skipped.is_empty()
            || !parsed.findings.ambiguities.is_empty()
            || !parsed.findings.approximations.is_empty(),
        "an approximate reading must not carry empty findings"
    );
}

/// An approximate custom unit is reported, not silently approximated.
#[test]
fn approximate_custom_unit_is_reported_in_findings() {
    let text = "3 smoots";

    assert_reported_approximate(&parse(text, Some(smoot_ctx(true))), text);
    assert_reported_approximate(&parse_quantity_fast(text, Some(smoot_ctx(true))), text);

    let matches = parse_all(text, Some(smoot_ctx(true)));
    assert_eq!(matches.len(), 1);
    assert_reported_approximate(&matches[0].parsed, text);
}

/// The fix must not over-report: an exact custom unit stays finding-free.
#[test]
fn exact_custom_unit_produces_no_finding() {
    let text = "3 smoots";

    for parsed in [
        parse(text, Some(smoot_ctx(false))),
        parse_quantity_fast(text, Some(smoot_ctx(false))),
    ] {
        let best = parsed.best.as_ref().expect("a reading");
        assert_eq!(best.approximate, Some(false));
        assert!(
            no_findings(&parsed.findings),
            "exact custom unit gained a finding: {:?}",
            parsed.findings
        );
    }
}

/// The same hole existed for registry units flagged `approximate`, whose
/// calendar-derived factors (`month`, `year`) are averages.
#[test]
fn approximate_registry_unit_is_reported_in_findings() {
    for text in ["3 months", "2 years"] {
        assert_reported_approximate(&parse(text, None), text);
        assert_reported_approximate(&parse_quantity_fast(text, None), text);
    }

    // Compound quantities inherit the flag from any part, and report once.
    let text = "3 months 2 years";
    assert_reported_approximate(&parse(text, None), text);
    assert_reported_approximate(&parse_quantity_fast(text, None), text);
}

/// An exact registry unit of the same dimension gains nothing from the fix.
#[test]
fn exact_registry_unit_produces_no_finding() {
    let parsed = parse("1 fortnight", None);
    assert_eq!(parsed.best.as_ref().unwrap().approximate, Some(false));
    assert!(no_findings(&parsed.findings), "{:?}", parsed.findings);
}

/// `Reading::approximate` is authoritative; `Findings::approximations` is not a
/// substitute for it, because the explanation can land in another list. This is
/// what the [`humanize`] documentation now says.
#[test]
fn reading_approximate_and_findings_approximations_are_not_interchangeable() {
    let parsed = parse("1.5 cups", None);

    assert_eq!(parsed.best.as_ref().unwrap().approximate, Some(true));
    assert!(
        parsed.findings.approximations.is_empty(),
        "{:?}",
        parsed.findings.approximations
    );
    assert_eq!(parsed.findings.ambiguities.len(), 1);
    assert_eq!(
        parsed.findings.ambiguities[0].code,
        IssueCode::AmbiguousUnit
    );

    // And the marker is not in the rendered string either.
    let best = parsed.best.clone().unwrap();
    assert!(!humanize(&best, None).contains("approx"));
}

/// `ParseCtx::locale` leaves the reading and the findings alone, but the whole
/// `Parsed` values are not equal, because `Parsed::locale` echoes the context.
#[test]
fn locale_leaves_reading_and_findings_identical_but_not_the_whole_parsed() {
    let parse_with = |locale: Option<Locale>| {
        parse(
            "1.234",
            Some(ParseCtx {
                locale,
                ..ParseCtx::default()
            }),
        )
    };

    let baseline = parse_with(None);
    for locale in [
        Locale::Ja,
        Locale::En,
        Locale::EnUs,
        Locale::EnGb,
        Locale::Other("de-DE".to_owned()),
    ] {
        let parsed = parse_with(Some(locale.clone()));
        assert_eq!(parsed.best, baseline.best, "{locale:?}");
        assert_eq!(parsed.alternatives, baseline.alternatives, "{locale:?}");
        assert_eq!(parsed.findings, baseline.findings, "{locale:?}");

        // Not byte-identical: the echoed locale differs.
        assert_ne!(parsed, baseline, "{locale:?}");
        assert_eq!(parsed.locale, Some(locale));
    }
    assert_eq!(baseline.locale, None);
}
