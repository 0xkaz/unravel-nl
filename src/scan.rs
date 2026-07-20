use crate::*;

pub(crate) fn push_clause_matches(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
) {
    match broad_clause_dispatch(&text[start..end]) {
        BroadClauseDispatch::None => {
            if !push_clause_range_match(matches, text, start, end, ctx, false) {
                push_numeric_window_matches(matches, text, start, end, ctx);
            }
            return;
        }
        BroadClauseDispatch::Prefix => {
            match push_broad_clause_match(matches, text, start, end, ctx) {
                Some(true) => return,
                Some(false) if clause_has_numeric_candidate(text, start, end) => {
                    matches.pop();
                }
                Some(false) => return,
                None => {}
            }
            if !push_clause_range_match(matches, text, start, end, ctx, true) {
                push_numeric_window_matches(matches, text, start, end, ctx);
            }
            return;
        }
        BroadClauseDispatch::Short => {}
    }

    let mut first_numeric = None;
    let mut numeric_count = 0usize;
    for_numeric_candidate_spans(text, start, end, |candidate| {
        numeric_count += 1;
        if first_numeric.is_none() {
            first_numeric = Some(candidate);
        }
        true
    });

    let clause_bounds = trimmed_bounds(text, start, end);
    if numeric_count == 1
        && first_numeric.map(|candidate| (candidate.start, candidate.end)) == clause_bounds
    {
        // Via the window scan, so that a window abandoned for its numeric core
        // still gets its tail re-scanned: `"3 apples4"` is one candidate that
        // covers the whole clause, and the `4` lives past the core.
        push_numeric_window_matches(matches, text, start, end, ctx);
        return;
    }

    match push_broad_clause_match(matches, text, start, end, ctx) {
        Some(true) => return,
        Some(false) if numeric_count > 0 => {
            matches.pop();
        }
        _ => {}
    }

    // Both arms used to differ, the single-candidate one pushing `first_numeric`
    // directly. They agree on the candidate — the same scan produced it — but
    // only the window scan re-scans the tail of a window that was abandoned for
    // its numeric core, which is where `"1 and2 kg"` hid its `2 kg`.
    if first_numeric.is_some() && !push_clause_range_match(matches, text, start, end, ctx, true) {
        push_numeric_window_matches(matches, text, start, end, ctx);
    }
}

/// Re-reads a clause as one range before the token scan splits it into digits.
///
/// The token scan reads a clause one numeric window at a time, and a range
/// separator the window grammar does not cross ends the window: `"工期2〜3日"`
/// offers `2` and `3` as two unrelated candidates, and `"10 ± 0.5 mm"` offers
/// `10` and `0.5 mm`. Each of those candidates parses, so nothing looks lost —
/// but [`parse`] reads the same clause as a single interval, and the two halves
/// it decayed into say something the text never did.
///
/// The span retried is the clause with any leading label removed (from the
/// first numeric candidate to the end of the clause), because the label is what
/// stops the whole-clause broad read: `parse("面積100-120㎡")` finds nothing,
/// `parse("100-120㎡")` finds the range. `clause_broad_tried` says the caller
/// already put the whole clause through the broad grammar, so an unlabelled
/// clause is not parsed a second time.
///
/// Only a range whose endpoints carry a unit is taken, and only a range: this
/// reading *overrides* narrower readings that parsed on their own, so it has to
/// be more than a guess about which digits belong together. `"add 1 to 2"` does
/// read as an interval from 1 to 2, but bare numbers either side of a `to` are
/// exactly the guess this must not make, and its `1 to` (one tonne) and `2`
/// stay as they were. Contrast [`push_numeric_candidate_match_resume`], where
/// the window had no reading at all and any range is strictly more than the
/// nothing it would otherwise report.
pub(crate) fn push_clause_range_match(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
    clause_broad_tried: bool,
) -> bool {
    let Some((clause_start, clause_end)) = trimmed_bounds(text, start, end) else {
        return false;
    };
    let mut first_numeric = None;
    for_numeric_candidate_spans(text, start, end, |candidate| {
        first_numeric = Some(candidate.start);
        false
    });
    let Some(hull_start) = first_numeric.map(|candidate| candidate.max(clause_start)) else {
        return false;
    };
    if hull_start >= clause_end || (clause_broad_tried && hull_start == clause_start) {
        return false;
    }
    let Some(hull) = text.get(hull_start..clause_end) else {
        return false;
    };
    if !span_may_be_range(hull) {
        return false;
    }
    push_range_reading_match(matches, text, hull_start, clause_end, ctx, true)
}

/// Whether a span holds a two-endpoint range separator, before parsing it.
///
/// A pre-filter for the two retries that reach for the broad grammar, so that
/// text with no separator in it never pays for a second parse. It is
/// deliberately narrower than the range grammar — one-sided forms such as
/// `"under 5kg"` are not listed, since neither retry exists to rescue those —
/// so a span it rejects is only left to the readings the scan already had.
pub(crate) fn span_may_be_range(span: &str) -> bool {
    span.contains(['±', '〜', '～', '-'])
        || span.contains("..")
        || find_ascii_case(span, " to ").is_some()
        || span.contains("+/-")
}

