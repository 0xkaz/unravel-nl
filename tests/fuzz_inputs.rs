use unravel_nl::{
    IssueCode, Locale, ParseCtx, Parsed, ParsedMatch, Reading, complete, parse,
    parse_dimensions_for_editor, ranked_findings,
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
        // Range separators, so range readings — and therefore the endpoint
        // invariants below — are actually reached.
        '~', '〜', '–', '±', 't', 'o',
        // Cup spelling, the locale-dependent unit with three candidates.
        'c', 'u', 'p', 's',
        // More currency symbols, which parse through a grammar of their own.
        '$', '£', '.',
        // A high digit, so a long run of them overflows f64 rather than merely
        // being large.
        '9',
        // Characters that have to be escaped before the input can be echoed
        // back into a JSON envelope. None of them appeared anywhere in the
        // corpus, so the emitter's backslash and control-character arms were
        // never reached from a real parse.
        '\\', '"', '\u{1}', '\n', '\t',
    ];

    for len in 0..96 {
        let mut input = String::new();
        for _ in 0..len {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push(alphabet[(seed as usize) % alphabet.len()]);
        }
        assert_invariants_hold(&input);
    }

    // Word-shaped fragments the character alphabet is unlikely to assemble by
    // chance, plus digit runs long enough to overflow to infinity (an f64
    // needs more than 308 digits before a bare integer stops being finite).
    let tokens = [
        "5",
        "3",
        "2 cups",
        " to ",
        "-",
        "~",
        "kg",
        "m",
        "$",
        "¥1,234",
        "about ",
        " and ",
        "9",
        &"9".repeat(400),
        "1.234",
        "5尺3寸",
        "20°C",
        "every tuesday",
    ];
    for len in 0..48 {
        let mut input = String::new();
        for _ in 0..len {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push_str(tokens[(seed as usize) % tokens.len()]);
        }
        assert_invariants_hold(&input);
    }

    // Hand-written shapes that exercise each invariant directly.
    for input in [
        "5 to 3 kg",
        "2 cups",
        "2-3 kg",
        "5kg to 3m",
        "3m and 20kg and ¥1,234",
        &"9".repeat(400),
        &format!("{} to {}", "9".repeat(400), "9".repeat(400)),
        "",
    ] {
        assert_invariants_hold(input);
    }
}

fn assert_invariants_hold(input: &str) {
    let ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: unravel_nl::Date::new(2026, 7, 19),
        ..ParseCtx::default()
    });
    let parsed = parse(input, ctx.clone());
    assert_spans_address_input(&parsed, input);
    assert_reading_invariants(&parsed, input);
    assert_ranked_findings_invariants(&parsed, input);

    let editor = parse_dimensions_for_editor(input, ctx);
    assert_matches_are_ordered_and_disjoint(&editor, input);
    for found in &editor {
        assert_spans_address_input(&found.parsed, input);
        assert_reading_invariants(&found.parsed, input);
    }

    assert_completion_invariants(input);
    #[cfg(feature = "wasm")]
    assert_json_envelope_escapes_the_input(input);
}

/// The envelope echoes the input verbatim, so no `Cc` character reaches the
/// document raw.
///
/// A raw `U+0000`–`U+001F`, or a backslash that swallows the closing quote,
/// makes the whole document unparseable on the JS side — not one field — and
/// those are the characters RFC 8259 actually forbids raw inside a string.
/// The sweep below is deliberately stricter: `char::is_control` is the Unicode
/// `Cc` category, which also covers `U+007F` and `U+0080`–`U+009F`, and those
/// are legal unescaped. The emitter escapes them too (see
/// `escapes_the_two_structural_characters_and_every_cc_character` in
/// `src/json_out.rs`, which pins the escaping table), so asserting the wider
/// property here is a true statement about this emitter rather than about JSON.
/// The alphabet above carries `\`, `"`, `U+0001`, newline and tab for exactly
/// this invariant.
#[cfg(feature = "wasm")]
fn assert_json_envelope_escapes_the_input(input: &str) {
    let json = unravel_nl::parse_json(input);
    for (idx, ch) in json.char_indices() {
        assert!(
            !ch.is_control(),
            "{input:?}: raw Cc character {ch:?} at {idx} in {json:?}"
        );
    }
    let echoed = json
        .split_once("\"input\":")
        .expect("an input field")
        .1
        .to_owned();
    assert_eq!(
        decode_json_string_prefix(&echoed).as_deref(),
        Some(input),
        "{input:?}: {json:?}"
    );
}

