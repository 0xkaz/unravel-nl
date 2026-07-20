use unravel_nl::{
    Locale, ParseCtx, Parsed, complete, parse, parse_all, parse_dimensions_for_editor,
    ranked_findings,
};

#[test]
fn hostile_unicode_inputs_do_not_panic() {
    let mut seed = 0x5eed_2026_u64;
    let alphabet = [
        '0', '1', '2', '3', '4', '5', '９', '．', ',', '，', ' ', '\u{202f}', '\u{200b}', 'm', 'k',
        'g', '㎡', '尺', '寸', '平', '米', '万', '億', '円', '/', '-', '±', '毎', '週', '月', '曜',
        '日', '€', '¥', 'a', 'e', 'r', 'y',
        // Characters whose byte length changes under normalization, which is
        // exactly what makes a span drift away from the original input:
        // full-width forms collapse, compatibility units expand, and the
        // zero-width and byte-order marks vanish outright.
        '０', '５', 'ｍ', 'ｋ', 'ｇ', 'ｅ', '㎞', '㎏', '㍍', '　', '\u{00a0}', '\u{feff}', '－',
        '×', '％', '約',
        // A combining mark, which normalization must leave exactly as written.
        '\u{0301}',
    ];

    for len in 0..96 {
        let mut input = String::new();
        for _ in 0..len {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push(alphabet[(seed as usize) % alphabet.len()]);
        }

        let ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: unravel_nl::Date::new(2026, 7, 19),
            ..ParseCtx::default()
        });
        let parsed = parse(&input, ctx.clone());
        assert_spans_address_input(&parsed, &input);
        let _issues = ranked_findings(&parsed);
        for found in parse_all(&input, ctx.clone()) {
            assert_spans_address_input(&found.parsed, &input);
        }
        for found in parse_dimensions_for_editor(&input, ctx) {
            assert_spans_address_input(&found.parsed, &input);
        }
        let _completions = complete(&input, None);
    }
}

/// Every finding must point at a real slice of the input it was parsed from.
///
/// Spans are advertised for editor highlighting, so a consumer slices
/// `parsed.input` with them. An offset that is not a char boundary panics in
/// that consumer, and one that is merely shifted highlights the wrong text.
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
        );

    for span in spans {
        assert!(
            input.is_char_boundary(span.start) && input.is_char_boundary(span.end),
            "{label:?}: span {}..{} is not on char boundaries of {input:?}",
            span.start,
            span.end
        );
        assert_eq!(
            input.get(span.start..span.end),
            Some(span.text.as_str()),
            "{label:?}: span {}..{} does not slice {input:?}",
            span.start,
            span.end
        );
    }
}