/// Pushes `start..end` read by the broad grammar, but only if it is a range.
///
/// Returns whether it was pushed. `require_dimensioned_endpoints` demands that
/// both ends carry a unit, a dimension, or a date, which is what separates an
/// interval the text states from two adjacent numbers.
pub(crate) fn push_range_reading_match(
    matches: &mut Vec<ParsedMatch>,
    source: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
    require_dimensioned_endpoints: bool,
) -> bool {
    if start >= end
        || matches
            .last()
            .is_some_and(|item| item.start == start && item.end == end)
    {
        return false;
    }
    let Some(text) = source.get(start..end).map(str::trim) else {
        return false;
    };
    if text.is_empty() {
        return false;
    }
    let parsed = parse(text, Some(ctx.clone()));
    if !parsed_reads_as_range(&parsed, require_dimensioned_endpoints) {
        return false;
    }
    let leading = source[start..end].len() - source[start..end].trim_start().len();
    let trailing = source[start..end].len() - source[start..end].trim_end().len();
    matches.push(ParsedMatch {
        start: start + leading,
        end: end - trailing,
        text: text.to_owned(),
        parsed,
    });
    true
}

pub(crate) fn parsed_reads_as_range(parsed: &Parsed, require_dimensioned_endpoints: bool) -> bool {
    let Some(range) = parsed
        .best
        .as_ref()
        .filter(|best| best.kind == Kind::Range)
        .and_then(|best| best.range.as_ref())
    else {
        return false;
    };
    !require_dimensioned_endpoints
        || [&range.from, &range.to].into_iter().all(|endpoint| {
            endpoint.unit.is_some() || endpoint.dimension.is_some() || endpoint.date.is_some()
        })
}

pub(crate) fn push_broad_clause_match(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
) -> Option<bool> {
    push_parsed_match(
        matches,
        text,
        CandidateSpan {
            start,
            end,
            numeric_core_end: None,
            parser: CandidateParser::Broad,
        },
        ctx,
    )
}

pub(crate) fn push_numeric_window_matches(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
) {
    for_numeric_candidate_steps(text, start, end, |candidate| {
        match push_numeric_candidate_match_resume(matches, text, candidate, ctx) {
            // The window was abandoned for its numeric core, so everything the
            // speculation absorbed past that core is unread text: scanning
            // resumes there instead of after the window.
            Some(core_end) => CandidateStep::ResumeAt(core_end),
            None => CandidateStep::Continue,
        }
    });
}

/// Pushes a numeric candidate window, falling back to its numeric core.
///
/// [`candidate_window`] speculates: after a number it will cross a space and keep
/// consuming as long as what follows *could* be a unit, so `"1 and 2"` offers
/// the window `"1 and"`. When the speculation is wrong the window does not
/// parse, and before this fallback existed the scanner resumed *after* the
/// failed window — so the `1` that reads perfectly well on its own was dropped
/// with nothing on any findings channel to say so.
///
/// A window the token grammar cannot read is offered to the broad grammar
/// first, and taken if that reads it as a range. The token grammar is quantity
/// then number, so a window is all one value to it and an interval inside one
/// window has no reading: `"100-120㎡"`, `"5-10kg"` and `"3pm-4pm"` are single
/// windows that [`parse`] reads completely and that used to leave `parse_all`
/// with nothing at all, on no findings channel. Nothing narrower was competing
/// — the window had failed outright — so any range it reads is taken, including
/// one between bare numbers such as `"10-20"`.
///
/// The second retry is the number the window started from, without the text it
/// speculatively absorbed ([`CandidateSpan::numeric_core_end`]). It only exists
/// when the window actually crossed such a gap, so a window that never
/// speculated and is no range either is still all-or-nothing, and a window that
/// parses (`"1 in"` as one inch, `"5 kg"`) never reaches either retry at all.
///
/// The return value is where scanning has to resume. `Some(core_end)` means the
/// window was abandoned for its core, so the text the speculation absorbed past
/// that core was never read by any candidate: resuming after the window would
/// drop it, which is how `"1 and2 kg"` recovered its `1` and still lost the
/// whole `2 kg`, with nothing on any findings channel to say so. `None` means
/// the window was taken as it stands — it parsed, or it had no narrower reading
/// to fall back to — and scanning continues after it.
pub(crate) fn push_numeric_candidate_match_resume(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    candidate: CandidateSpan,
    ctx: &ParseCtx,
) -> Option<usize> {
    if push_parsed_match(matches, text, candidate, ctx).is_some() {
        return None;
    }
    if text
        .get(candidate.start..candidate.end)
        .is_some_and(span_may_be_range)
        && push_range_reading_match(matches, text, candidate.start, candidate.end, ctx, false)
    {
        return None;
    }
    let core_end = candidate.numeric_core_end?;
    if core_end >= candidate.end {
        return None;
    }
    // The narrower span is what the scanner settles for, whether or not it
    // parses: either way the window itself was rejected, so the text past the
    // core still has to be offered its own candidates.
    let _ = push_parsed_match(
        matches,
        text,
        CandidateSpan {
            end: core_end,
            numeric_core_end: None,
            ..candidate
        },
        ctx,
    );
    Some(core_end)
}

pub(crate) fn sorted_non_overlapping_matches(mut matches: Vec<ParsedMatch>) -> Vec<ParsedMatch> {
    if matches.len() <= 1 {
        return matches;
    }

    matches.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
    });

    let mut non_overlapping: Vec<ParsedMatch> = Vec::with_capacity(matches.len());
    for candidate in matches {
        if non_overlapping.last().is_some_and(|existing| {
            spans_overlap(existing.start, existing.end, candidate.start, candidate.end)
        }) {
            continue;
        }
        non_overlapping.push(candidate);
    }
    non_overlapping
}