/// Decodes the JSON string literal at the start of `text`, ignoring whatever
/// follows it.
#[cfg(feature = "wasm")]
fn decode_json_string_prefix(text: &str) -> Option<String> {
    let mut chars = text.strip_prefix('"')?.chars();
    let mut out = String::new();
    loop {
        match chars.next()? {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{8}'),
                'f' => out.push('\u{c}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => {
                    let digits: String = chars.by_ref().take(4).collect();
                    out.push(char::from_u32(u32::from_str_radix(&digits, 16).ok()?)?);
                }
                _ => return None,
            },
            ch => out.push(ch),
        }
    }
}

/// `ranked_findings` sorts a mixed set of findings by rank, highest first.
///
/// A real parse reports at most one finding today, so the ordering contract is
/// unfalsifiable against parser output alone. The lists are therefore assembled
/// from four separate parses into one `Parsed`, with the two ambiguities pushed
/// in *ascending* rank — `UNIT_ASSUMED` (40) before `AMBIGUOUS_UNIT` (55) — so
/// that an implementation which merely concatenated the three lists in order
/// would return them the wrong way round and fail here.
#[test]
fn ranked_findings_orders_a_mixed_finding_set_by_rank() {
    let approximation = parse("about 20kg", None);
    let ambiguous_unit = parse("2 cups", None);
    let unit_assumed = parse(
        "3640",
        Some(ParseCtx {
            expected_dimensions: unravel_nl::DimensionSet::from(unravel_nl::Dimension::Length),
            ..ParseCtx::default()
        }),
    );
    let skipped = parse("3pm Europe/Paris", None);

    let mut mixed = approximation.clone();
    mixed.findings.skipped = skipped.findings.skipped.clone();
    mixed.findings.ambiguities = unit_assumed
        .findings
        .ambiguities
        .iter()
        .chain(ambiguous_unit.findings.ambiguities.iter())
        .cloned()
        .collect();
    assert_eq!(mixed.findings.approximations.len(), 1);
    assert_eq!(mixed.findings.skipped.len(), 1);
    assert_eq!(
        mixed
            .findings
            .ambiguities
            .iter()
            .map(|issue| issue.code)
            .collect::<Vec<_>>(),
        vec![IssueCode::UnitAssumed, IssueCode::AmbiguousUnit]
    );

    let issues = ranked_findings(&mixed);
    assert_eq!(
        issues
            .iter()
            .map(|issue| (issue.code, issue.rank))
            .collect::<Vec<_>>(),
        vec![
            (IssueCode::TimezoneUnsupported, 90),
            (IssueCode::AmbiguousUnit, 55),
            (IssueCode::UnitAssumed, 40),
            (IssueCode::Approximation, 30),
        ]
    );
}

/// `ranked_findings` is the flat view a UI renders, so it must lose nothing and
/// arrive in the order it claims.
///
/// Every skipped, ambiguous, and approximate finding gets exactly one entry —
/// a dropped one is silent loss at the presentation layer — and the entries are
/// ordered by rank, highest first, so the most serious issue is at the top.
fn assert_ranked_findings_invariants(parsed: &Parsed, label: &str) {
    let issues = ranked_findings(parsed);
    let findings = &parsed.findings;
    assert_eq!(
        issues.len(),
        findings.skipped.len() + findings.ambiguities.len() + findings.approximations.len(),
        "{label:?}: ranked findings do not cover every finding"
    );

    // Ordering is checked here too, but no input in this corpus makes a single
    // `parse` report two findings at once, so this loop is a guard rather than
    // the proof; `ranked_findings_orders_a_mixed_finding_set_by_rank` below
    // builds the multi-finding case that actually exercises the sort.
    for pair in issues.windows(2) {
        assert!(
            pair[0].rank >= pair[1].rank,
            "{label:?}: ranked findings are not ordered by rank, {} before {}",
            pair[0].rank,
            pair[1].rank
        );
    }

    // The three finding lists have distinct types, so they are compared as
    // `(code, ref_text)` multisets: every finding must show up in the flat view
    // with the text it was reported against.
    let mut reported: Vec<_> = findings
        .skipped
        .iter()
        .map(|issue| (issue.code, issue.ref_text.clone()))
        .chain(
            findings
                .ambiguities
                .iter()
                .map(|issue| (issue.code, issue.ref_text.clone())),
        )
        .chain(
            findings
                .approximations
                .iter()
                .map(|issue| (issue.code, issue.ref_text.clone())),
        )
        .collect();
    let mut ranked: Vec<_> = issues
        .iter()
        .map(|issue| (issue.code, issue.ref_text.clone()))
        .collect();
    reported.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.as_str().cmp(b.0.as_str())));
    ranked.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.as_str().cmp(b.0.as_str())));
    assert_eq!(
        ranked, reported,
        "{label:?}: ranked_findings does not match the reported findings"
    );
}

