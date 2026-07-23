//! Finding spans must address the original input, byte for byte.
//!
//! Dispatch runs on a normalized copy of the input, and normalization changes
//! byte lengths (`'０'` 3→1, `'㎞'` 3→2, a zero-width space 3→0), so a span
//! taken straight from the normalized text points somewhere else in — or into
//! the middle of a character of — what the caller actually holds. Every
//! assertion here is written against `parsed.input`, which is the string a
//! consumer highlights.

mod support;
use support::{
    parse, parse_date_fast, parse_dimensions_for_editor, parse_number_fast, parse_quantity_fast,
};

use unravel_nl::{
    Dimension, DimensionSet, IssueCode, Kind, Locale, ParseCtx, Parsed, Strictness, ranked_findings,
};

/// The contract, checked for every finding: the span is a slice of the input.
fn assert_spans_address_input(parsed: &Parsed, label: &str) {
    let input = parsed.input.as_str();
    let spans = parsed
        .findings
        .skipped
        .iter()
        .map(|issue| &issue.span)
        .chain(parsed.findings.ambiguities.iter().map(|issue| &issue.span))
        .chain(
            parsed
                .findings
                .approximations
                .iter()
                .map(|issue| &issue.span),
        )
        // `ranked_findings` republishes the same spans, so it inherits the
        // contract; checking it here keeps the flattened view covered too.
        .cloned()
        .chain(ranked_findings(parsed).into_iter().map(|issue| issue.span));

    for span in spans {
        assert!(
            input.is_char_boundary(span.start),
            "{label}: start {} is not a char boundary of {input:?}",
            span.start
        );
        assert!(
            input.is_char_boundary(span.end),
            "{label}: end {} is not a char boundary of {input:?}",
            span.end
        );
        assert!(
            span.start <= span.end,
            "{label}: span {}..{} runs backwards",
            span.start,
            span.end
        );
        assert_eq!(
            input.get(span.start..span.end),
            Some(span.text.as_str()),
            "{label}: span {}..{} does not slice {input:?}",
            span.start,
            span.end
        );
    }
}

fn ja() -> Option<ParseCtx> {
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    })
}

#[test]
fn full_width_digit_does_not_shift_a_timezone_span() {
    // `'３'` is three bytes and normalizes to one, so the unfixed span 4..16
    // sliced "m Europe/Par" out of the original.
    let parsed = parse("３pm Europe/Paris", None);
    assert_spans_address_input(&parsed, "３pm Europe/Paris");

    let issue = parsed
        .findings
        .skipped
        .iter()
        .find(|issue| issue.code == IssueCode::TimezoneUnsupported)
        .expect("timezone finding");
    assert_eq!((issue.span.start, issue.span.end), (6, 18));
    assert_eq!(issue.span.text, "Europe/Paris");
    assert_eq!(issue.ref_text, "Europe/Paris");
}

#[test]
fn full_width_letters_keep_span_offsets_on_char_boundaries() {
    // Every character here is three bytes wide but one byte after
    // normalization, so the unfixed span 2..8 landed inside two of them and
    // slicing the input panicked.
    let parsed = parse("５ ｍｅｔｅｒｚ", None);
    assert_spans_address_input(&parsed, "５ ｍｅｔｅｒｚ");

    let issue = parsed
        .findings
        .ambiguities
        .iter()
        .find(|issue| issue.code == IssueCode::TypoCorrected)
        .expect("typo finding");
    assert_eq!((issue.span.start, issue.span.end), (4, 22));
    assert_eq!(issue.span.text, "ｍｅｔｅｒｚ");
    // The fragment as the user typed it, not the ASCII form the parser read.
    assert_eq!(issue.ref_text, "ｍｅｔｅｒｚ");
    // The reading itself is untouched by the span fix.
    assert_eq!(parsed.best.as_ref().expect("reading").value, Some(5.0));
}

