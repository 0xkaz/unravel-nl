//! Entry points.
//!
//! [`parse`] is the broad entry point: use it when the input could be any of a
//! quantity, date, time, range, recurrence, conversion request, or plain
//! number. Every other entry point here is narrower, and narrower is better
//! whenever the caller already knows what the field holds — a dedicated entry
//! point does less grammar dispatch, so it is faster and, more importantly, it
//! cannot misread the input as some other kind of value. A date field parsed
//! with [`parse_date_fast`] will never come back holding a currency.
//!
//! | Entry point | Use when |
//! | --- | --- |
//! | [`parse`] | The kind of value is unknown. |
//! | [`parse_quantity_fast`] | The field holds a measurement. |
//! | [`parse_number_fast`] | The field holds a bare number. |
//! | [`parse_date_fast`] | The field holds a date. |
//! | [`parse_recurrence_fast`] | The field holds a repeating schedule. |
//! | [`parse_all`] | Free text that may contain several values. |
//! | [`parse_dimensions_for_editor`] | Free text where only lengths and areas count. |

use crate::*;

/// Parses one value out of `text`, trying every supported grammar.
///
/// This is the general entry point. Set [`ParseCtx::purpose`] to restrict the
/// dispatch without switching functions, or call one of the narrower entry
/// points in this module directly.
///
/// The reading the parser ranked first is in [`Parsed::best`], competing
/// readings are in [`Parsed::alternatives`], and anything skipped, ambiguous,
/// or approximated is reported in [`Parsed::findings`] rather than dropped.
/// `best` is `None` when nothing could be read at all.
///
/// ```
/// use unravel_nl::{parse, Locale, ParseCtx};
///
/// let parsed = parse(
///     "5尺3寸",
///     Some(ParseCtx {
///         locale: Some(Locale::Ja),
///         ..ParseCtx::default()
///     }),
/// );
///
/// let best = parsed.best.expect("a canonical reading");
/// assert_eq!(best.unit.as_deref(), Some("m"));
/// ```
pub fn parse(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, &ctx);

    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }

    match ctx.purpose {
        ParsePurpose::General => parse_normalized_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Quantity => parse_quantity_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Number => parse_number_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Date => parse_date_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Recurrence => parse_recurrence_fast_into(trimmed, &mut parsed),
        ParsePurpose::DimensionEditor => parse_editor_dimension_into(trimmed, &ctx, &mut parsed),
    }
    parsed
}

/// Parses `text` as a measurement, skipping date and recurrence grammars.
///
/// Note that [`ParseCtx::expected_dimension`] is a hint, not a filter: this
/// entry point reports whatever dimension it reads, so `5 kg` still parses as a
/// mass even when a length was expected. Callers that need the reading refused
/// should check [`Reading::dimension`] themselves, or use
/// [`parse_dimensions_for_editor`], which does enforce the expectation.
///
/// ```
/// use unravel_nl::{parse_quantity_fast, Dimension, ParseCtx};
///
/// let parsed = parse_quantity_fast(
///     "1,234 kg",
///     Some(ParseCtx {
///         expected_dimension: Some(Dimension::Mass),
///         ..ParseCtx::default()
///     }),
/// );
///
/// assert_eq!(parsed.best.unwrap().unit.as_deref(), Some("kg"));
/// ```
pub fn parse_quantity_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    parse_quantity_fast_with_ctx(text, &ctx)
}

pub(crate) fn parse_quantity_fast_with_ctx(text: &str, ctx: &ParseCtx) -> Parsed {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_quantity_fast_into(trimmed, ctx, &mut parsed);
    parsed
}

/// Parses `text` as a bare number, without attaching a unit.
///
/// Locale number formats still apply, so grouping and decimal separators are
/// read according to [`ParseCtx::number_format`] and [`ParseCtx::locale`].
pub fn parse_number_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    parse_number_fast_with_ctx(text, &ctx)
}

