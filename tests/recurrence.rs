use unravel_nl::{IssueCode, Kind, humanize, parse};

#[test]
fn parses_simple_recurrence_to_rrule() {
    for (input, expected) in [
        ("every monday", "FREQ=WEEKLY;BYDAY=MO"),
        ("毎週月曜日", "FREQ=WEEKLY;BYDAY=MO"),
        ("每周五", "FREQ=WEEKLY;BYDAY=FR"),
        ("every day", "FREQ=DAILY"),
        ("毎日", "FREQ=DAILY"),
        ("monthly", "FREQ=MONTHLY"),
    ] {
        let best = parse(input, None).best.expect(input);
        assert_eq!(best.kind, Kind::Recurrence, "{input}");
        assert_eq!(best.recurrence.as_deref(), Some(expected), "{input}");
        assert_eq!(humanize(&best, None), expected);
    }
}

#[test]
fn recurrence_round_trips_through_humanize() {
    let first = parse("every friday", None).best.expect("recurrence");
    let rendered = humanize(&first, None);
    let second = parse(&rendered, None).best.expect("rrule");
    assert_eq!(first.kind, Kind::Recurrence);
    assert_eq!(first.recurrence, second.recurrence);
}

#[test]
fn unsupported_recurrence_still_fails_loudly() {
    let parsed = parse("every other monday", None);
    assert!(parsed.best.is_none());
    assert_eq!(
        parsed.findings.skipped[0].code,
        IssueCode::RecurrenceUnsupported
    );
}
