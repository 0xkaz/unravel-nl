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
        ("every 2 weeks", "FREQ=WEEKLY;INTERVAL=2"),
        ("every 3 days", "FREQ=DAILY;INTERVAL=3"),
        ("every 4 months", "FREQ=MONTHLY;INTERVAL=4"),
        ("every other monday", "FREQ=WEEKLY;INTERVAL=2;BYDAY=MO"),
        ("every monday for 5 times", "FREQ=WEEKLY;BYDAY=MO;COUNT=5"),
        ("monthly on the 15th", "FREQ=MONTHLY;BYMONTHDAY=15"),
        ("every month on the 1st", "FREQ=MONTHLY;BYMONTHDAY=1"),
        ("monthly on the second monday", "FREQ=MONTHLY;BYDAY=2MO"),
        ("every month on the last friday", "FREQ=MONTHLY;BYDAY=-1FR"),
        ("毎月15日", "FREQ=MONTHLY;BYMONTHDAY=15"),
        ("毎月第2月曜日", "FREQ=MONTHLY;BYDAY=2MO"),
    ] {
        let best = parse(input, None).best.expect(input);
        assert_eq!(best.kind, Kind::Recurrence, "{input}");
        assert_eq!(best.recurrence.as_deref(), Some(expected), "{input}");
        assert_eq!(humanize(&best, None), expected);
    }
}

#[test]
fn recurrence_round_trips_through_humanize() {
    for input in [
        "every friday",
        "every friday for 5 times",
        "monthly on the 15th",
        "monthly on the second monday",
        "every 2 weeks",
        "every other monday",
    ] {
        let first = parse(input, None).best.expect(input);
        let rendered = humanize(&first, None);
        let second = parse(&rendered, None).best.expect("rrule");
        assert_eq!(first.kind, Kind::Recurrence, "{input} -> {rendered}");
        assert_eq!(first.recurrence, second.recurrence, "{input} -> {rendered}");
    }
}

#[test]
fn unsupported_recurrence_still_fails_loudly() {
    let parsed = parse("every third business day", None);
    assert!(parsed.best.is_none());
    assert_eq!(
        parsed.findings.skipped[0].code,
        IssueCode::RecurrenceUnsupported
    );
}
