use crate::*;

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
    // The label hint is this function's own inference, so it never reaches
    // `ctx`: only what the caller declared may refuse a reading out loud.
    let dimensions = EditorDimensions::scanned(ctx.expected_dimensions, hint);
    let mut parsed = parsed_shell(text, ctx);
    let refused = parse_editor_dimension_into(text, ctx, dimensions, &mut parsed);
    retarget_findings_to_input(&mut parsed);
    // A candidate the caller's declared set refused is kept, so that the
    // refusal is reported rather than dropped. A candidate the label hint alone
    // filtered out refuses nothing the caller declared, and is dropped as it
    // always was.
    let mut read = text;
    if !refused && !parsed_is_editor_dimension(&parsed, dimensions) {
        // The window guessed its way across a space, and what it swallowed made
        // the whole thing unreadable. Retrying without the guess is what keeps
        // `幅3640 and 2` from throwing away the 3640 the caller typed: the value
        // is returned, and the part that beat the parser is named in the
        // findings instead of vanishing with it.
        let Some((shorter, shorter_parsed)) = read_without_the_guess(text, ctx, dimensions) else {
            return;
        };
        read = shorter;
        parsed = shorter_parsed;
        note_trailing_input(&mut parsed, text, read.len());
    }

    let matched = &source[start..end];
    let leading = matched.len() - matched.trim_start().len();
    matches.push(ParsedMatch {
        start: start + leading,
        end: start + leading + read.len(),
        text: read.to_owned(),
        parsed,
    });
}

/// Re-reads a candidate window that read as nothing, without its space guesses.
///
/// Returns the leading part that did read, and its result. The guesses are
/// dropped one at a time and widest first, so `1 234 567 apples` still reads its
/// grouped number before anything falls back to the bare `1`.
fn read_without_the_guess<'a>(
    text: &'a str,
    ctx: &ParseCtx,
    dimensions: EditorDimensions,
) -> Option<(&'a str, Parsed)> {
    let mut tried = text.len();
    for guesses in [WindowGuesses::GroupingOnly, WindowGuesses::None] {
        let cut = candidate_window_guessing(text, 0, text.len(), guesses);
        if cut == 0 || cut >= tried {
            continue;
        }
        tried = cut;
        let shorter = &text[..cut];
        let mut parsed = parsed_shell(shorter, ctx);
        let refused = parse_editor_dimension_into(shorter, ctx, dimensions, &mut parsed);
        retarget_findings_to_input(&mut parsed);
        if refused || parsed_is_editor_dimension(&parsed, dimensions) {
            return Some((shorter, parsed));
        }
    }
    None
}

/// Records the part of a candidate window that no reading covers.
///
/// `parsed` was read from `text[..read_len]` alone, so its input is widened to
/// the whole window first: a span may only address [`Parsed::input`], and the
/// residue is outside the part that was read.
fn note_trailing_input(parsed: &mut Parsed, text: &str, read_len: usize) {
    parsed.input = text.to_owned();
    let rest = &text[read_len..];
    let residue_start = read_len + (rest.len() - rest.trim_start().len());
    let residue = text[residue_start..].trim_end();
    if residue.is_empty() {
        return;
    }
    parsed.findings.skipped.push(skipped_with_span(
        residue,
        "text follows the reading and was not interpreted",
        IssueCode::TrailingInput,
        Span {
            start: residue_start,
            end: residue_start + residue.len(),
            text: residue.to_owned(),
        },
    ));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CandidateSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
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
    let mut cursor = start;
    while cursor < end {
        let Some((idx, ch)) = text[cursor..end].char_indices().next() else {
            break;
        };
        let abs = cursor + idx;
        if is_candidate_start_at(text, abs, ch) {
            let candidate_end = candidate_window(text, abs, end);
            if candidate_end > abs
                && !emit(CandidateSpan {
                    start: abs,
                    end: candidate_end,
                })
            {
                return;
            }
            cursor = candidate_end.max(abs + ch.len_utf8());
        } else {
            cursor = abs + ch.len_utf8();
        }
    }
}

pub(crate) fn candidate_starts_with_currency(text: &str, start: usize) -> bool {
    text[start..]
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '$' | '€' | '£' | '¥' | '￥'))
}