pub(crate) fn push_parsed_match(
    matches: &mut Vec<ParsedMatch>,
    source: &str,
    candidate: CandidateSpan,
    ctx: &ParseCtx,
) -> Option<bool> {
    let start = candidate.start;
    let end = candidate.end;
    if start >= end
        || matches
            .last()
            .is_some_and(|item| item.start == start && item.end == end)
    {
        return None;
    }
    let text = source.get(start..end).map(str::trim)?;
    if text.is_empty() {
        return None;
    }
    let parsed = match candidate.parser {
        CandidateParser::Broad => parse(text, Some(ctx.clone())),
        CandidateParser::TokenWindow => parse_token_window(text, ctx),
    };
    if !parsed_has_actionable_match(&parsed) {
        return None;
    }
    let suppresses_inner_tokens = parsed_suppresses_inner_tokens(&parsed);
    let leading = source[start..end].len() - source[start..end].trim_start().len();
    let trailing = source[start..end].len() - source[start..end].trim_end().len();
    matches.push(ParsedMatch {
        start: start + leading,
        end: end - trailing,
        text: text.to_owned(),
        parsed,
    });
    Some(suppresses_inner_tokens)
}

pub(crate) fn parsed_suppresses_inner_tokens(parsed: &Parsed) -> bool {
    parsed.best.is_some()
        || !parsed.alternatives.is_empty()
        || !parsed.findings.ambiguities.is_empty()
        || !parsed.findings.approximations.is_empty()
        || parsed.findings.skipped.iter().any(|issue| {
            matches!(
                issue.code,
                IssueCode::Approximation
                    | IssueCode::TypoCorrected
                    | IssueCode::TimezoneUnsupported
                    | IssueCode::RecurrenceUnsupported
            )
        })
}

pub(crate) fn push_editor_dimension_match(
    matches: &mut Vec<ParsedMatch>,
    source: &str,
    candidate: CandidateSpan,
    clause_start: usize,
    ctx: &ParseCtx,
) {
    let start = candidate.start;
    let end = candidate.end;
    if start >= end
        || matches
            .last()
            .is_some_and(|item| item.start == start && item.end == end)
    {
        return;
    }
    let Some(text) = source.get(start..end).map(str::trim) else {
        return;
    };
    if text.is_empty() {
        return;
    }

    let hint = editor_dimension_hint(source, clause_start, start);
    if hint.is_none() && candidate_has_identifier_prefix(source, clause_start, start) {
        return;
    }
    let local_ctx_storage;
    let local_ctx = if ctx.expected_dimension.is_none() {
        if let Some(hint) = hint {
            let mut updated = ctx.clone();
            updated.expected_dimension = Some(hint);
            local_ctx_storage = updated;
            &local_ctx_storage
        } else {
            ctx
        }
    } else {
        ctx
    };
    let mut parsed = parsed_shell(text, local_ctx);
    parse_editor_dimension_into(text, local_ctx, &mut parsed);
    retarget_findings_to_input(&mut parsed);
    if !parsed_is_editor_dimension(&parsed, hint, ctx.expected_dimension) {
        return;
    }

    let leading = source[start..end].len() - source[start..end].trim_start().len();
    let trailing = source[start..end].len() - source[start..end].trim_end().len();
    matches.push(ParsedMatch {
        start: start + leading,
        end: end - trailing,
        text: text.to_owned(),
        parsed,
    });
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CandidateSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
    /// End of the number the window started from, when the window then crossed
    /// a space to speculatively absorb a possible unit; `None` when it never
    /// speculated. See [`push_numeric_candidate_match_resume`], which retries this
    /// narrower span when the whole window fails to parse.
    pub(crate) numeric_core_end: Option<usize>,
    pub(crate) parser: CandidateParser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateParser {
    Broad,
    TokenWindow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BroadClauseDispatch {
    None,
    Short,
    Prefix,
}

pub(crate) fn broad_clause_dispatch(clause: &str) -> BroadClauseDispatch {
    let trimmed = clause.trim();
    if trimmed.is_empty() {
        return BroadClauseDispatch::None;
    }
    if trimmed.starts_with('約') {
        return BroadClauseDispatch::Prefix;
    }
    if strip_prefix_ascii_case(trimmed, "between ").is_some()
        || strip_prefix_ascii_case(trimmed, "from ").is_some()
        || strip_prefix_ascii_case(trimmed, "about ").is_some()
        || strip_prefix_ascii_case(trimmed, "around ").is_some()
        || strip_prefix_ascii_case(trimmed, "roughly ").is_some()
        || strip_prefix_ascii_case(trimmed, "approximately ").is_some()
    {
        BroadClauseDispatch::Prefix
    } else if has_at_most_three_words(trimmed) {
        BroadClauseDispatch::Short
    } else {
        BroadClauseDispatch::None
    }
}

pub(crate) fn has_at_most_three_words(text: &str) -> bool {
    text.split_whitespace().take(4).count() <= 3
}

pub(crate) fn clause_has_numeric_candidate(text: &str, start: usize, end: usize) -> bool {
    let mut found = false;
    for_numeric_candidate_spans(text, start, end, |_| {
        found = true;
        false
    });
    found
}

pub(crate) fn for_clause_spans<F>(text: &str, mut emit: F)
where
    F: FnMut(usize, usize),
{
    let mut start = 0;
    for (idx, ch) in text.char_indices() {
        if is_clause_separator(text, idx, ch) {
            if start < idx {
                emit(start, idx);
            }
            start = idx + ch.len_utf8();
        }
    }
    if start < text.len() {
        emit(start, text.len());
    }
}

pub(crate) fn trimmed_bounds(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let span = text.get(start..end)?;
    let leading = span.len() - span.trim_start().len();
    let trailing = span.len() - span.trim_end().len();
    let trimmed_start = start + leading;
    let trimmed_end = end - trailing;
    (trimmed_start < trimmed_end).then_some((trimmed_start, trimmed_end))
}

pub(crate) fn is_clause_separator(text: &str, idx: usize, ch: char) -> bool {
    match ch {
        '、' | ';' | '；' | '\n' | '\t' => true,
        ',' => {
            let previous = text[..idx].chars().rev().find(|ch| !ch.is_whitespace());
            let next = text[idx + ch.len_utf8()..]
                .chars()
                .find(|ch| !ch.is_whitespace());
            !matches!((previous, next), (Some(left), Some(right)) if left.is_ascii_digit() && right.is_ascii_digit())
        }
        _ => false,
    }
}

pub(crate) fn for_numeric_candidate_spans<F>(text: &str, start: usize, end: usize, mut emit: F)
where
    F: FnMut(CandidateSpan) -> bool,
{
    for_numeric_candidate_steps(text, start, end, |candidate| {
        if emit(candidate) {
            CandidateStep::Continue
        } else {
            CandidateStep::Stop
        }
    });
}

/// What the scanner does after a candidate window was offered.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CandidateStep {
    /// Resume after the window, which the consumer took as it stands.
    Continue,
    /// Stop scanning; the consumer has what it needs.
    Stop,
    /// Resume at this offset, because the consumer only used the window up to
    /// there and the rest of it is still unread text. Always clamped so the
    /// cursor advances, so a consumer cannot spin the scan.
    ResumeAt(usize),
}

pub(crate) fn for_numeric_candidate_steps<F>(text: &str, start: usize, end: usize, mut emit: F)
where
    F: FnMut(CandidateSpan) -> CandidateStep,
{
    let mut cursor = start;
    while cursor < end {
        let Some((idx, ch)) = text[cursor..end].char_indices().next() else {
            break;
        };
        let abs = cursor + idx;
        if is_candidate_start_at(text, abs, ch) {
            let (candidate_end, numeric_core_end) = candidate_window(text, abs, end);
            let mut next = candidate_end;
            if candidate_end > abs {
                match emit(CandidateSpan {
                    start: abs,
                    end: candidate_end,
                    numeric_core_end,
                    parser: CandidateParser::TokenWindow,
                }) {
                    CandidateStep::Continue => {}
                    CandidateStep::Stop => return,
                    CandidateStep::ResumeAt(resume) => next = resume,
                }
            }
            cursor = next.max(abs + ch.len_utf8());
        } else {
            cursor = abs + ch.len_utf8();
        }
    }
}

pub(crate) fn parse_token_window(text: &str, ctx: &ParseCtx) -> Parsed {
    let quantity = parse_quantity_fast_with_ctx(text, ctx);
    if parsed_has_actionable_match(&quantity) {
        return quantity;
    }
    parse_number_fast_with_ctx(text, ctx)
}

pub(crate) fn parsed_has_actionable_match(parsed: &Parsed) -> bool {
    parsed.best.is_some()
        || !parsed.alternatives.is_empty()
        || !parsed.suggestions.is_empty()
        || !parsed.findings.ambiguities.is_empty()
        || !parsed.findings.approximations.is_empty()
        || parsed
            .findings
            .skipped
            .iter()
            .any(|issue| !matches!(issue.code, IssueCode::NoValue | IssueCode::UnknownUnit))
}

pub(crate) fn candidate_starts_with_currency(text: &str, start: usize) -> bool {
    text[start..]
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '$' | '€' | '£' | '¥' | '￥'))
}