#[test]
fn ambiguity_ref_text_quotes_the_input_as_typed() {
    let parsed = parse("１,２３４", None);
    assert_spans_address_input(&parsed, "１,２３４");

    let issue = parsed
        .findings
        .ambiguities
        .iter()
        .find(|issue| issue.code == IssueCode::AmbiguousNumber)
        .expect("ambiguous number finding");
    assert_eq!((issue.span.start, issue.span.end), (0, 13));
    assert_eq!(issue.span.text, "１,２３４");
    assert_eq!(issue.ref_text, "１,２３４");
    assert_eq!(issue.candidate_count, Some(2));
    // Still the same two readings; only where they point changed.
    assert_eq!(parsed.best.as_ref().expect("reading").value, Some(1234.0));
    assert_eq!(parsed.alternatives.len(), 1);
}

#[test]
fn every_length_changing_normalization_class_keeps_spans_honest() {
    // (input, expected span text) — one case per rewrite that moves bytes.
    for (input, expected) in [
        // Full-width digit: 3 bytes to 1.
        ("３ meterz", "meterz"),
        // Full-width letters: 3 bytes to 1, repeatedly.
        ("５ ｍｅｔｅｒｚ", "ｍｅｔｅｒｚ"),
        // Ideographic space: 3 bytes to 1.
        ("１　meterz", "meterz"),
        // No-break space: 2 bytes to 1.
        ("1\u{00a0}meterz", "meterz"),
        // Narrow no-break space: 3 bytes to 1.
        ("1\u{202f}meterz", "meterz"),
        // Zero-width space: 3 bytes to nothing.
        ("1\u{200b} meterz", "meterz"),
        ("1 \u{200b}meterz", "meterz"),
        // Byte-order mark: 3 bytes to nothing.
        ("\u{feff}1 meterz", "meterz"),
        // Full-width full stop: 3 bytes to 1.
        ("１．５ meterz", "meterz"),
    ] {
        let parsed = parse(input, None);
        assert_spans_address_input(&parsed, input);
        let issue = parsed
            .findings
            .ambiguities
            .iter()
            .find(|issue| issue.code == IssueCode::TypoCorrected)
            .unwrap_or_else(|| panic!("typo finding for {input:?}"));
        assert_eq!(issue.span.text, expected, "{input:?}");
        assert_eq!(
            &input[issue.span.start..issue.span.end],
            expected,
            "{input:?}"
        );
    }

    // Compatibility units expand instead of shrinking: `'㎞'` is 3 bytes and
    // normalizes to the 2 bytes of "km", `'㎏'` likewise to "kg". The unfixed
    // whole-input span therefore stopped one byte short of the real end.
    for input in [
        "5㎞ 3 cups",
        "5㎏ 3 cups",
        "3 ㎞ Europe/Paris",
        "1㎎ meterz",
    ] {
        let parsed = parse(input, ja());
        assert_spans_address_input(&parsed, input);
        let issue = parsed.findings.skipped.first().expect("skipped finding");
        assert_eq!(
            (issue.span.start, issue.span.end),
            (0, input.len()),
            "{input:?}"
        );
        assert_eq!(issue.span.text, input, "{input:?}");
    }

    // A finding that starts after a compatibility unit still points past it.
    let parsed = parse("約 5 ㎞", ja());
    assert_spans_address_input(&parsed, "約 5 ㎞");
    let issue = parsed
        .findings
        .approximations
        .first()
        .expect("approximation finding");
    assert_eq!((issue.span.start, issue.span.end), (0, 3));
    assert_eq!(issue.span.text, "約");
    assert_eq!(parsed.best.as_ref().expect("reading").value, Some(5000.0));
}

#[test]
fn combining_marks_survive_untouched() {
    // Combining marks are not part of the normalization table, so the identity
    // path has to stay byte-exact rather than, say, folding to NFC.
    for input in [
        "e\u{0301}5 meterz",
        "1 meterz\u{0301}",
        "cafe\u{0301} 5 meterz",
        "e\u{0301}xtraordinary",
        "５ e\u{0301} meterz",
    ] {
        let parsed = parse(input, None);
        assert_spans_address_input(&parsed, input);
        assert_eq!(parsed.input, input);
    }
}

