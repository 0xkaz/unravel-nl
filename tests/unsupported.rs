//! Recurrence input is refused, and the refusal is on the findings channel.
//!
//! The crate used to canonicalize `every monday` into an RRULE string. That
//! surface is gone — see the "Recurrence" sections of both READMEs for why —
//! and this file pins the one property its removal had to preserve: a
//! recurrence phrase is not *silently* dropped. It comes back with no reading
//! and a stated reason, exactly as any other string the parser cannot read
//! does.
//!
//! A crate that reads `毎週月曜` as nothing and says nothing is worse than one
//! that never read it, because the caller cannot tell the two apart.

mod support;
use support::parse;

use unravel_nl::{IssueCode, ParseCtx, ParsePurpose};

/// Every shape the deleted grammar used to accept now refuses, with a reason.
#[test]
fn recurrence_phrases_are_refused_with_a_stated_reason() {
    for input in [
        // English phrases the deleted grammar read.
        "every monday",
        "every day",
        "daily",
        "monthly",
        "every 2 weeks",
        "every other monday",
        "monthly on the 15th",
        "monthly on the second monday",
        "every month on the last friday",
        "every third business day",
        "every sixth business day",
        "every monday for 3 times",
        // Japanese and Chinese phrases it read.
        "毎日",
        "毎週月曜",
        "毎週月曜日",
        "毎月",
        "毎月15日",
        "毎月第2月曜日",
        "毎月第3営業日",
        "毎月第六月曜日",
        "每周一",
        // Raw RRULE strings it accepted verbatim.
        "FREQ=DAILY",
        "FREQ=MONTHLY",
        "FREQ=WEEKLY;BYDAY=MO",
        "FREQ=MONTHLY;BYDAY=2MO",
        "FREQ=MONTHLY;BYSETPOS=3;BYDAY=MO,TU,WE,TH,FR",
        // The multibyte shapes that used to panic a byte split.
        "FREQ=MONTHLY;BYDAY=あ",
        "FREQ=MONTHLY;BYDAY=1𝍄",
    ] {
        let parsed = parse(input, None);
        assert!(
            parsed.best.is_none(),
            "{input:?} was read as {:?}",
            parsed.best
        );
        assert!(
            !parsed.findings.skipped.is_empty(),
            "{input:?} produced no reading and no finding"
        );
        assert_eq!(
            parsed.findings.skipped[0].code,
            IssueCode::NoValue,
            "{input}"
        );
        assert!(
            !parsed.findings.skipped[0].reason.is_empty(),
            "{input:?} was refused with an empty reason"
        );
    }
}

/// No purpose reads one either: the recurrence purpose is gone with the grammar.
///
/// `ParsePurpose` had a `Recurrence` variant that selected the deleted grammar.
/// Removing the variant is what makes "there is no recurrence entry point"
/// checkable at compile time; this pins that the remaining purposes did not
/// quietly inherit the behaviour.
#[test]
fn no_surviving_purpose_reads_a_recurrence() {
    for purpose in [
        ParsePurpose::General,
        ParsePurpose::Quantity,
        ParsePurpose::Number,
        ParsePurpose::Date,
        ParsePurpose::DimensionEditor,
    ] {
        for input in ["every monday", "毎週月曜", "FREQ=DAILY"] {
            let parsed = parse(
                input,
                Some(ParseCtx {
                    purpose,
                    ..ParseCtx::default()
                }),
            );
            assert!(
                parsed.best.is_none(),
                "{purpose:?} read {input:?} as {:?}",
                parsed.best
            );
            assert!(
                !parsed.findings.skipped.is_empty(),
                "{purpose:?} dropped {input:?} without a finding"
            );
        }
    }
}

/// Dates, times, durations and clock slots are a different question, and still
/// read. The recurrence removal is not allowed to take them with it.
#[test]
fn dates_times_and_durations_are_untouched() {
    for input in ["3pm-4pm", "1h30", "PT1H30M", "3pm", "14:30", "2d4h"] {
        let parsed = parse(input, None);
        assert!(
            parsed.best.is_some(),
            "{input:?} stopped reading: {:?}",
            parsed.findings
        );
    }
}
