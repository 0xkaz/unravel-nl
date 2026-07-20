//! Findings: everything the parser could not silently resolve.
//!
//! The parser never discards part of its input without saying so. Anything it
//! could not read, could not read unambiguously, or read only approximately is
//! reported through [`Findings`] alongside the value. Callers that ignore
//! findings still get a usable [`Parsed`], but they give up the guarantee that
//! nothing was quietly dropped.

use crate::*;

/// Machine-readable reason a fragment was skipped, ambiguous, or approximate.
///
/// Codes are stable strings across the FFI boundary via [`IssueCode::as_str`],
/// so UI and tool layers can branch on them without matching on prose.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueCode {
    /// The input was empty, or contained nothing but whitespace.
    Empty,
    /// The input was non-empty but no reading could be extracted from it.
    NoValue,
    /// A unit-like token was found but is not in the unit registry.
    ///
    /// Reserved: unreadable input is currently reported as
    /// [`IssueCode::NoValue`], so no parse produces this code today.
    UnknownUnit,
    /// A misspelled unit was corrected to a registry entry, e.g. `meterz` to `m`.
    TypoCorrected,
    /// No unit was written and one was inferred from context or expectation.
    UnitAssumed,
    /// The number itself has more than one plausible reading, e.g. `1.234`.
    AmbiguousNumber,
    /// The date has more than one plausible reading, e.g. `05/06/2026`.
    AmbiguousDate,
    /// The unit has more than one plausible reading, e.g. a locale-dependent cup.
    AmbiguousUnit,
    /// The currency has more than one plausible reading, e.g. a bare `$`.
    AmbiguousCurrency,
    /// A timezone was recognized but cannot be resolved in this configuration.
    TimezoneUnsupported,
    /// A recurrence phrase was recognized but is not expressible as a rule.
    RecurrenceUnsupported,
    /// A reading was found but refused by the active [`Strictness`] policy.
    RejectedByPolicy,
    /// The value is approximate, e.g. `about 20kg` or a shakkanhō conversion.
    Approximation,
}

impl IssueCode {
    /// Returns the stable `SCREAMING_SNAKE_CASE` string for this code.
    ///
    /// ```
    /// use unravel_nl::IssueCode;
    ///
    /// assert_eq!(IssueCode::UnknownUnit.as_str(), "UNKNOWN_UNIT");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "EMPTY",
            Self::NoValue => "NO_VALUE",
            Self::UnknownUnit => "UNKNOWN_UNIT",
            Self::TypoCorrected => "TYPO_CORRECTED",
            Self::UnitAssumed => "UNIT_ASSUMED",
            Self::AmbiguousNumber => "AMBIGUOUS_NUMBER",
            Self::AmbiguousDate => "AMBIGUOUS_DATE",
            Self::AmbiguousUnit => "AMBIGUOUS_UNIT",
            Self::AmbiguousCurrency => "AMBIGUOUS_CURRENCY",
            Self::TimezoneUnsupported => "TIMEZONE_UNSUPPORTED",
            Self::RecurrenceUnsupported => "RECURRENCE_UNSUPPORTED",
            Self::RejectedByPolicy => "REJECTED_BY_POLICY",
            Self::Approximation => "APPROXIMATION",
        }
    }
}

/// A byte range within the original input, with the text it covers.
///
/// The range addresses [`Parsed::input`] — the string the caller passed in, not
/// the normalized copy the parser works on. `start` and `end` are always char
/// boundaries of that string, `input[start..end]` is always valid, and it
/// always equals [`Span::text`]. Slicing the input by a span is therefore safe,
/// which is what makes spans usable for editor highlighting.
///
/// ```
/// use unravel_nl::parse;
///
/// let parsed = parse("３pm Europe/Paris", None);
/// let span = &parsed.findings.skipped[0].span;
///
/// assert_eq!(&parsed.input[span.start..span.end], span.text);
/// assert_eq!(span.text, "Europe/Paris");
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    /// Byte offset of the first byte of the fragment.
    pub start: usize,
    /// Byte offset one past the last byte of the fragment.
    pub end: usize,
    /// The fragment itself, as it appeared in the input.
    ///
    /// Written the way the user wrote it: `１,２３４` stays full-width rather
    /// than being reported as the `1,234` the parser read.
    pub text: String,
}