pub(crate) fn parsed_is_editor_dimension(
    parsed: &Parsed,
    hint: Option<Dimension>,
    expected_dimension: Option<Dimension>,
) -> bool {
    let allowed_dimension = expected_dimension.or(hint);
    if let Some(best) = parsed.best.as_ref() {
        if reading_is_dimension_quantity(best, allowed_dimension) {
            return true;
        }
        if best.kind == Kind::Number {
            return allowed_dimension == Some(Dimension::Length)
                && parsed
                    .alternatives
                    .iter()
                    .any(|reading| reading.dimension == Some(Dimension::Length));
        }
    }
    parsed
        .alternatives
        .iter()
        .any(|reading| reading_is_dimension_quantity(reading, allowed_dimension))
}

pub(crate) fn candidate_has_identifier_prefix(
    source: &str,
    clause_start: usize,
    candidate_start: usize,
) -> bool {
    source
        .get(clause_start..candidate_start)
        .and_then(|before| before.chars().next_back())
        .is_some_and(is_embedded_identifier_char)
}

pub(crate) fn is_embedded_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric() || matches!(ch, 'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９')
}

pub(crate) fn reading_is_dimension_quantity(
    reading: &Reading,
    expected_dimension: Option<Dimension>,
) -> bool {
    if reading.kind != Kind::Quantity {
        return false;
    }
    match reading.dimension {
        Some(Dimension::Length | Dimension::Area) => match expected_dimension {
            Some(dimension) => reading.dimension == Some(dimension),
            None => true,
        },
        _ => false,
    }
}

/// Non-whitespace characters of context every label test can possibly read.
///
/// The widest label is `"wall thickness"` — 13 non-whitespace characters — and
/// [`ascii_label_suffix_matches`] also looks at the one character in front of
/// the match. 16 leaves margin over that 14, and every other test in
/// [`editor_label_matches`] reads strictly less. Because every test is a suffix
/// test, a window holding this much context answers exactly what the whole
/// clause prefix would; see `editor_label_window`.
pub(crate) const EDITOR_LABEL_CONTEXT_CHARS: usize = 16;

pub(crate) fn is_editor_label_separator(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, ':' | '：' | '=' | '＝' | '-' | 'ー' | '―' | '–' | '—')
}