/// The measurement domains one editor candidate may be read as.
///
/// Two independent things decide it, and they **compose rather than replace**:
///
/// - what the caller declared, in [`ParseCtx::expected_dimensions`], and
/// - what the label next to the candidate says, which this crate infers.
///
/// A dimensioned reading is accepted only where the two agree, so declaring a
/// set narrows what the label already allowed and never widens it: `面積3640`
/// is no length just because a length was declared, and `予算1234` — a budget,
/// with no dimension label at all — is no length either.
///
/// Only the declaration may *refuse* a reading, because a refusal is a
/// statement about what the caller said. A candidate the label alone rules out
/// is dropped in silence, exactly as it was before the declaration existed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EditorDimensions {
    /// What the caller declared. Empty is no restriction.
    pub(crate) declared: DimensionSet,
    /// What the label next to the candidate names, when it names anything.
    hint: Option<Dimension>,
    /// The domain a bare number stands for here, when anything makes it one.
    bare: Option<Dimension>,
}

impl EditorDimensions {
    /// The policy for a candidate [`parse_dimensions_for_editor`] found in free
    /// text, next to a label that inferred `hint`.
    ///
    /// A bare number is a dimension here only because a label says so, which is
    /// what keeps `予算1234` and an unlabelled `3640` out of the results however
    /// the field was declared.
    pub(crate) fn scanned(declared: DimensionSet, hint: Option<Dimension>) -> Self {
        Self {
            declared,
            hint,
            bare: hint,
        }
    }

    /// The policy for [`parse`] under [`ParsePurpose::DimensionEditor`], which
    /// reads the whole input and has no neighbouring label to consult.
    ///
    /// With no label, the field itself is the label: a bare number in a
    /// dimension editor is a length written without its millimetres, as it
    /// always was. A declaration that excludes lengths does not make it some
    /// other domain — there is no bare-number reading in another domain to give
    /// — it refuses it, and says so.
    pub(crate) fn declared_only(declared: DimensionSet) -> Self {
        Self {
            declared,
            hint: None,
            bare: Some(Dimension::Length),
        }
    }

    /// The domains a dimensioned reading may come from.
    ///
    /// `None` when the label and the declaration contradict each other, so that
    /// no reading is acceptable at all; an empty set is no restriction.
    pub(crate) fn accepted(self) -> Option<DimensionSet> {
        match self.hint {
            Some(hint) if !self.declared.allows(hint) => None,
            Some(hint) => Some(DimensionSet::from(hint)),
            None => Some(self.declared),
        }
    }

    /// The domain a bare number stands for here, when anything makes it one.
    pub(crate) fn bare_number(self) -> Option<Dimension> {
        self.bare
    }

    /// The domain a bare number would have been read as, and that the caller
    /// declared away.
    ///
    /// `寸法3640` in an area-only field is a millimetre length the declaration
    /// refuses; reporting that is what keeps the declaration from dropping a
    /// reading in silence.
    pub(crate) fn bare_number_refused(self) -> Option<Dimension> {
        let bare = self.bare?;
        (!self.declared.allows(bare)).then_some(bare)
    }
}