#[test]
fn leading_whitespace_shifts_spans_past_the_original_spaces() {
    // `.trim()` runs after normalization, so both the trim and the rewrite have
    // to be undone before a span can address the input.
    let parsed = parse("  ３m", None);
    assert_spans_address_input(&parsed, "  ３m");
    assert!(
        parsed
            .findings
            .skipped
            .iter()
            .chain(parsed.findings.skipped.iter())
            .all(|issue| issue.span.start >= 2)
    );
    assert_eq!(parsed.best.as_ref().expect("reading").value, Some(3.0));

    // Same input shape, but with a finding to pin the offsets on.
    let parsed = parse(
        "  ３",
        Some(ParseCtx {
            expect: Some(Kind::Quantity),
            expected_dimensions: DimensionSet::from(Dimension::Length),
            ..ParseCtx::default()
        }),
    );
    assert_spans_address_input(&parsed, "  ３");
    let issue = parsed
        .findings
        .ambiguities
        .first()
        .expect("unit-assumed finding");
    assert_eq!((issue.span.start, issue.span.end), (2, 5));
    assert_eq!(issue.span.text, "３");

    let parsed = parse("  ３ ｍｅｔｅｒｚ", None);
    assert_spans_address_input(&parsed, "  ３ ｍｅｔｅｒｚ");
    let issue = parsed.findings.ambiguities.first().expect("typo finding");
    assert_eq!((issue.span.start, issue.span.end), (6, 24));
    assert_eq!(issue.span.text, "ｍｅｔｅｒｚ");

    // Trailing whitespace must not stretch the span either.
    let parsed = parse("１,２３４  ", None);
    assert_spans_address_input(&parsed, "１,２３４  ");
    let issue = parsed
        .findings
        .ambiguities
        .first()
        .expect("ambiguous number finding");
    assert_eq!((issue.span.start, issue.span.end), (0, 13));
}

#[test]
fn every_entry_point_reports_spans_against_its_own_input() {
    let corpus = [
        "３pm Europe/Paris",
        "５ ｍｅｔｅｒｚ",
        "１,２３４",
        "  ３m",
        "  ５㎏  ",
        "１０　㎞",
        "5\u{200b}０ ㎏",
        "$１２",
        "５ cups",
        "約２０ｋｇ",
        "こんにちは\u{200b}world",
        "１．５ｍ",
        "５尺３寸",
        "毎週金曜日\u{3000}",
        "０５/０６/２０２６",
        "ｆｒｏｍ １０ｋｇ ｔｏ ２ｋｇ",
        "１０ ± ３ ｍｍ",
        "\u{feff}１０００円",
        "e\u{0301}very ２ weeks",
        "",
        "   ",
        "\u{200b}",
        "　",
    ];

    for input in corpus {
        for strictness in [
            Strictness::Forgiving,
            Strictness::Confirm,
            Strictness::Strict,
        ] {
            let ctx = ParseCtx {
                locale: Some(Locale::Ja),
                strictness,
                reference_date: unravel_nl::Date::new(2026, 7, 19),
                ..ParseCtx::default()
            };

            assert_spans_address_input(&parse(input, Some(ctx.clone())), input);
            assert_spans_address_input(&parse_quantity_fast(input, Some(ctx.clone())), input);
            assert_spans_address_input(&parse_number_fast(input, Some(ctx.clone())), input);
            assert_spans_address_input(&parse_date_fast(input, Some(ctx.clone())), input);

            for found in parse_dimensions_for_editor(input, Some(ctx.clone())) {
                assert_spans_address_input(&found.parsed, input);
                assert_eq!(
                    input.get(found.start..found.end),
                    Some(found.text.as_str()),
                    "editor match does not slice {input:?}"
                );
            }
        }
    }
}