/// Returns the tail of the clause prefix that the label tests can see.
///
/// Every test in [`editor_label_matches`] is a suffix test on this string (or
/// on its whitespace-stripped form), so only the last
/// [`EDITOR_LABEL_CONTEXT_CHARS`] non-whitespace characters can affect the
/// answer. Slicing the whole prefix instead — and lowercasing it — made
/// [`crate::parse_dimensions_for_editor`] quadratic in clause length, because
/// each of the clause's candidates re-read everything in front of it.
pub(crate) fn editor_label_window(
    source: &str,
    clause_start: usize,
    candidate_start: usize,
) -> Option<&str> {
    let trimmed = source
        .get(clause_start..candidate_start)?
        .trim_end_matches(is_editor_label_separator);

    let mut window_start = trimmed.len();
    let mut seen = 0usize;
    for (idx, ch) in trimmed.char_indices().rev() {
        window_start = idx;
        if !ch.is_whitespace() {
            seen += 1;
            if seen == EDITOR_LABEL_CONTEXT_CHARS {
                break;
            }
        }
    }
    trimmed.get(window_start..)
}

pub(crate) fn editor_dimension_hint(
    source: &str,
    clause_start: usize,
    candidate_start: usize,
) -> Option<Dimension> {
    let before = editor_label_window(source, clause_start, candidate_start)?;
    let lower = ascii_lower_cow(before);
    let mut compact = None;

    for (label, dimension) in EDITOR_DIMENSION_LABELS {
        if editor_label_matches(before, lower.as_ref(), &mut compact, label) {
            return Some(*dimension);
        }
    }
    None
}

pub(crate) fn editor_label_matches(
    before: &str,
    lower_before: &str,
    compact: &mut Option<String>,
    label: &str,
) -> bool {
    if label.len() == 1 && label.as_bytes()[0].is_ascii_alphabetic() {
        let trimmed = before.trim_end();
        let Some((idx, ch)) = trimmed.char_indices().next_back() else {
            return false;
        };
        return ch.eq_ignore_ascii_case(&char::from(label.as_bytes()[0]))
            && trimmed[..idx]
                .chars()
                .next_back()
                .is_none_or(|previous| !previous.is_ascii_alphanumeric());
    }
    if matches!(label, "area" | "width" | "height" | "depth" | "length") {
        return ascii_label_suffix_matches(lower_before, label);
    }
    if let Some(spaced_label) = compound_editor_label(label) {
        if ascii_label_suffix_matches(lower_before, spaced_label)
            || ascii_label_suffix_matches(lower_before, label)
        {
            return true;
        }
        let compact = compact.get_or_insert_with(|| {
            lower_before
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect()
        });
        return ascii_label_suffix_matches(compact, label);
    }
    lower_before.ends_with(label)
}

pub(crate) fn compound_editor_label(label: &str) -> Option<&'static str> {
    match label {
        "floorarea" => Some("floor area"),
        "sitearea" => Some("site area"),
        "wallthickness" => Some("wall thickness"),
        _ => None,
    }
}

pub(crate) fn ascii_label_suffix_matches(lower_before: &str, label: &str) -> bool {
    let lower = lower_before.trim_end();
    if !lower.ends_with(label) {
        return false;
    }
    let prefix = &lower[..lower.len() - label.len()];
    prefix
        .chars()
        .next_back()
        .is_none_or(|previous| !previous.is_ascii_alphanumeric())
}

pub(crate) fn is_candidate_start_at(text: &str, idx: usize, ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(ch, '０'..='９' | '$' | '€' | '£' | '¥' | '￥')
        || is_cjk_number_char(ch)
        || (ch == '約'
            && text[idx + ch.len_utf8()..]
                .chars()
                .next()
                .is_some_and(is_candidate_number_start))
}

pub(crate) fn is_candidate_number_start(ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(ch, '０'..='９' | '$' | '€' | '£' | '¥' | '￥')
        || is_cjk_number_char(ch)
}

/// Returns the candidate window and, when the window speculated across a space,
/// the end of the number it started from.
///
/// The second value is exactly the `end` this scan had reached the moment
/// before it took the "space, then something that could start a unit" branch —
/// the point past which everything in the window is a guess. It is `None` when
/// that branch never fired, so a window such as `"3pm-4pm"`, which never
/// crosses a space, offers no narrower reading to fall back to.
pub(crate) fn candidate_window(text: &str, start: usize, limit: usize) -> (usize, Option<usize>) {
    let mut end = start;
    let mut numeric_core_end = None;
    let mut saw_unit = false;
    let mut saw_number = false;
    let mut previous_was_digit = false;
    let mut after_number_gap = false;
    // Memoizes the first non-space character at or after `resolved_at`. Every
    // space in one whitespace run resolves to the same character, so the run is
    // scanned once instead of once per space (which made this loop O(w^2)).
    // `resolved_at = 0` is never reachable as a real answer — the loop only
    // consults this past a space, so the offset asked about is always above
    // zero — which makes it a free "nothing memoized yet".
    let mut resolved_at = 0usize;
    let mut resolved_char = None;

    for (idx, ch) in text[start..limit].char_indices() {
        let abs = start + idx;
        if idx > 0 && is_candidate_boundary(text, abs, ch) {
            break;
        }
        if idx == 0 && ch == '約' {
            end = abs + ch.len_utf8();
            continue;
        }
        if is_numeric_body_char(ch) {
            saw_number = true;
            previous_was_digit = is_digit_like(ch);
            after_number_gap = false;
            end = abs + ch.len_utf8();
            continue;
        }
        if is_candidate_space(ch) {
            let after = abs + ch.len_utf8();
            // Everything in `[after, resolved_at)` is a space when the memo
            // still applies, so a later space in the same run resolves to the
            // character the run's first space already found. Neither test below
            // consults it unless one of these two flags holds, and the original
            // short-circuited the same way, so a space that ends the candidate
            // still costs nothing.
            if (previous_was_digit || (saw_number && !saw_unit)) && after > resolved_at {
                (resolved_at, resolved_char) = first_nonspace_from(text, after, limit);
            }
            if previous_was_digit && resolved_char.is_some_and(is_digit_like) {
                end = after;
                continue;
            }
            if saw_number && !saw_unit && resolved_char.is_some_and(is_candidate_unit_char) {
                after_number_gap = true;
                numeric_core_end.get_or_insert(end);
                end = after;
                continue;
            }
            break;
        }
        if saw_number && is_candidate_unit_char(ch) {
            saw_unit = true;
            previous_was_digit = false;
            after_number_gap = false;
            end = abs + ch.len_utf8();
            continue;
        }
        if after_number_gap {
            break;
        }
        if idx == 0 && matches!(ch, '$' | '€' | '£' | '¥' | '￥') {
            end = abs + ch.len_utf8();
            continue;
        }
        break;
    }

    // Same result as popping trailing whitespace characters one at a time, in
    // one pass instead of one pass per character removed.
    end = start + text[start..end].trim_end_matches(char::is_whitespace).len();

    if !saw_number {
        return (start, None);
    }
    (end, numeric_core_end.filter(|core| *core < end))
}