/// `complete` is documented to return at most 24 candidates, scored `1.0` for
/// an exact match and otherwise between `0.6` and `1.0`, highest first. A
/// candidate outside that band or out of order breaks the picker that consumes
/// it, and an oversized list breaks the cap the doc promises.
fn assert_completion_invariants(input: &str) {
    let completions = complete(input, None);
    assert!(
        completions.len() <= 24,
        "{input:?}: {} completions exceeds the documented cap of 24",
        completions.len()
    );
    for candidate in &completions {
        assert!(
            (0.6..=1.0).contains(&candidate.score),
            "{input:?}: completion {:?} scored {} outside 0.6..=1.0",
            candidate.value,
            candidate.score
        );
    }
    for pair in completions.windows(2) {
        assert!(
            pair[0].score >= pair[1].score,
            "{input:?}: completions are not ordered by score, {} before {}",
            pair[0].score,
            pair[1].score
        );
    }
}

/// No reading ever escapes carrying a non-finite value, and no range ever
/// escapes with endpoints that disagree.
///
/// A value that overflowed to infinity or collapsed to `NaN`, and a range whose
/// endpoints landed on different dimensions or units, are both documented as
/// withdrawn and reported as `IssueCode::NoValue` rather than returned — so if
/// one shows up here the withdrawal missed a path, and a consumer doing
/// arithmetic on `best.value` inherits the poison.
fn assert_reading_invariants(parsed: &Parsed, label: &str) {
    for reading in parsed.best.iter().chain(parsed.alternatives.iter()) {
        assert_reading_is_usable(reading, label);
    }

    // `candidate_count` counts the readings the parser weighed, which is at
    // least the ones it returned. It is deliberately *not* equal to
    // `alternatives.len()`: a descending range such as `5 to 3 kg` reports two
    // candidates with one reading and no alternatives, because the second
    // candidate is the swapped reading the parser refused to invent.
    let returned = usize::from(parsed.best.is_some()) + parsed.alternatives.len();
    for ambiguity in &parsed.findings.ambiguities {
        if let Some(count) = ambiguity.candidate_count {
            assert!(
                count >= returned,
                "{label:?}: candidate_count {count} < {returned} returned readings"
            );
        }
    }
}

fn assert_reading_is_usable(reading: &Reading, label: &str) {
    if let Some(value) = reading.value {
        assert!(value.is_finite(), "{label:?}: non-finite value {value}");
    }
    let Some(range) = &reading.range else {
        return;
    };
    for endpoint in [&range.from, &range.to] {
        if let Some(value) = endpoint.value {
            assert!(
                value.is_finite(),
                "{label:?}: non-finite range endpoint {value}"
            );
        }
        assert!(endpoint.range.is_none(), "{label:?}: nested range");
    }
    assert_eq!(
        range.from.dimension, range.to.dimension,
        "{label:?}: range endpoints disagree on dimension"
    );
    assert_eq!(
        range.from.unit, range.to.unit,
        "{label:?}: range endpoints disagree on unit"
    );
}

/// Matches are advertised as spans into the caller's string, so an editor lays
/// them out in order. Overlapping or reordered matches would double-highlight.
fn assert_matches_are_ordered_and_disjoint(matches: &[ParsedMatch], label: &str) {
    let mut previous_end = 0usize;
    for found in matches {
        assert!(
            found.start <= found.end,
            "{label:?}: match {}..{} runs backwards",
            found.start,
            found.end
        );
        assert!(
            found.start >= previous_end,
            "{label:?}: match {}..{} overlaps a match ending at {previous_end}",
            found.start,
            found.end
        );
        assert!(
            found.end <= label.len(),
            "{label:?}: match {}..{} runs past the input",
            found.start,
            found.end
        );
        assert_eq!(
            label.get(found.start..found.end),
            Some(found.text.as_str()),
            "{label:?}: match {}..{} does not slice the input",
            found.start,
            found.end
        );
        previous_end = found.end;
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
