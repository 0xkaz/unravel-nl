mod support;
use support::parse;

use unravel_nl::{IssueCode, IssueSeverity, ParseCtx, Strictness, ranked_findings};

#[test]
fn ranked_findings_orders_blocking_issues_first() {
    let parsed = parse("3pm Europe/Paris", None);
    let issues = ranked_findings(&parsed);

    assert_eq!(issues[0].code, IssueCode::TimezoneUnsupported);
    assert_eq!(issues[0].severity, IssueSeverity::Error);
    assert_eq!(issues[0].rank, 90);
    assert!(issues[0].recoverable);
    assert_eq!(issues[0].span.start, 4);
    assert_eq!(issues[0].span.end, 16);
}

#[test]
fn ranked_findings_keeps_approximation_recoverable_for_ui() {
    let parsed = parse(
        "about 20kg",
        Some(ParseCtx {
            strictness: Strictness::Strict,
            ..ParseCtx::default()
        }),
    );
    let issues = ranked_findings(&parsed);

    assert_eq!(issues[0].code, IssueCode::Approximation);
    assert_eq!(issues[0].severity, IssueSeverity::Warning);
    assert_eq!(issues[0].rank, 30);
    assert!(issues[0].recoverable);
}

#[test]
fn severity_has_stable_adapter_strings() {
    assert_eq!(IssueSeverity::Info.as_str(), "info");
    assert_eq!(IssueSeverity::Warning.as_str(), "warning");
    assert_eq!(IssueSeverity::Error.as_str(), "error");
}