pub(crate) fn is_digit_like(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '０'..='９') || is_cjk_number_char(ch)
}

pub(crate) fn is_numeric_body_char(ch: char) -> bool {
    is_digit_like(ch)
        || matches!(
            ch,
            '.' | ',' | '+' | '-' | '．' | '，' | '万' | '億' | '兆' | '/' | '／'
        )
}

pub(crate) fn is_candidate_space(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '\u{00A0}' | '\u{202F}' | '\u{2009}' | '\u{2007}')
}

/// Finds the first non-space character at or after `cursor`, before `limit`.
///
/// Returns the offset it was found at (or `limit`) and the character itself.
/// Callers memoize the result across one whitespace run: every space in a run
/// resolves to the same character, so the run only has to be walked once.
pub(crate) fn first_nonspace_from(
    text: &str,
    mut cursor: usize,
    limit: usize,
) -> (usize, Option<char>) {
    while cursor < limit {
        let Some(ch) = text[cursor..limit].chars().next() else {
            return (limit, None);
        };
        if is_candidate_space(ch) {
            cursor += ch.len_utf8();
            continue;
        }
        return (cursor, Some(ch));
    }
    (limit, None)
}

pub(crate) fn is_candidate_unit_char(ch: char) -> bool {
    ch.is_ascii_alphabetic()
        || matches!(ch, 'Ａ'..='Ｚ' | 'ａ'..='ｚ')
        || matches!(
            ch,
            'μ' | 'µ'
                | '°'
                | '%'
                | '/'
                | '^'
                | '²'
                | '³'
                | '₂'
                | '尺'
                | '寸'
                | '間'
                | '帖'
                | '畳'
                | '坪'
                | '平'
                | '米'
                | '㎡'
                | '円'
                | '度'
                | 'キ'
                | 'ロ'
                | 'グ'
                | 'ラ'
                | 'ム'
                | '公'
                | '斤'
                | '千'
                | '克'
                | 'リ'
                | 'ッ'
                | 'ト'
                | 'ル'
                | '半'
        )
}

pub(crate) fn is_candidate_boundary(text: &str, idx: usize, ch: char) -> bool {
    if matches!(ch, '、' | ';' | '；' | '\n' | '\t' | '(' | ')' | '[' | ']') {
        return true;
    }
    if matches!(ch, '×' | '*') {
        return text[idx + ch.len_utf8()..]
            .chars()
            .find(|next| !next.is_whitespace())
            .is_some_and(is_candidate_number_start);
    }
    false
}

pub(crate) fn spans_overlap(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    left_start < right_end && right_start < left_end
}

pub(crate) fn parse_editor_dimension_into(text: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();

    if is_editor_plain_number_candidate(trimmed) {
        parse_editor_dimension_number_into(trimmed, ctx, parsed);
        return;
    }

    parse_quantity_fast_into(trimmed, ctx, parsed);
    if parsed_is_editor_dimension(parsed, ctx.expected_dimension, ctx.expected_dimension) {
        return;
    }

    parse_editor_dimension_number_into(trimmed, ctx, parsed);
}

pub(crate) fn parse_editor_dimension_number_into(text: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    let expected_dimension = ctx.expected_dimension.unwrap_or(Dimension::Length);
    let mut number = parsed_shell(text, ctx);
    if let Some(ambiguous) = parse_ambiguous_number(text, ctx) {
        number.best = ambiguous.best;
        number.alternatives = ambiguous.alternatives;
        number.findings.ambiguities.push(ambiguous.ambiguity);
    } else if let Some(reading) = parse_plain_number_ctx(text, ctx) {
        set_editor_plain_number_result(text, expected_dimension, reading, &mut number);
    } else {
        number
            .findings
            .skipped
            .push(skipped(text, "no supported number matched"));
    }
    finalize_parsed(text, &mut number);

    if parsed_is_editor_dimension(&number, Some(expected_dimension), Some(expected_dimension)) {
        // `number` was built from the normalized text, so adopting it wholesale
        // would replace the original input the caller handed in — and the spans
        // are translated against that original afterwards.
        let input = std::mem::take(&mut parsed.input);
        *parsed = number;
        parsed.input = input;
        return;
    }

    parsed.best = None;
    parsed.alternatives.clear();
    parsed.suggestions.clear();
    parsed.findings = Findings::default();
    parsed
        .findings
        .skipped
        .push(skipped(text, "no supported editor dimension matched"));
}

pub(crate) fn set_editor_plain_number_result(
    text: &str,
    expected_dimension: Dimension,
    reading: Reading,
    parsed: &mut Parsed,
) {
    if expected_dimension == Dimension::Length {
        parsed.alternatives.push(Reading::quantity(
            reading.value.unwrap_or_default(),
            "mm",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.41,
        ));
        parsed.findings.ambiguities.push(ambiguity(
            text,
            "Plain number could be unitless or a context-implied millimeter length.",
            Some(2),
            IssueCode::UnitAssumed,
        ));
    }
    parsed.best = Some(reading);
}