pub(crate) fn parse_number_fast_with_ctx(text: &str, ctx: &ParseCtx) -> Parsed {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_number_fast_into(trimmed, ctx, &mut parsed);
    parsed
}

/// Parses `text` as a repeating schedule, canonicalized to an RRULE string.
///
/// The rule lands in [`Reading::recurrence`]. Phrases that are recognized as
/// recurrences but cannot be expressed as a supported rule are reported as
/// [`IssueCode::RecurrenceUnsupported`] instead of being approximated.
///
/// ```
/// use unravel_nl::{parse_recurrence_fast, Kind};
///
/// let parsed = parse_recurrence_fast("every monday", None);
/// let best = parsed.best.unwrap();
///
/// assert_eq!(best.kind, Kind::Recurrence);
/// assert_eq!(best.recurrence.as_deref(), Some("FREQ=WEEKLY;BYDAY=MO"));
/// ```
pub fn parse_recurrence_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, &ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_recurrence_fast_into(trimmed, &mut parsed);
    parsed
}

/// Parses `text` as a date, skipping quantity and currency grammars.
///
/// The parser never reads the host clock. Relative expressions such as
/// `next friday` resolve only when [`ParseCtx::reference_date`] is supplied and
/// the `dates-jiff` feature is enabled; otherwise they are reported as findings
/// rather than resolved against an implicit "today".
pub fn parse_date_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, &ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_date_fast_into(trimmed, &ctx, &mut parsed);
    parsed
}

pub(crate) fn parsed_shell(text: &str, ctx: &ParseCtx) -> Parsed {
    Parsed {
        input: text.to_owned(),
        locale: ctx.locale.clone(),
        best: None,
        alternatives: Vec::new(),
        suggestions: Vec::new(),
        findings: Findings::default(),
    }
}

/// Extracts every value found in a sentence, with byte spans.
///
/// The input is split into clauses and each clause is scanned for readings.
/// Overlapping matches are resolved so the returned matches are ordered by
/// position and never overlap, which makes them safe to use directly for
/// highlighting the original string.
///
/// ```
/// use unravel_nl::{parse_all, Locale, ParseCtx};
///
/// let matches = parse_all(
///     "延床100㎡、敷地面積120㎡、高さ3.5m",
///     Some(ParseCtx {
///         locale: Some(Locale::Ja),
///         ..ParseCtx::default()
///     }),
/// );
///
/// assert_eq!(matches.len(), 3);
/// assert_eq!(matches[0].text, "延床100㎡");
/// ```
pub fn parse_all(text: &str, ctx: Option<ParseCtx>) -> Vec<ParsedMatch> {
    let ctx = ctx.unwrap_or_default();
    let mut matches = Vec::new();
    for_clause_spans(text, |start, end| {
        push_clause_matches(&mut matches, text, start, end, &ctx);
    });
    sorted_non_overlapping_matches(matches)
}

/// Extracts only building dimensions from free text, for editor fields.
///
/// A narrowed [`parse_all`] for inputs where a length or an area is the only
/// meaningful reading. Currency, dates, and general grammar are deliberately
/// not attempted, so text like `予算1234` or `next friday` yields nothing
/// instead of a wrong value. Japanese building units such as `帖` are kept, and
/// labelled bare numbers such as `寸法3640` are read as unitless dimensions.
///
/// ```
/// use unravel_nl::{parse_dimensions_for_editor, Locale, ParseCtx};
///
/// let matches = parse_dimensions_for_editor(
///     "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
///     Some(ParseCtx {
///         locale: Some(Locale::Ja),
///         ..ParseCtx::default()
///     }),
/// );
///
/// assert_eq!(matches.len(), 4);
/// ```
pub fn parse_dimensions_for_editor(text: &str, ctx: Option<ParseCtx>) -> Vec<ParsedMatch> {
    let mut ctx = ctx.unwrap_or_default();
    ctx.purpose = ParsePurpose::DimensionEditor;
    ctx.expect = Some(Kind::Quantity);

    let mut matches = Vec::new();
    for_clause_spans(text, |clause_start, clause_end| {
        for_numeric_candidate_spans(text, clause_start, clause_end, |candidate| {
            if candidate_starts_with_currency(text, candidate.start) {
                return true;
            }
            push_editor_dimension_match(&mut matches, text, candidate, clause_start, &ctx);
            true
        });
    });

    sorted_non_overlapping_matches(matches)
}

