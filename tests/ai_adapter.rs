use unravel_nl::{
    CanonicalizeRequest, Dimension, DimensionSet, IssueCode, ParseCtx, Strictness,
    canonicalize_values, repair_tool_call_message,
};

#[test]
fn canonicalize_values_accepts_clean_values_and_rejects_strict_assumptions() {
    let values = canonicalize_values(&[
        CanonicalizeRequest::new("height", "180cm", None),
        CanonicalizeRequest::new(
            "weight",
            "about 20kg",
            Some(ParseCtx {
                strictness: Strictness::Strict,
                ..ParseCtx::default()
            }),
        ),
        CanonicalizeRequest::new(
            "length",
            "5 meterz",
            Some(ParseCtx {
                expected_dimensions: DimensionSet::from(Dimension::Length),
                strictness: Strictness::Confirm,
                ..ParseCtx::default()
            }),
        ),
    ]);

    assert!(values[0].ok);
    assert_eq!(
        values[0].canonical.as_ref().unwrap().unit.as_deref(),
        Some("m")
    );

    assert!(!values[1].ok);
    assert!(values[1].canonical.is_none());
    assert!(
        values[1]
            .message
            .as_ref()
            .unwrap()
            .contains("[APPROXIMATION]")
    );

    assert!(!values[2].ok);
    assert_eq!(
        values[2].parsed.findings.skipped[0].code,
        IssueCode::TypoCorrected
    );
    assert!(
        values[2]
            .message
            .as_ref()
            .unwrap()
            .contains("Did you mean `m`?")
    );
}

#[test]
fn repair_tool_call_message_surfaces_timezone_policy() {
    let message = repair_tool_call_message("starts_at", "3pm Europe/Paris", None).expect("message");
    assert!(message.contains("[TIMEZONE_UNSUPPORTED]"));
    assert!(message.contains("starts_at"));
    assert!(message.contains("Europe/Paris"));
}
