use crate::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueCode {
    Empty,
    NoValue,
    UnknownUnit,
    TypoCorrected,
    UnitAssumed,
    AmbiguousNumber,
    AmbiguousDate,
    AmbiguousUnit,
    AmbiguousCurrency,
    TimezoneUnsupported,
    RecurrenceUnsupported,
    RejectedByPolicy,
    Approximation,
}

impl IssueCode {
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Findings {
    pub skipped: Vec<Skipped>,
    pub ambiguities: Vec<Ambiguity>,
    pub approximations: Vec<Approximation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Skipped {
    pub code: IssueCode,
    pub ref_text: String,
    pub reason: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ambiguity {
    pub code: IssueCode,
    pub ref_text: String,
    pub reason: String,
    pub candidate_count: Option<usize>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Approximation {
    pub code: IssueCode,
    pub ref_text: String,
    pub reason: String,
    pub relative_error: Option<f64>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
}

impl IssueSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedIssue {
    pub code: IssueCode,
    pub severity: IssueSeverity,
    pub rank: u16,
    pub recoverable: bool,
    pub ref_text: String,
    pub reason: String,
    pub span: Span,
}

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