pub(crate) fn parse_normalized_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    let features = InputFeatures::new(trimmed);

    if let Some(result) = parse_qualified_reading(trimmed, ctx) {
        if ctx.strictness == Strictness::Strict {
            parsed.findings.skipped.push(skipped_with_span(
                trimmed,
                "approximate qualifier requires confirmation in strict mode",
                IssueCode::Approximation,
                span(trimmed),
            ));
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    if let Some(result) = parse_fuzzy_reading(trimmed, ctx) {
        if ctx.strictness == Strictness::Strict {
            parsed.findings.skipped.push(skipped_with_span(
                trimmed,
                "fuzzy reading requires confirmation in strict mode",
                IssueCode::Approximation,
                span(trimmed),
            ));
        } else if !ctx.accept.fuzzy {
            reject_candidate(
                parsed,
                trimmed,
                result.reading,
                "fuzzy readings are disabled by acceptance policy",
            );
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    if features.has_slash
        && let Some(ambiguous) = parse_ambiguous_slash_date_or_fraction(trimmed, ctx)
    {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
        return;
    }

    if features.maybe_date
        && let Some(reading) = parse_relative_date(trimmed, ctx)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_recurrence
        && let Some(reading) = parse_recurrence(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_range
        && let Some(reading) = parse_plus_minus_range(trimmed, ctx)
    {
        if !ctx.accept.ranges {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "range readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_range
        && let Some(reading) = parse_upper_bound_range(trimmed, ctx)
    {
        if !ctx.accept.ranges {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "range readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_range
        && let Some(reading) = parse_range(trimmed, ctx)
    {
        if !ctx.accept.ranges {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "range readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_conversion
        && let Some(reading) = parse_conversion_request(trimmed, ctx)
    {
        if !ctx.accept.conversions {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "conversion readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_japanese_length
        && let Some(reading) = parse_japanese_length(trimmed)
    {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Japanese customary length converted to SI meters.",
        ));
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_tatami
        && let Some(reading) = parse_tatami_area(trimmed)
    {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Tatami area uses a trade-custom regional approximation of 1.62 m2.",
        ));
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_tsubo
        && let Some(reading) = parse_tsubo_area(trimmed)
    {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Tsubo area converted through Japanese customary area.",
        ));
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_area
        && let Some(reading) = parse_square_meter(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_temperature
        && let Some(reading) = parse_temperature(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_compound_quantity
        && let Some(reading) = parse_compound_registered_quantity_ctx(trimmed, ctx)
    {
        if !ctx.accept.compounds {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "compound quantity readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_quantity
        && let Some(reading) = parse_registered_quantity(trimmed, ctx)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_metric_length
        && let Some(reading) = parse_metric_length(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_mass
        && let Some(reading) = parse_mass(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_timezone_clock
        && let Some(reading) = parse_timezone_clock_time(trimmed, ctx)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_clock
        && let Some(reading) = parse_clock_time(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_duration
        && let Some(reading) = parse_duration(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_feet_inches
        && let Some(reading) = parse_feet_inches(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_cups
        && let Some((best, alternatives, ambiguity)) = parse_cups(trimmed, ctx)
    {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        parsed.findings.ambiguities.push(ambiguity);
        return;
    }

    if features.maybe_currency
        && let Some((best, alternatives, ambiguity)) = parse_currency(trimmed, ctx)
    {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        if let Some(ambiguity) = ambiguity {
            parsed.findings.ambiguities.push(ambiguity);
        }
        return;
    }

    if features.maybe_number
        && let Some(ambiguous) = parse_ambiguous_number(trimmed, ctx)
    {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
        return;
    }

    if features.maybe_number
        && let Some(reading) = parse_plain_number_ctx(trimmed, ctx)
    {
        set_plain_number_result(trimmed, ctx, reading, parsed);
        return;
    }

    if features.maybe_quantity
        && let Some((reading, suggestion, unit_text)) =
            parse_typo_corrected_quantity_ctx(trimmed, ctx)
    {
        parsed.suggestions.push(suggestion);
        match ctx.strictness {
            Strictness::Forgiving => {
                parsed.findings.ambiguities.push(ambiguity_with_span(
                    &unit_text,
                    "Unit spelling was corrected by did-you-mean matching.",
                    Some(1),
                    IssueCode::TypoCorrected,
                    span_token_in(trimmed, &unit_text),
                ));
                parsed.best = Some(reading);
            }
            Strictness::Confirm | Strictness::Strict => {
                parsed.findings.skipped.push(skipped_with_span(
                    &unit_text,
                    "unit spelling correction requires confirmation",
                    IssueCode::TypoCorrected,
                    span_token_in(trimmed, &unit_text),
                ));
            }
        }
        return;
    }

    if features.maybe_timezone_clock
        && let Some(timezone) = unsupported_timezone_suffix(trimmed)
    {
        parsed.findings.skipped.push(skipped_with_span(
            timezone,
            "unsupported timezone conversion requires an explicit adapter policy",
            IssueCode::TimezoneUnsupported,
            span_token_in(trimmed, timezone),
        ));
        return;
    }

    if features.maybe_recurrence
        && let Some(recurrence) = unsupported_recurrence_phrase(trimmed)
    {
        parsed.findings.skipped.push(skipped_with_span(
            recurrence,
            "recurring date/time expressions require a recurrence adapter and are not interpreted by the core parser",
            IssueCode::RecurrenceUnsupported,
            span_token_in(trimmed, recurrence),
        ));
        return;
    }

    if features.maybe_suggestion {
        parsed.suggestions = suggestions_for(trimmed);
    }
    parsed
        .findings
        .skipped
        .push(skipped(trimmed, "no supported reading matched"));
}

pub(crate) fn parse_quantity_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    if let Some(result) = parse_qualified_reading(trimmed, ctx) {
        if ctx.strictness == Strictness::Strict {
            parsed.findings.skipped.push(skipped_with_span(
                trimmed,
                "approximate qualifier requires confirmation in strict mode",
                IssueCode::Approximation,
                span(trimmed),
            ));
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    if let Some(result) = parse_fuzzy_reading(trimmed, ctx) {
        if !ctx.accept.fuzzy {
            reject_candidate(
                parsed,
                trimmed,
                result.reading,
                "fuzzy readings are disabled by acceptance policy",
            );
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    for parser in [
        parse_japanese_length as fn(&str) -> Option<Reading>,
        parse_tatami_area,
        parse_tsubo_area,
        parse_square_meter,
        parse_temperature,
        parse_metric_length,
        parse_mass,
        parse_clock_time,
        parse_duration,
        parse_feet_inches,
    ] {
        if let Some(reading) = parser(trimmed) {
            if reading.approximate == Some(true) {
                parsed
                    .findings
                    .approximations
                    .push(approximation(trimmed, "Approximate quantity conversion."));
            }
            parsed.best = Some(reading);
            return;
        }
    }

    if let Some(reading) = parse_compound_registered_quantity_ctx(trimmed, ctx) {
        if !ctx.accept.compounds {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "compound quantity readings are disabled by acceptance policy",
            );
        } else {
            parsed.best = Some(reading);
        }
        return;
    }

    if let Some(reading) = parse_registered_quantity(trimmed, ctx) {
        parsed.best = Some(reading);
        return;
    }

    if let Some((best, alternatives, ambiguity)) = parse_cups(trimmed, ctx) {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        parsed.findings.ambiguities.push(ambiguity);
        return;
    }

    if let Some((best, alternatives, ambiguity)) = parse_currency(trimmed, ctx) {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        if let Some(ambiguity) = ambiguity {
            parsed.findings.ambiguities.push(ambiguity);
        }
        return;
    }

    if let Some((reading, suggestion, unit_text)) = parse_typo_corrected_quantity_ctx(trimmed, ctx)
    {
        parsed.suggestions.push(suggestion);
        match ctx.strictness {
            Strictness::Forgiving => {
                parsed.findings.ambiguities.push(ambiguity_with_span(
                    &unit_text,
                    "Unit spelling was corrected by did-you-mean matching.",
                    Some(1),
                    IssueCode::TypoCorrected,
                    span_token_in(trimmed, &unit_text),
                ));
                parsed.best = Some(reading);
            }
            Strictness::Confirm | Strictness::Strict => {
                parsed.findings.skipped.push(skipped_with_span(
                    &unit_text,
                    "unit spelling correction requires confirmation",
                    IssueCode::TypoCorrected,
                    span_token_in(trimmed, &unit_text),
                ));
            }
        }
        return;
    }

    parsed
        .findings
        .skipped
        .push(skipped(trimmed, "no supported quantity matched"));
}

pub(crate) fn parse_number_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    if let Some(ambiguous) = parse_ambiguous_number(trimmed, ctx) {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
    } else if let Some(reading) = parse_plain_number_ctx(trimmed, ctx) {
        set_plain_number_result(trimmed, ctx, reading, parsed);
    } else {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "no supported number matched"));
    }
}

pub(crate) fn parse_date_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    if let Some(reading) = parse_relative_date(trimmed, ctx) {
        parsed.best = Some(reading);
    } else {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "no supported date matched"));
    }
}

pub(crate) fn parse_recurrence_fast_into(trimmed: &str, parsed: &mut Parsed) {
    if let Some(reading) = parse_recurrence(trimmed) {
        parsed.best = Some(reading);
    } else if let Some(recurrence) = unsupported_recurrence_phrase(trimmed) {
        parsed.findings.skipped.push(skipped_with_span(
            recurrence,
            "recurring date/time expressions require a recurrence adapter and are not interpreted by the core parser",
            IssueCode::RecurrenceUnsupported,
            span_token_in(trimmed, recurrence),
        ));
    } else {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "no supported recurrence matched"));
    }
}

pub(crate) fn set_plain_number_result(
    text: &str,
    ctx: &ParseCtx,
    reading: Reading,
    parsed: &mut Parsed,
) {
    if ctx.expect == Some(Kind::Quantity) || ctx.expected_dimension == Some(Dimension::Length) {
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

pub(crate) fn reject_candidate(parsed: &mut Parsed, text: &str, reading: Reading, reason: &str) {
    parsed.alternatives.push(reading);
    parsed.findings.skipped.push(skipped_with_span(
        text,
        reason,
        IssueCode::RejectedByPolicy,
        span(text),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_hostile_no_match_corpus() {
        for input in [
            "meters meters meters",
            "1,,,,,,,,kg",
            "nextnextnextnextnext",
            "(((((((((((((((((((((((((((((((((",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "尺尺尺尺尺",
        ] {
            let parsed = parse(input, None);
            assert!(parsed.best.is_none(), "{input}");
            assert_eq!(parsed.findings.skipped.len(), 1, "{input}");
            assert_eq!(parsed.findings.skipped[0].code, IssueCode::NoValue);
            if input.starts_with('a') {
                assert!(parsed.suggestions.is_empty(), "{input}");
            }
        }
    }

    #[test]
    fn suggests_context_implied_millimeters_for_plain_number() {
        let parsed = parse(
            "3640",
            Some(ParseCtx {
                expect: Some(Kind::Quantity),
                expected_dimension: Some(Dimension::Length),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(parsed.best.as_ref().unwrap().kind, Kind::Number);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.alternatives[0].unit.as_deref(), Some("mm"));
        assert_eq!(parsed.findings.ambiguities.len(), 1);
    }
}