/// Everything the parser could not resolve silently.
///
/// An empty `Findings` means the whole input was consumed into the reading
/// with no guesswork. A non-empty one is the parser telling you exactly where
/// it had to skip, choose, or approximate.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Findings {
    /// Fragments that produced no reading at all.
    pub skipped: Vec<Skipped>,
    /// Fragments that had more than one plausible reading.
    pub ambiguities: Vec<Ambiguity>,
    /// Readings that are not exact, and how far off they may be.
    pub approximations: Vec<Approximation>,
}

/// A fragment of the input that produced no reading.
#[derive(Clone, Debug, PartialEq)]
pub struct Skipped {
    /// Why the fragment was skipped, as a stable code.
    pub code: IssueCode,
    /// The fragment that was skipped.
    pub ref_text: String,
    /// Human-readable explanation, intended for display.
    pub reason: String,
    /// Where the fragment sits in the original input.
    pub span: Span,
}

/// A fragment that had more than one plausible reading.
///
/// The reading the parser ranked first is still in [`Parsed::best`]; the
/// competing readings are in [`Parsed::alternatives`]. The parser does not
/// silently commit to one reading — it records the ambiguity here.
#[derive(Clone, Debug, PartialEq)]
pub struct Ambiguity {
    /// Which kind of ambiguity this is, as a stable code.
    pub code: IssueCode,
    /// The fragment that was ambiguous.
    pub ref_text: String,
    /// Human-readable explanation, intended for display.
    pub reason: String,
    /// How many readings were plausible, when that count is known.
    pub candidate_count: Option<usize>,
    /// Where the fragment sits in the original input.
    pub span: Span,
}

/// A reading that is approximate rather than exact.
#[derive(Clone, Debug, PartialEq)]
pub struct Approximation {
    /// Why the value is approximate, as a stable code.
    pub code: IssueCode,
    /// The fragment the approximation came from.
    pub ref_text: String,
    /// Human-readable explanation, intended for display.
    pub reason: String,
    /// Relative error as a fraction (`0.05` meaning 5%), when it is known.
    pub relative_error: Option<f64>,
    /// Where the fragment sits in the original input.
    pub span: Span,
}

/// How much a finding should interrupt the user.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueSeverity {
    /// The parser filled in something reasonable; worth showing, not blocking.
    Info,
    /// The reading is usable but the parser made a choice worth confirming.
    Warning,
    /// No usable reading, or the active policy refused the one that was found.
    Error,
}