pub(crate) fn parsed_is_editor_dimension(parsed: &Parsed, dimensions: EditorDimensions) -> bool {
    // The label and the declaration contradict: nothing is acceptable, and
    // nothing was refused either, since no reading of this candidate was ever
    // one the caller could have received.
    let Some(allowed) = dimensions.accepted() else {
        return false;
    };
    if let Some(best) = parsed.best.as_ref() {
        if reading_is_dimension_quantity(best, allowed) {
            return true;
        }
        if best.kind == Kind::Number {
            // A bare number is a dimension only where something said it stands
            // for one, and the only reading it can offer is the millimetre one.
            return dimensions.bare_number() == Some(Dimension::Length)
                && allowed.allows(Dimension::Length)
                && parsed
                    .alternatives
                    .iter()
                    .any(|reading| reading.dimension == Some(Dimension::Length));
        }
    }
    parsed
        .alternatives
        .iter()
        .any(|reading| reading_is_dimension_quantity(reading, allowed))
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

pub(crate) fn reading_is_dimension_quantity(reading: &Reading, allowed: DimensionSet) -> bool {
    if reading.kind != Kind::Quantity {
        return false;
    }
    match reading.dimension {
        // The editor only ever accepts these two, whatever else is declared;
        // an empty `allowed` is no further restriction, as everywhere else.
        Some(dimension @ (Dimension::Length | Dimension::Area)) => allowed.allows(dimension),
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

/// Which whitespace-crossing guesses a candidate window is allowed to make.
///
/// A window is bounded by punctuation and by non-unit characters without
/// guessing at anything. Whitespace is the one place it has to guess, because a
/// space inside a candidate can mean two different things — a grouped number
/// (`1 234 567`) or a number and its unit (`3 m`) — and neither is decidable
/// from the space itself.
///
/// [`WindowGuesses::All`] is what the scanner offers first, and it is
/// deliberately optimistic. The narrower settings exist so that a window that
/// read as nothing can be retried without the guess that swallowed it, rather
/// than the whole candidate being dropped: see
/// [`push_editor_dimension_match`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WindowGuesses {
    /// Cross a space for a grouped number, and for a unit that may follow.
    All,
    /// Cross a space only with digits on both sides of it.
    GroupingOnly,
    /// Never cross a space.
    None,
}

impl WindowGuesses {
    fn crosses_for_grouping(self) -> bool {
        self != Self::None
    }

    fn crosses_for_unit(self) -> bool {
        self == Self::All
    }
}

/// Returns the widest candidate window starting at `start`.
///
/// The window crosses a space on the guess that a unit follows, so `"3 m"` is
/// one window rather than a bare `3`, and on the guess that digits either side
/// of a space are one grouped number. Both are guesses, and both are wrong
/// often enough — `"3640 and"`, `"3640 2"` — that the widest window is only the
/// scanner's *first* offer: [`push_editor_dimension_match`] retries with
/// [`candidate_window_guessing`] when it reads as nothing, so being optimistic
/// here costs a reading nothing.
pub(crate) fn candidate_window(text: &str, start: usize, limit: usize) -> usize {
    candidate_window_guessing(text, start, limit, WindowGuesses::All)
}

/// [`candidate_window`], with the whitespace guesses named rather than assumed.
pub(crate) fn candidate_window_guessing(
    text: &str,
    start: usize,
    limit: usize,
    guesses: WindowGuesses,
) -> usize {
    let mut end = start;
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
            let grouping = previous_was_digit && guesses.crosses_for_grouping();
            let unit = saw_number && !saw_unit && guesses.crosses_for_unit();
            if (grouping || unit) && after > resolved_at {
                (resolved_at, resolved_char) = first_nonspace_from(text, after, limit);
            }
            if grouping && resolved_char.is_some_and(is_digit_like) {
                end = after;
                continue;
            }
            if unit && resolved_char.is_some_and(is_candidate_unit_char) {
                after_number_gap = true;
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
        return start;
    }
    end
}

pub(crate) fn is_digit_like(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '０'..='９') || is_cjk_number_char(ch)
}

/// Characters that continue a numeric candidate without being digits.
///
/// Grouping is not spelled out here: it comes from
/// [`NON_SPACE_GROUP_SEPARATORS`], the same list the number parser groups by,
/// so a scanner cannot stop a candidate at a character the parser would have
/// read. The whitespace group separators are deliberately not included — this
/// scanner resolves runs of whitespace itself, in `candidate_end`.
pub(crate) fn is_numeric_body_char(ch: char) -> bool {
    is_digit_like(ch)
        || NON_SPACE_GROUP_SEPARATORS.contains(&ch)
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

/// Reads one editor-dimension candidate, and says whether it was refused.
///
/// Returns `true` when [`ParseCtx::expected_dimensions`] refused the reading —
/// the reading was a dimension, and not one of the declared ones. The refusal
/// is on `parsed` as an [`IssueCode::RejectedByPolicy`] finding either way; the
/// return value is what tells [`push_editor_dimension_match`] that dropping the
/// candidate would now be dropping a refusal, which the no-silent-loss contract
/// does not allow.
pub(crate) fn parse_editor_dimension_into(
    text: &str,
    ctx: &ParseCtx,
    dimensions: EditorDimensions,
    parsed: &mut Parsed,
) -> bool {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();

    if is_editor_plain_number_candidate(trimmed) {
        return parse_editor_dimension_number_into(trimmed, ctx, dimensions, parsed);
    }

    parse_quantity_fast_into(trimmed, ctx, parsed);
    // Enforcement reads `ctx`, never `dimensions`: the label hint filters, but
    // only the caller's own declaration refuses.
    if enforce_expected_dimensions(trimmed, ctx, parsed) {
        // Falling through to the number fallback would wipe the refusal along
        // with everything else it clears.
        return true;
    }
    if parsed_is_editor_dimension(parsed, dimensions) {
        return false;
    }

    parse_editor_dimension_number_into(trimmed, ctx, dimensions, parsed)
}

/// Reads an editor candidate as a bare number, and says whether it was refused.
pub(crate) fn parse_editor_dimension_number_into(
    text: &str,
    ctx: &ParseCtx,
    dimensions: EditorDimensions,
    parsed: &mut Parsed,
) -> bool {
    let expected_dimension = dimensions.bare_number().unwrap_or(Dimension::Length);
    let mut number = parsed_shell(text, ctx);
    // The same bare-number grammar `parse_number_fast` runs, in the same order,
    // differing only in where the millimetre alternative's dimension comes
    // from. Spelling the grammar out again here is how the editor came to drop
    // `'234` from `幅1'234` while the other entry points read it.
    parse_number_into(
        PlainNumberSink::EditorDimension(expected_dimension),
        text,
        ctx,
        &mut number,
    );

    // Judged by what made this a dimension in the first place — the label, or
    // the declaration standing in for one. Whether the caller's declaration
    // then accepts that dimension is the next question, and a candidate that
    // was never a dimension has nothing for it to refuse.
    let by_label = EditorDimensions::scanned(DimensionSet::new(), Some(expected_dimension));
    if parsed_is_editor_dimension(&number, by_label) {
        if let Some(refused) = dimensions.bare_number_refused() {
            // `寸法3640` in an area field: a millimetre length the caller
            // declared away. Dropping it here is the silent loss the
            // declaration is documented not to cause, so it is reported, and
            // the reading it refused is kept as an alternative.
            parsed.best = None;
            parsed.alternatives = number
                .alternatives
                .into_iter()
                .filter(|reading| reading_is_dimension_quantity(reading, DimensionSet::new()))
                .collect();
            parsed.suggestions.clear();
            parsed.findings = Findings::default();
            parsed.findings.skipped.push(skipped_with_span(
                text,
                &expected_dimensions_reason(refused, dimensions.declared),
                IssueCode::RejectedByPolicy,
                span(text),
            ));
            return true;
        }
        // `number` was built from the normalized text, so adopting it wholesale
        // would replace the original input the caller handed in — and the spans
        // are translated against that original afterwards.
        let input = std::mem::take(&mut parsed.input);
        *parsed = number;
        parsed.input = input;
        return false;
    }

    parsed.best = None;
    parsed.alternatives.clear();
    parsed.suggestions.clear();
    parsed.findings = Findings::default();
    parsed
        .findings
        .skipped
        .push(skipped(text, "no supported editor dimension matched"));
    false
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
        // Grouping comes from the one list of group separators, so this
        // predicate and `is_numeric_body_char` cannot disagree about where a
        // number ends.
        if is_group_separator(ch)
            || matches!(
                ch,
                '.' | ',' | '+' | '-' | '．' | '，' | '/' | '／' | '万' | '億' | '兆'
            )
        {
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
    fn candidate_end_reference(text: &str, start: usize, limit: usize) -> usize {
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
            return start;
        }
        end
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
        let short = parse_dimensions_for_editor("5 m", None);
        for width in [1usize, 2, 500, 4000, 16000] {
            let text = format!("5{}m", " ".repeat(width));
            let long = parse_dimensions_for_editor(&text, None);
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

    /// A window crosses a space only on the guess that a unit follows, and
    /// never on a digit-space-digit grouping or on a space followed by nothing
    /// unit-like.
    #[test]
    fn candidate_window_crosses_a_space_only_for_a_possible_unit() {
        for (text, expected) in [
            ("1 and", 5usize),
            ("4 apples", 8),
            ("3.5 metres", 10),
            ("1 in", 4),
            ("5 kg", 4),
            // No space, so nothing was ever guessed at.
            ("3pm-4pm", 7),
            ("100-120", 7),
            ("3m", 2),
            ("1,234", 5),
            // Digit-space-digit is a grouped number, not a guessed unit.
            ("1 234 567", 9),
            // The run of spaces resolves to one gap.
            ("12   kgg", 8),
            // A space with nothing unit-like after it just ends the window.
            ("5 3", 3),
            ("5   ", 1),
        ] {
            assert_eq!(candidate_window(text, 0, text.len()), expected, "{text:?}");
        }
    }

    /// Each whitespace guess can be switched off on its own.
    ///
    /// `All` is what the scanner offers first; the narrower settings are what a
    /// window that read as nothing is retried with, so they must shorten the
    /// window monotonically and must not change anything that never crossed a
    /// space in the first place.
    #[test]
    fn dropping_a_guess_only_ever_shortens_the_window() {
        for (text, all, grouping_only, none) in [
            ("3640 and", 8usize, 4, 4),
            ("1 234 567 apples", 16, 9, 1),
            ("3 m", 3, 1, 1),
            ("1 234 567", 9, 9, 1),
            ("5 3", 3, 3, 1),
            // No space anywhere, so no guess was ever made and all three agree.
            ("3.5m", 4, 4, 4),
            ("1,234", 5, 5, 5),
            ("100㎡", 6, 6, 6),
        ] {
            assert_eq!(candidate_window(text, 0, text.len()), all, "{text:?} all");
            for (guesses, expected) in [
                (WindowGuesses::All, all),
                (WindowGuesses::GroupingOnly, grouping_only),
                (WindowGuesses::None, none),
            ] {
                assert_eq!(
                    candidate_window_guessing(text, 0, text.len(), guesses),
                    expected,
                    "{text:?} {guesses:?}"
                );
            }
            assert!(none <= grouping_only && grouping_only <= all, "{text:?}");
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