pub(crate) fn is_editor_plain_number_candidate(text: &str) -> bool {
    let mut saw_number = false;
    for ch in text.chars() {
        if is_digit_like(ch) {
            saw_number = true;
            continue;
        }
        if matches!(
            ch,
            '.' | ','
                | '+'
                | '-'
                | '．'
                | '，'
                | '/'
                | '／'
                | '万'
                | '億'
                | '兆'
                | ' '
                | '_'
                | '\''
                | '\u{00A0}'
                | '\u{202F}'
                | '\u{2009}'
        ) {
            continue;
        }
        return false;
    }
    saw_number
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The scanner as it read before the whitespace run was resolved in one
    /// pass: a fresh forward walk over the rest of the run for every space.
    /// Kept as an oracle so "faster" stays provably "same answer".
    fn candidate_end_reference(text: &str, start: usize, limit: usize) -> (usize, Option<usize>) {
        fn next_nonspace_is(
            text: &str,
            mut cursor: usize,
            limit: usize,
            accept: fn(char) -> bool,
        ) -> bool {
            while cursor < limit {
                let Some(ch) = text[cursor..limit].chars().next() else {
                    return false;
                };
                if is_candidate_space(ch) {
                    cursor += ch.len_utf8();
                    continue;
                }
                return accept(ch);
            }
            false
        }

        let mut end = start;
        let mut numeric_core_end = None;
        let mut saw_unit = false;
        let mut saw_number = false;
        let mut previous_was_digit = false;
        let mut after_number_gap = false;

        for (idx, ch) in text[start..limit].char_indices() {
            let abs = start + idx;
            if idx > 0 && is_candidate_boundary(text, abs, ch) {
                break;
            }
            if idx == 0 && ch == '約' {
                end = abs + ch.len_utf8();
                continue;
            }
            if is_numeric_body_char(ch) {
                saw_number = true;
                previous_was_digit = is_digit_like(ch);
                after_number_gap = false;
                end = abs + ch.len_utf8();
                continue;
            }
            if is_candidate_space(ch) {
                if previous_was_digit
                    && next_nonspace_is(text, abs + ch.len_utf8(), limit, is_digit_like)
                {
                    end = abs + ch.len_utf8();
                    continue;
                }
                if saw_number
                    && !saw_unit
                    && next_nonspace_is(text, abs + ch.len_utf8(), limit, is_candidate_unit_char)
                {
                    after_number_gap = true;
                    if numeric_core_end.is_none() {
                        numeric_core_end = Some(end);
                    }
                    end = abs + ch.len_utf8();
                    continue;
                }
                break;
            }
            if saw_number && is_candidate_unit_char(ch) {
                saw_unit = true;
                previous_was_digit = false;
                after_number_gap = false;
                end = abs + ch.len_utf8();
                continue;
            }
            if after_number_gap {
                break;
            }
            if idx == 0 && matches!(ch, '$' | '€' | '£' | '¥' | '￥') {
                end = abs + ch.len_utf8();
                continue;
            }
            break;
        }

        while end > start
            && text[start..end]
                .chars()
                .last()
                .is_some_and(char::is_whitespace)
        {
            let Some((idx, _)) = text[start..end].char_indices().last() else {
                break;
            };
            end = start + idx;
        }

        if !saw_number {
            return (start, None);
        }
        (end, numeric_core_end.filter(|core| *core < end))
    }

    #[test]
    fn candidate_end_matches_the_per_space_walk() {
        let mut inputs = vec![
            String::from("5 m"),
            String::from("5  m"),
            String::from("5\u{00A0}\u{202F} m"),
            String::from("1 234 567"),
            String::from("5 \t m"),
            String::from("5 \n m"),
            String::from("5 "),
            String::from("5   "),
            String::from("約 5 m"),
            String::from("$ 5"),
            String::from("5 m 3 kg"),
            String::from("3 × 4 m"),
            String::from("5   ×   4"),
            String::from("5 、m"),
            String::from("１２ ｍ"),
            String::from("5   3   m"),
        ];
        for width in [0usize, 1, 2, 3, 7, 64, 300] {
            let spaces = " ".repeat(width);
            for tail in ["m", "", "3", "kg", "\tm", "、", "×4", "-"] {
                inputs.push(format!("5{spaces}{tail}"));
                inputs.push(format!("5.5{spaces}{tail}"));
                inputs.push(format!("約5{spaces}{tail}"));
            }
        }

        for text in &inputs {
            for start in 0..text.len() {
                if !text.is_char_boundary(start) {
                    continue;
                }
                assert_eq!(
                    candidate_window(text, start, text.len()),
                    candidate_end_reference(text, start, text.len()),
                    "{text:?} at {start}"
                );
            }
        }
    }

    #[test]
    fn long_whitespace_run_reads_as_the_short_one() {
        let short = parse_all("5 m", None);
        for width in [1usize, 2, 500, 4000, 16000] {
            let text = format!("5{}m", " ".repeat(width));
            let long = parse_all(&text, None);
            assert_eq!(long.len(), short.len(), "width {width}");
            assert_eq!(long[0].text, text, "width {width}");
            let reading = long[0].parsed.best.as_ref().expect("reading");
            let expected = short[0].parsed.best.as_ref().expect("reading");
            assert_eq!(reading.kind, expected.kind, "width {width}");
            assert_eq!(reading.unit, expected.unit, "width {width}");
            assert_eq!(reading.dimension, expected.dimension, "width {width}");
            assert_eq!(reading.value, expected.value, "width {width}");
            // The whole run belongs to the match, exactly as one space does.
            assert_eq!(long[0].start, 0, "width {width}");
            assert_eq!(long[0].end, text.len(), "width {width}");
        }
    }

    /// The core is offered exactly when the window crossed a space on the guess
    /// that a unit followed, and it always ends the leading number.
    #[test]
    fn numeric_core_is_offered_only_for_speculative_windows() {
        for (text, expected) in [
            ("1 and", (5usize, Some(1usize))),
            ("4 apples", (8, Some(1))),
            ("3.5 metres", (10, Some(3))),
            ("1 in", (4, Some(1))),
            ("5 kg", (4, Some(1))),
            // No space, so nothing was ever guessed at.
            ("3pm-4pm", (7, None)),
            ("100-120", (7, None)),
            ("3m", (2, None)),
            ("1,234", (5, None)),
            // Digit-space-digit is a grouped number, not a guessed unit.
            ("1 234 567", (9, None)),
            // The run of spaces resolves to one gap, and the core sits before it.
            ("12   kgg", (8, Some(2))),
            // A space with nothing unit-like after it just ends the window.
            ("5 3", (3, None)),
            ("5   ", (1, None)),
        ] {
            assert_eq!(candidate_window(text, 0, text.len()), expected, "{text:?}");
            if let Some(core) = expected.1 {
                assert!(core < expected.0, "{text:?}");
                assert!(text[..core].chars().any(is_digit_like), "{text:?}");
                assert!(!text[..core].ends_with(char::is_whitespace), "{text:?}");
            }
        }
    }

    /// The hint as it read before the inspected window was bounded: the whole
    /// clause prefix, sliced and lowercased for every candidate.
    fn editor_dimension_hint_reference(
        source: &str,
        clause_start: usize,
        candidate_start: usize,
    ) -> Option<Dimension> {
        let before = source.get(clause_start..candidate_start)?.trim_end();
        let before = before
            .trim_end_matches(|ch: char| {
                ch.is_whitespace()
                    || matches!(ch, ':' | '：' | '=' | '＝' | '-' | 'ー' | '―' | '–' | '—')
            })
            .trim_end();
        let lower = ascii_lower_cow(before);
        let mut compact = None;

        for (label, dimension) in EDITOR_DIMENSION_LABELS {
            if editor_label_matches(before, lower.as_ref(), &mut compact, label) {
                return Some(*dimension);
            }
        }
        None
    }

    #[test]
    fn editor_label_window_never_changes_which_label_wins() {
        let mut prefixes: Vec<String> = Vec::new();
        for label in EDITOR_DIMENSION_LABELS {
            let label = label.0;
            let spaced = compound_editor_label(label).unwrap_or(label);
            for lead in ["", "x", "0", "床", "Total ", "3m ", "note:", "の"] {
                for tail in ["", ":", "：", " = ", "  ", "ー", " -- ", "—"] {
                    prefixes.push(format!("{lead}{label}{tail}"));
                    prefixes.push(format!("{lead}{spaced}{tail}"));
                    prefixes.push(format!("{lead}{}{tail}", label.to_uppercase()));
                    // Padding in front must not reach the answer.
                    prefixes.push(format!("{}{lead}{label}{tail}", "3m ".repeat(40)));
                    prefixes.push(format!("{}{lead}{spaced}{tail}", "a".repeat(40)));
                }
            }
        }
        prefixes.push(String::new());
        prefixes.push(String::from("   "));
        prefixes.push(String::from("wall   thickness "));
        prefixes.push(String::from("floor    area:"));
        prefixes.push(String::from("swidth "));
        prefixes.push(String::from("3width "));
        prefixes.push(String::from("aw "));
        prefixes.push(String::from("1w "));

        for prefix in &prefixes {
            let source = format!("{prefix}3m");
            let candidate_start = prefix.len();
            assert_eq!(
                editor_dimension_hint(&source, 0, candidate_start),
                editor_dimension_hint_reference(&source, 0, candidate_start),
                "{prefix:?}"
            );
        }
    }

    #[test]
    fn many_editor_candidates_read_like_a_few() {
        let small = parse_dimensions_for_editor(&"3m ".repeat(4), None);
        assert_eq!(small.len(), 4);

        for count in [8usize, 256, 2048] {
            let matches = parse_dimensions_for_editor(&"3m ".repeat(count), None);
            assert_eq!(matches.len(), count, "count {count}");
            for (index, item) in matches.iter().enumerate() {
                assert_eq!(item.start, index * 3, "count {count} index {index}");
                assert_eq!(item.end, index * 3 + 2, "count {count} index {index}");
                assert_eq!(item.text, small[0].text, "count {count} index {index}");
                let reading = item.parsed.best.as_ref().expect("reading");
                let expected = small[0].parsed.best.as_ref().expect("reading");
                assert_eq!(reading.unit, expected.unit, "count {count} index {index}");
                assert_eq!(reading.value, expected.value, "count {count} index {index}");
                assert_eq!(
                    reading.dimension, expected.dimension,
                    "count {count} index {index}"
                );
            }
        }
    }

    #[test]
    fn labelled_editor_clause_keeps_its_hint_at_any_offset() {
        // The label sits at the very start; later candidates are far past the
        // bounded window and must still be read against it or against nothing,
        // exactly as the full-prefix scan decided.
        let source = format!("幅：{}", "3m ".repeat(64));
        let matches = parse_dimensions_for_editor(&source, None);
        let reference: Vec<Option<Dimension>> = matches
            .iter()
            .map(|item| editor_dimension_hint_reference(&source, 0, item.start))
            .collect();
        let actual: Vec<Option<Dimension>> = matches
            .iter()
            .map(|item| editor_dimension_hint(&source, 0, item.start))
            .collect();
        assert_eq!(actual, reference);
    }
}