impl IssueSeverity {
    /// Returns the stable lowercase string for this severity.
    ///
    /// ```
    /// use unravel_nl::IssueSeverity;
    ///
    /// assert_eq!(IssueSeverity::Warning.as_str(), "warning");
    /// ```
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// A finding flattened into display-ready form, with severity and priority.
#[derive(Clone, Debug, PartialEq)]
pub struct RankedIssue {
    /// The underlying finding code.
    pub code: IssueCode,
    /// How much this finding should interrupt the user.
    pub severity: IssueSeverity,
    /// Display priority, higher first. Ranges from `30` to `100`.
    pub rank: u16,
    /// Whether a usable reading still exists despite this finding.
    ///
    /// `false` only for [`IssueCode::Empty`] and [`IssueCode::NoValue`], where
    /// there is nothing to fall back on.
    pub recoverable: bool,
    /// The fragment the finding refers to.
    pub ref_text: String,
    /// Human-readable explanation, intended for display.
    pub reason: String,
    /// Where the fragment sits in the original input.
    pub span: Span,
}

/// Flattens [`Parsed::findings`] into a single list ordered for display.
///
/// Skipped fragments, ambiguities, and approximations are merged, each tagged
/// with a [`IssueSeverity`] and a numeric rank, then sorted by rank descending
/// (ties broken by the referenced text) so a UI can show the most important
/// problem first without knowing the code taxonomy.
///
/// ```
/// use unravel_nl::{parse, ranked_findings, IssueSeverity};
///
/// let parsed = parse("", None);
/// let issues = ranked_findings(&parsed);
///
/// assert_eq!(issues[0].severity, IssueSeverity::Error);
/// assert_eq!(issues[0].rank, 100);
/// assert!(!issues[0].recoverable);
/// ```
pub fn ranked_findings(parsed: &Parsed) -> Vec<RankedIssue> {
    let mut issues = Vec::new();

    for issue in &parsed.findings.skipped {
        issues.push(ranked_issue(
            issue.code,
            issue.ref_text.clone(),
            issue.reason.clone(),
            issue.span.clone(),
        ));
    }
    for issue in &parsed.findings.ambiguities {
        issues.push(ranked_issue(
            issue.code,
            issue.ref_text.clone(),
            issue.reason.clone(),
            issue.span.clone(),
        ));
    }
    for issue in &parsed.findings.approximations {
        issues.push(ranked_issue(
            issue.code,
            issue.ref_text.clone(),
            issue.reason.clone(),
            issue.span.clone(),
        ));
    }

    issues.sort_by(|a, b| {
        b.rank
            .cmp(&a.rank)
            .then_with(|| a.ref_text.cmp(&b.ref_text))
    });
    issues
}

pub(crate) fn ranked_issue(
    code: IssueCode,
    ref_text: String,
    reason: String,
    span: Span,
) -> RankedIssue {
    RankedIssue {
        code,
        severity: issue_severity(code),
        rank: issue_rank(code),
        recoverable: issue_recoverable(code),
        ref_text,
        reason,
        span,
    }
}

pub(crate) fn issue_severity(code: IssueCode) -> IssueSeverity {
    match code {
        IssueCode::Empty
        | IssueCode::NoValue
        | IssueCode::UnknownUnit
        | IssueCode::TimezoneUnsupported
        | IssueCode::RecurrenceUnsupported
        | IssueCode::RejectedByPolicy => IssueSeverity::Error,
        IssueCode::TypoCorrected
        | IssueCode::AmbiguousNumber
        | IssueCode::AmbiguousDate
        | IssueCode::AmbiguousUnit
        | IssueCode::AmbiguousCurrency
        | IssueCode::Approximation => IssueSeverity::Warning,
        IssueCode::UnitAssumed => IssueSeverity::Info,
    }
}

pub(crate) fn issue_rank(code: IssueCode) -> u16 {
    match code {
        IssueCode::Empty | IssueCode::NoValue => 100,
        IssueCode::TimezoneUnsupported
        | IssueCode::RecurrenceUnsupported
        | IssueCode::RejectedByPolicy => 90,
        IssueCode::UnknownUnit => 80,
        IssueCode::TypoCorrected => 65,
        IssueCode::AmbiguousDate
        | IssueCode::AmbiguousNumber
        | IssueCode::AmbiguousUnit
        | IssueCode::AmbiguousCurrency => 55,
        IssueCode::UnitAssumed => 40,
        IssueCode::Approximation => 30,
    }
}

pub(crate) fn issue_recoverable(code: IssueCode) -> bool {
    !matches!(code, IssueCode::Empty | IssueCode::NoValue)
}

pub(crate) fn skipped(ref_text: &str, reason: &str) -> Skipped {
    let code = if ref_text.is_empty() {
        IssueCode::Empty
    } else {
        IssueCode::NoValue
    };
    skipped_with_code(ref_text, reason, code)
}

pub(crate) fn skipped_with_code(ref_text: &str, reason: &str, code: IssueCode) -> Skipped {
    skipped_with_span(ref_text, reason, code, span(ref_text))
}

pub(crate) fn skipped_with_span(
    ref_text: &str,
    reason: &str,
    code: IssueCode,
    span: Span,
) -> Skipped {
    Skipped {
        code,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        span,
    }
}

pub(crate) fn ambiguity(
    ref_text: &str,
    reason: &str,
    candidate_count: Option<usize>,
    code: IssueCode,
) -> Ambiguity {
    ambiguity_with_span(ref_text, reason, candidate_count, code, span(ref_text))
}

pub(crate) fn ambiguity_with_span(
    ref_text: &str,
    reason: &str,
    candidate_count: Option<usize>,
    code: IssueCode,
    span: Span,
) -> Ambiguity {
    Ambiguity {
        code,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        candidate_count,
        span,
    }
}

pub(crate) fn approximation(ref_text: &str, reason: &str) -> Approximation {
    approximation_with_span(ref_text, reason, span(ref_text))
}

pub(crate) fn approximation_with_span(ref_text: &str, reason: &str, span: Span) -> Approximation {
    Approximation {
        code: IssueCode::Approximation,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        relative_error: None,
        span,
    }
}

/// Rewrites every finding span so it addresses [`Parsed::input`].
///
/// Grammar dispatch runs on the normalized, trimmed text, so the spans the
/// grammars produce are in normalized coordinates while `input` holds the
/// original. This pass translates them back and re-reads [`Span::text`] from
/// `input`, which is what makes the documented guarantee true: `start` and
/// `end` are char boundaries of `input`, `input[start..end]` exists, and it
/// equals `text` — the fragment as the user typed it, not its normalized form.
///
/// `ref_text` follows `text` only when the two already agreed, since a
/// `ref_text` that differs was never a quotation of the input (it is the
/// corrected unit, the matched keyword, and so on).
///
/// Call this exactly once per [`Parsed`], at the entry point that owns `input`.
/// It is not idempotent: a second pass would translate offsets that are already
/// in original coordinates.
pub(crate) fn retarget_findings_to_input(parsed: &mut Parsed) {
    if parsed.findings.skipped.is_empty()
        && parsed.findings.ambiguities.is_empty()
        && parsed.findings.approximations.is_empty()
    {
        return;
    }

    let offsets = OriginalOffsets::for_input(&parsed.input);
    let input = std::mem::take(&mut parsed.input);
    for issue in &mut parsed.findings.skipped {
        retarget_span(&input, &offsets, &mut issue.span, &mut issue.ref_text);
    }
    for issue in &mut parsed.findings.ambiguities {
        retarget_span(&input, &offsets, &mut issue.span, &mut issue.ref_text);
    }
    for issue in &mut parsed.findings.approximations {
        retarget_span(&input, &offsets, &mut issue.span, &mut issue.ref_text);
    }
    parsed.input = input;
}

fn retarget_span(input: &str, offsets: &OriginalOffsets, span: &mut Span, ref_text: &mut String) {
    let start = floor_char_boundary(input, offsets.start(span.start));
    let end = ceil_char_boundary(input, offsets.end(span.end)).max(start);
    let Some(text) = input.get(start..end) else {
        return;
    };

    let quoted_the_input = *ref_text == span.text;
    span.start = start;
    span.end = end;
    if span.text != text {
        span.text.clear();
        span.text.push_str(text);
    }
    if quoted_the_input && ref_text.as_str() != text {
        ref_text.clear();
        ref_text.push_str(text);
    }
}

/// Rounds `idx` down to the nearest char boundary of `text`.
pub(crate) fn floor_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    let mut idx = idx;
    while !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Rounds `idx` up to the nearest char boundary of `text`.
pub(crate) fn ceil_char_boundary(text: &str, idx: usize) -> usize {
    if idx >= text.len() {
        return text.len();
    }
    let mut idx = idx;
    while !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx
}

pub(crate) fn span(text: &str) -> Span {
    Span {
        start: 0,
        end: text.len(),
        text: text.to_owned(),
    }
}

pub(crate) fn span_in(source: &str, fragment: &str) -> Span {
    if let Some(start) = source.find(fragment) {
        Span {
            start,
            end: start + fragment.len(),
            text: fragment.to_owned(),
        }
    } else {
        span(fragment)
    }
}

pub(crate) fn span_token_in(source: &str, fragment: &str) -> Span {
    token_spans(source)
        .into_iter()
        .find(|token| token.text.eq_ignore_ascii_case(fragment))
        .unwrap_or_else(|| span_in(source, fragment))
}

pub(crate) fn token_spans(source: &str) -> Vec<Span> {
    let mut tokens = Vec::new();
    let mut current: Option<(usize, TokenKind)> = None;

    for (idx, ch) in source.char_indices() {
        let Some(kind) = TokenKind::of(ch) else {
            if let Some((start, _)) = current.take() {
                tokens.push(span_slice(source, start, idx));
            }
            continue;
        };

        match current {
            Some((_, current_kind)) if current_kind == kind && kind != TokenKind::Symbol => {}
            Some((start, _)) => {
                tokens.push(span_slice(source, start, idx));
                current = Some((idx, kind));
            }
            None => current = Some((idx, kind)),
        }
    }

    if let Some((start, _)) = current {
        tokens.push(span_slice(source, start, source.len()));
    }

    tokens
}

pub(crate) fn span_slice(source: &str, start: usize, end: usize) -> Span {
    Span {
        start,
        end,
        text: source[start..end].to_owned(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Number,
    Word,
    Symbol,
}

impl TokenKind {
    pub(crate) fn of(ch: char) -> Option<Self> {
        if ch.is_whitespace() {
            None
        } else if ch.is_ascii_digit() || matches!(ch, '.' | ',' | '+' | '-') {
            Some(Self::Number)
        } else if ch.is_alphabetic() || ch == '_' {
            Some(Self::Word)
        } else {
            Some(Self::Symbol)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_source_spans_for_findings() {
        let tokens = token_spans("USD 10 to JPY");
        assert_eq!(
            tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<Vec<_>>(),
            vec!["USD", "10", "to", "JPY"]
        );

        let dollar = span_token_in("$12", "$");
        assert_eq!(dollar.start, 0);
        assert_eq!(dollar.end, 1);

        let cups = span_token_in("1.5 cups", "cups");
        assert_eq!(cups.start, 4);
        assert_eq!(cups.end, 8);
    }
}
