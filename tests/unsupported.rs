use unravel_nl::{IssueCode, parse};

#[test]
fn recurrence_phrases_fail_loudly_until_adapter_exists() {
    for (input, ref_text) in [("every monday", "every"), ("毎週月曜日", "毎")] {
        let parsed = parse(input, None);
        assert!(parsed.best.is_none(), "{input}");
        assert_eq!(
            parsed.findings.skipped[0].code,
            IssueCode::RecurrenceUnsupported,
            "{input}"
        );
        assert_eq!(parsed.findings.skipped[0].ref_text, ref_text, "{input}");
    }
}
