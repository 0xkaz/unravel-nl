//! Internal parser dispatch.
//!
//! Public calls enter through [`Parser`]; this module holds the one normalized
//! route used by every configured instance.

use crate::*;

/// Parses one value out of `text`, trying every supported grammar.
///
/// [`Parser`] supplies the context. [`ParseCtx::purpose`] restricts the
/// dispatch without adding another public entry point.
///
/// The reading the parser ranked first is in [`Parsed::best`], competing
/// readings are in [`Parsed::alternatives`], and anything skipped, ambiguous,
/// or approximated is reported in [`Parsed::findings`] rather than dropped.
/// `best` is `None` when nothing could be read at all.
///
/// [`ParseCtx::expected_dimensions`] is enforced here: with a non-empty set, a
/// reading from a measurement domain outside it is refused rather than
/// returned, and the refusal is reported as [`IssueCode::RejectedByPolicy`].
///
/// ```
/// use unravel_nl::Parser;
///
/// let parsed = Parser::japanese_building().parse("5尺3寸");
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
        retarget_findings_to_input(&mut parsed);
        return parsed;
    }

    match ctx.purpose {
        ParsePurpose::General => parse_normalized_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Quantity => parse_quantity_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Number => parse_number_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Date => parse_date_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::DimensionEditor => {
            // The refusal it reports is already on `parsed`; only the editor
            // extractor needs the answer as a value. There is no neighbouring
            // label here, so the declaration is the only thing that decides
            // what a bare number stands for.
            let _ = parse_editor_dimension_into(
                trimmed,
                &ctx,
                EditorDimensions::declared_only(ctx.expected_dimensions),
                &mut parsed,
            );
        }
    }
    enforce_expected_dimensions(trimmed, &ctx, &mut parsed);
    retarget_findings_to_input(&mut parsed);
    parsed
}

// The old specialized functions survive only as test adapters while the
// regression corpus migrates to the single `Parser::parse` route. They contain
// no parsing logic of their own.
#[cfg(test)]
pub(crate) fn parse_quantity_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    parse_with_test_purpose(text, ctx, ParsePurpose::Quantity)
}

#[cfg(test)]
pub(crate) fn parse_number_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    parse_with_test_purpose(text, ctx, ParsePurpose::Number)
}

#[cfg(test)]
pub(crate) fn parse_date_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    parse_with_test_purpose(text, ctx, ParsePurpose::Date)
}

#[cfg(test)]
fn parse_with_test_purpose(text: &str, ctx: Option<ParseCtx>, purpose: ParsePurpose) -> Parsed {
    let mut ctx = ctx.unwrap_or_default();
    ctx.purpose = purpose;
    parse(text, Some(ctx))
}

pub(crate) fn parsed_shell(text: &str, ctx: &ParseCtx) -> Parsed {
    Parsed {
        input: text.to_owned(),
        locale: ctx.locale.clone(),
        strictness: ctx.strictness,
        best: None,
        alternatives: Vec::new(),
        suggestions: Vec::new(),
        findings: Findings::default(),
    }
}

/// Extracts only building dimensions from free text, for editor fields.
///
/// A scanner for inputs where a length or an area is the only
/// meaningful reading. Currency, dates, and general grammar are deliberately
/// not attempted, so text like `予算1234` or `next friday` yields nothing
/// instead of a wrong value. Japanese building units such as `帖` are kept, and
/// labelled bare numbers such as `寸法3640` are read as unitless dimensions.
///
/// With an empty [`ParseCtx::expected_dimensions`] the accepted set is decided
/// as it always was, by the label next to each candidate. A non-empty set
/// **composes with that label rather than replacing it**: the two intersect, so
/// a declaration can only narrow what the label already allowed. Declaring
/// lengths does not make `面積3640` — a bare number under an *area* label — into
/// one, and it does not make `予算1234`, which carries no dimension label at
/// all, into one either. Text that is no dimension under its own label is
/// simply not a match, whatever was declared, since nothing was refused.
///
/// What the declaration does do is refuse: a candidate that *would* have been a
/// dimension under its label but is not in the declared set is returned as a
/// match with `best: None`, the refused reading in [`Parsed::alternatives`],
/// and an [`IssueCode::RejectedByPolicy`] finding, rather than dropped. That
/// covers a labelled bare number as well as a labelled unit — `寸法3640` is a
/// millimetre length, and an area-only field says so instead of quietly
/// dropping it.
///
/// The label is this crate's own inference, so it never produces a refusal of
/// its own: with nothing declared, every result is exactly what it was before
/// the field existed.
///
/// Text the scanner cannot interpret never takes a reading down with it. The
/// scanner bounds each candidate optimistically — a space may be a grouped
/// number or a unit about to follow — so `幅3640 and 2` is first offered as
/// `3640 and`, which reads as nothing. Rather than drop the candidate, the
/// guesses are retried away until something reads: the match is `3640`, and the
/// `and` it could not use is reported as [`IssueCode::TrailingInput`]. A
/// non-empty result whose findings hold that code is a *partial* read, so
/// [`accepts`] is false for it.
///
/// ```
/// use unravel_nl::Parser;
///
/// let matches = Parser::japanese_building().parse_dimensions_for_editor(
///     "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
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
    parse_normalized_dispatch(trimmed, ctx, parsed);
    report_ambiguous_quantity_number(trimmed, ctx, parsed, parse_normalized_dispatch);
    report_closed_compound_alternative(trimmed, ctx, parsed);
    finalize_parsed(trimmed, parsed);
}

pub(crate) fn parse_normalized_dispatch(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    dispatch(
        Entry::General,
        PlainNumberSink::Contextual,
        trimmed,
        ctx,
        parsed,
    );
}

pub(crate) fn parse_quantity_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    parse_quantity_fast_dispatch(trimmed, ctx, parsed);
    report_ambiguous_quantity_number(trimmed, ctx, parsed, parse_quantity_fast_dispatch);
    report_closed_compound_alternative(trimmed, ctx, parsed);
    finalize_parsed(trimmed, parsed);
}

pub(crate) fn parse_quantity_fast_dispatch(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    dispatch(
        Entry::Quantity,
        PlainNumberSink::Contextual,
        trimmed,
        ctx,
        parsed,
    );
}

pub(crate) fn parse_number_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    parse_number_into(PlainNumberSink::Contextual, trimmed, ctx, parsed);
}

/// The bare-number grammar, for the broad entry point and the editor alike.
///
/// Only the sink for a plain number differs between them, so they share the
/// grammar rather than each keeping a copy of it.
pub(crate) fn parse_number_into(
    sink: PlainNumberSink,
    trimmed: &str,
    ctx: &ParseCtx,
    parsed: &mut Parsed,
) {
    dispatch(Entry::Number, sink, trimmed, ctx, parsed);
    finalize_parsed(trimmed, parsed);
}

pub(crate) fn parse_date_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    dispatch(
        Entry::Date,
        PlainNumberSink::Contextual,
        trimmed,
        ctx,
        parsed,
    );
}

pub(crate) fn set_plain_number_result(
    text: &str,
    ctx: &ParseCtx,
    reading: Reading,
    parsed: &mut Parsed,
) {
    if ctx.expect == Some(Kind::Quantity) || ctx.expected_dimensions.contains(Dimension::Length) {
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

/// Reports the grouping ambiguity a unit-bearing reading inherits from its number.
///
/// A bare `1.234` is undecidable under [`NumberFormat::Auto`] and says so:
/// 1.234, with 1234 as an alternative and an [`IssueCode::AmbiguousNumber`]
/// finding. Attaching a unit does not decide anything — `1.234 kg` is the same
/// number — but the quantity grammars parse their numeric part with
/// [`parse_number`], which just picks one reading. That made the factor of a
/// thousand disappear silently, and made strict callers accept a guess they
/// exist to refuse.
///
/// The competing reading is produced by rewriting the number and re-running the
/// same dispatch under a *declared* format, rather than by scaling the value:
/// the number is not always a linear factor of the reading (`1.234 F` is an
/// offset conversion, `1.234 hours` a unit multiple), and re-parsing gets every
/// grammar's own arithmetic for free. The declared format also means the
/// re-parse cannot re-enter this question.
///
/// An explicitly declared [`NumberFormat`] settles the question for quantities
/// exactly as it does for bare numbers, and nothing is reported.
pub(crate) fn report_ambiguous_quantity_number(
    trimmed: &str,
    ctx: &ParseCtx,
    parsed: &mut Parsed,
    dispatch: fn(&str, &ParseCtx, &mut Parsed),
) {
    let Some(best) = parsed.best.as_ref() else {
        return;
    };
    // Ranges carry one number per endpoint and are reported endpoint by
    // endpoint (`range_endpoint_ambiguities`); a bare number already reported
    // itself on the way in.
    if best.unit.is_none() || best.value.is_none() || best.range.is_some() {
        return;
    }
    if parsed
        .findings
        .ambiguities
        .iter()
        .any(|issue| issue.code == IssueCode::AmbiguousNumber)
    {
        return;
    }
    let (kind, unit, dimension, value) = (best.kind, best.unit.clone(), best.dimension, best.value);

    let Some((start, token, ambiguous)) = ambiguous_number_token(trimmed, ctx) else {
        return;
    };
    let Some(competing_value) = ambiguous
        .alternatives
        .first()
        .and_then(|reading| reading.value)
    else {
        return;
    };

    let mut rewritten = String::with_capacity(trimmed.len() + 4);
    rewritten.push_str(&trimmed[..start]);
    rewritten.push_str(&competing_value.to_string());
    rewritten.push_str(&trimmed[start + token.len()..]);

    let mut settled = ctx.clone();
    settled.number_format = NumberFormat::DotDecimal;
    let mut competing = parsed_shell(&rewritten, &settled);
    dispatch(&rewritten, &settled, &mut competing);

    let Some(mut competing_best) = competing.best else {
        return;
    };
    // Only the number was rewritten, so anything else moving means the rewrite
    // reached a different grammar and is not the competing reading of *this*
    // input.
    if competing_best.kind != kind
        || competing_best.unit != unit
        || competing_best.dimension != dimension
        || competing_best.value == value
    {
        return;
    }
    competing_best.confidence = competing_best
        .confidence
        .map(|confidence| confidence * AMBIGUOUS_ALTERNATIVE_FACTOR);

    let mut ambiguity = ambiguous.ambiguity;
    ambiguity.span = span_slice(trimmed, start, start + token.len());
    parsed.alternatives.push(competing_best);
    parsed.findings.ambiguities.push(ambiguity);
}

/// Ranks the competing reading below the one the grammar chose, in the same
/// proportion the bare-number path uses (0.56 against 0.64).
pub(crate) const AMBIGUOUS_ALTERNATIVE_FACTOR: f64 = 0.875;

/// Reports the compound reading the registry displaced in a closed-up
/// `<number><alias>` input.
///
/// `5m3` is the registry's cubic metre and `1m80` is metres and centimetres,
/// and the two are written identically, so reading `5m3` as 5 m + 3 cm is not a
/// mistake — it is the other plausible reading. [`closed_registry_unit`] decides
/// which one leads, because a declared registry alias outranks a guess at where
/// a compound splits; this reports the loser as an alternative with an
/// [`IssueCode::AmbiguousUnit`] finding, so the choice is visible rather than
/// silent, as the no-forced-choice guarantee requires.
///
/// The spaced form is not ambiguous this way. The compound idiom is never
/// written with a space before its unit, so `5 m3` has one reading and
/// `spaced_registry_unit` drops the other without reporting anything.
///
/// Runs from both `parse_normalized_into` and [`parse_quantity_fast_into`], so
/// every entry point reports the same alternative and the same finding.
pub(crate) fn report_closed_compound_alternative(
    trimmed: &str,
    ctx: &ParseCtx,
    parsed: &mut Parsed,
) {
    let Some(best) = parsed.best.as_ref() else {
        return;
    };
    let Some(mut alternative) = closed_compound_alternative(trimmed, ctx.unit_registry) else {
        return;
    };
    // The grammars are guarded, so the compound reading should never also be
    // the winner; if some other path ever makes it one, there is no competition
    // left to report.
    if alternative.value == best.value && alternative.unit == best.unit {
        return;
    }
    alternative.confidence = alternative
        .confidence
        .map(|confidence| confidence * AMBIGUOUS_ALTERNATIVE_FACTOR);
    parsed.alternatives.push(alternative);
    parsed.findings.ambiguities.push(ambiguity(
        trimmed,
        CLOSED_COMPOUND_REGISTRY_UNIT_READ,
        Some(2),
        IssueCode::AmbiguousUnit,
    ));
}

pub(crate) const NON_FINITE_REASON: &str =
    "numeric value overflowed to a magnitude with no finite representation";

pub(crate) const DESCENDING_RANGE_REASON: &str = "Range endpoints run from high to low; the written order was preserved rather than silently swapped.";

/// Applies the checks that every parse result must pass before it is returned.
///
/// Runs after grammar dispatch, so it sees the reading whichever grammar won.
/// It is idempotent: nested dispatch paths may run it more than once.
pub(crate) fn finalize_parsed(text: &str, parsed: &mut Parsed) {
    reject_non_finite(text, parsed);
    flag_descending_range(text, parsed);
}

/// Drops readings whose value overflowed to infinity or collapsed to NaN.
///
/// A non-finite value is not a reading of the input, it is the loss of one, so
/// it is reported rather than handed back as a clean value.
pub(crate) fn reject_non_finite(text: &str, parsed: &mut Parsed) {
    let best_lost = parsed
        .best
        .as_ref()
        .is_some_and(|best| !reading_is_finite(best));
    let alternative_lost = parsed
        .alternatives
        .iter()
        .any(|reading| !reading_is_finite(reading));
    if !best_lost && !alternative_lost {
        return;
    }

    if best_lost {
        parsed.best = None;
    }
    parsed.alternatives.retain(reading_is_finite);
    parsed.findings.skipped.push(skipped_with_span(
        text,
        NON_FINITE_REASON,
        IssueCode::NoValue,
        span(text),
    ));
}

pub(crate) fn reading_is_finite(reading: &Reading) -> bool {
    reading.value.is_none_or(f64::is_finite)
        && reading
            .range
            .as_ref()
            .is_none_or(|range| reading_is_finite(&range.from) && reading_is_finite(&range.to))
}

/// Records a range whose endpoints descend, without reordering them.
///
/// A caller iterating `from..to` would get an empty sweep, so the reading is
/// kept exactly as written and the surprise is reported instead. Reordering
/// silently would itself lose what the input said.
pub(crate) fn flag_descending_range(text: &str, parsed: &mut Parsed) {
    if !parsed.best.as_ref().is_some_and(range_is_descending) {
        return;
    }
    if parsed
        .findings
        .ambiguities
        .iter()
        .any(|issue| issue.reason == DESCENDING_RANGE_REASON)
    {
        return;
    }
    parsed.findings.ambiguities.push(ambiguity_with_span(
        text,
        DESCENDING_RANGE_REASON,
        Some(2),
        IssueCode::AmbiguousNumber,
        span(text),
    ));
}

pub(crate) fn range_is_descending(reading: &Reading) -> bool {
    let Some(range) = reading.range.as_ref() else {
        return false;
    };
    if let (Some(from), Some(to)) = (range.from.value, range.to.value) {
        return from > to;
    }
    if let (Some(from), Some(to)) = (range.from.date.as_deref(), range.to.date.as_deref()) {
        return from > to;
    }
    false
}

/// Refuses a reading whose measurement domain the caller did not declare.
///
/// This is the one place [`ParseCtx::expected_dimensions`] is enforced, and
/// every entry point runs it on the way out, so the declaration binds the same
/// way whichever door the caller came through. An empty set is no restriction
/// and returns immediately, which is what makes the field free for callers who
/// never set it.
///
/// Three rules, each of which the field documentation states:
///
/// - A reading with **no** dimension — a bare number, a date — is
///   never refused. It has no measurement domain to be outside of, and the
///   collisions this exists to remove were all unit against unit.
/// - A [`Kind::Range`] is judged by its endpoints, which is where a range keeps
///   its dimension.
/// - If a competing reading of the same input is in the set, it is promoted to
///   `best` rather than lost along with the refused one. The refusal is still
///   reported, because the choice between them is exactly what
///   [`Parsed::findings`] exists to make visible.
///
/// The refused reading is moved into [`Parsed::alternatives`] rather than
/// dropped, the same way an acceptance policy refusal is handled by
/// [`reject_candidate`].
///
/// Returns whether anything was refused.
pub(crate) fn enforce_expected_dimensions(text: &str, ctx: &ParseCtx, parsed: &mut Parsed) -> bool {
    if ctx.expected_dimensions.is_empty() {
        return false;
    }
    let Some(best) = parsed.best.as_ref() else {
        return false;
    };
    let Some(refused) = reading_dimension_outside(best, ctx.expected_dimensions) else {
        return false;
    };

    let promoted = parsed
        .alternatives
        .iter()
        .position(|reading| reading_is_within_dimensions(reading, ctx.expected_dimensions));
    let refused_reading = parsed.best.take().expect("checked above");
    if let Some(index) = promoted {
        parsed.best = Some(parsed.alternatives.remove(index));
    }
    parsed.alternatives.push(refused_reading);
    parsed.findings.skipped.push(skipped_with_span(
        text,
        &expected_dimensions_reason(refused, ctx.expected_dimensions),
        IssueCode::RejectedByPolicy,
        span(text),
    ));
    retell_closed_compound_ambiguity(parsed);
    true
}

/// Corrects the closed-compound ambiguity after a refusal moved `best`.
///
/// [`report_closed_compound_alternative`] runs while the registry unit is still
/// the reading, and says so. When the declared dimensions then refuse that
/// registry unit, the sentence it left behind states the opposite of what
/// happened — `5m3` under a length field reads as the compound, and the
/// ambiguity claimed the cubic metre was read. That finding is the one place a
/// caller is told which of the two readings won, so the refusal rewrites it
/// rather than adding a second, contradicting one.
pub(crate) fn retell_closed_compound_ambiguity(parsed: &mut Parsed) {
    let promoted = parsed.best.is_some();
    for ambiguity in &mut parsed.findings.ambiguities {
        if ambiguity.code == IssueCode::AmbiguousUnit
            && ambiguity.reason == CLOSED_COMPOUND_REGISTRY_UNIT_READ
        {
            ambiguity.reason = if promoted {
                CLOSED_COMPOUND_COMPOUND_READ.to_owned()
            } else {
                CLOSED_COMPOUND_NEITHER_READ.to_owned()
            };
        }
    }
}

/// What [`report_closed_compound_alternative`] says while the registry unit is
/// still the reading.
pub(crate) const CLOSED_COMPOUND_REGISTRY_UNIT_READ: &str = "Written closed up, this is both a registry unit and a compound quantity; the registry unit was read.";

/// What it says once the declared dimensions refused the registry unit and the
/// compound quantity took its place.
pub(crate) const CLOSED_COMPOUND_COMPOUND_READ: &str = "Written closed up, this is both a registry unit and a compound quantity; the registry unit is outside the expected dimensions, so the compound quantity was read.";

/// What it says when the declared dimensions refused the registry unit and left
/// nothing in its place.
pub(crate) const CLOSED_COMPOUND_NEITHER_READ: &str = "Written closed up, this is both a registry unit and a compound quantity; the registry unit is outside the expected dimensions, and neither reading was accepted.";

/// Returns the first dimension of `reading` that `expected` does not allow.
///
/// A range carries its dimensions on its endpoints, so it is asked about them;
/// a reading with no dimension anywhere answers `None` and is allowed.
pub(crate) fn reading_dimension_outside(
    reading: &Reading,
    expected: DimensionSet,
) -> Option<Dimension> {
    if let Some(dimension) = reading.dimension
        && !expected.allows(dimension)
    {
        return Some(dimension);
    }
    let range = reading.range.as_ref()?;
    reading_dimension_outside(&range.from, expected)
        .or_else(|| reading_dimension_outside(&range.to, expected))
}

/// Whether a reading carries a dimension and every one of them is expected.
///
/// Stricter than the negation of [`reading_dimension_outside`]: a dimensionless
/// reading is not refused, but it is not a *replacement* for a refused one
/// either, so it is never promoted over one.
pub(crate) fn reading_is_within_dimensions(reading: &Reading, expected: DimensionSet) -> bool {
    let dimensioned = reading.dimension.is_some()
        || reading
            .range
            .as_ref()
            .is_some_and(|range| range.from.dimension.is_some() || range.to.dimension.is_some());
    dimensioned && reading_dimension_outside(reading, expected).is_none()
}

pub(crate) fn expected_dimensions_reason(refused: Dimension, expected: DimensionSet) -> String {
    let mut reason = format!(
        "dimension {} is outside the expected dimensions: ",
        refused.as_str()
    );
    for (index, dimension) in expected.iter().enumerate() {
        if index > 0 {
            reason.push_str(", ");
        }
        reason.push_str(dimension.as_str());
    }
    reason
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

/// Which of the two approximate grammars produced a reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Approximate {
    /// An explicit qualifier in the text, such as `about` or `約`.
    Qualified,
    /// A caller-supplied fuzzy profile term.
    Fuzzy,
}

impl Approximate {
    fn confirmation_reason(self) -> &'static str {
        match self {
            Approximate::Qualified => "approximate qualifier requires confirmation in strict mode",
            Approximate::Fuzzy => "fuzzy reading requires confirmation in strict mode",
        }
    }
}

/// What the gate decided about an approximate reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Admission {
    /// The reading stands, with its approximations reported.
    Accept,
    /// Strict mode refuses it until a human confirms.
    RequireConfirmation,
    /// The acceptance policy switched this grammar off.
    RefusedByPolicy,
}

/// The single gate on approximate readings.
///
/// It used to be written twice, and the copy in the quantity entry point read
/// only `ctx.accept.fuzzy`: under [`Strictness::Strict`], `parse` refused a
/// fuzzy term while `parse_quantity_fast` returned it, so declaring strictness
/// bought nothing on the narrower entry point. Both now ask this function, and
/// so does anything added later.
pub(crate) fn approximate_admission(kind: Approximate, ctx: &ParseCtx) -> Admission {
    if ctx.strictness == Strictness::Strict {
        return Admission::RequireConfirmation;
    }
    if kind == Approximate::Fuzzy && !ctx.accept.fuzzy {
        return Admission::RefusedByPolicy;
    }
    Admission::Accept
}

/// Applies [`approximate_admission`] to one candidate reading.
pub(crate) fn admit_approximate(
    kind: Approximate,
    ctx: &ParseCtx,
    trimmed: &str,
    reading: Reading,
    approximations: Vec<Approximation>,
    parsed: &mut Parsed,
) {
    match approximate_admission(kind, ctx) {
        Admission::Accept => {
            parsed.best = Some(reading);
            parsed.findings.approximations = approximations;
        }
        Admission::RequireConfirmation => parsed.findings.skipped.push(skipped_with_span(
            trimmed,
            kind.confirmation_reason(),
            IssueCode::Approximation,
            span(trimmed),
        )),
        Admission::RefusedByPolicy => reject_candidate(
            parsed,
            trimmed,
            reading,
            "fuzzy readings are disabled by acceptance policy",
        ),
    }
}

/// Reports a run of same-dimension quantities that states no sum.
///
/// The run was recognized — every part is a number and a unit of one dimension
/// — so it is not `NoValue`: what failed is the claim that the parts are places
/// of a single measurement. Summing them anyway is how `3 m 5 m` used to read
/// as 8 m with nothing in the findings to say the 8 was the parser's.
pub(crate) fn note_malformed_compound(parsed: &mut Parsed, text: &str, reason: &str) {
    parsed.findings.skipped.push(skipped_with_span(
        text,
        reason,
        IssueCode::CompoundOverflow,
        span(text),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The `Auto` grouping ambiguity needs a grouping shape on *both* sides of
    /// the separator: one to three digits before it and exactly three after.
    #[test]
    fn auto_ambiguity_needs_one_to_three_digits_before_the_separator() {
        let ambiguous = |text: &str| {
            let parsed = parse_number_fast(text, None);
            assert!(parsed.best.is_some(), "{text}");
            parsed
                .findings
                .ambiguities
                .iter()
                .any(|issue| issue.code == IssueCode::AmbiguousNumber)
        };

        for text in [
            "1.234", "12.345", "123.456", "0.123", "00.123", "1,234", "12,345", "123,456",
        ] {
            assert!(ambiguous(text), "{text} should be ambiguous");
            assert_eq!(
                parse_number_fast(text, None).alternatives.len(),
                1,
                "{text}"
            );
        }

        // A left side longer than a leading group settles the shape, as does a
        // right side that is not exactly three digits.
        for text in [
            "1234.567",
            "12345.678",
            "0000.123",
            "1234,567",
            "1.23",
            "1.2345",
            "0.5",
            "1.2340",
        ] {
            assert!(!ambiguous(text), "{text} should not be ambiguous");
            assert!(
                parse_number_fast(text, None).alternatives.is_empty(),
                "{text}"
            );
        }
    }

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
                expected_dimensions: DimensionSet::from(Dimension::Length),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(parsed.best.as_ref().unwrap().kind, Kind::Number);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.alternatives[0].unit.as_deref(), Some("mm"));
        assert_eq!(parsed.findings.ambiguities.len(), 1);
    }

    #[test]
    fn rejects_numbers_that_overflow_to_non_finite() {
        for input in [
            "1".repeat(400),
            "9".repeat(400),
            format!("{}kg", "9".repeat(400)),
        ] {
            let parsed = parse(&input, None);
            assert!(parsed.best.is_none(), "{input}");
            assert!(
                parsed
                    .alternatives
                    .iter()
                    .all(|reading| reading.value.is_none_or(f64::is_finite)),
                "{input}"
            );
            let skipped = parsed
                .findings
                .skipped
                .iter()
                .find(|issue| issue.reason == NON_FINITE_REASON)
                .unwrap_or_else(|| panic!("non-finite finding for {input}"));
            assert_eq!(skipped.code, IssueCode::NoValue);
        }

        // The magnitude just below the overflow threshold is still readable.
        let finite = parse("100000000000000000000", None);
        assert_eq!(finite.best.expect("number").value, Some(1e20));
        assert!(finite.findings.skipped.is_empty());
    }

    #[test]
    fn rejects_non_finite_through_narrow_entry_points() {
        let overflowing = "9".repeat(400);
        for parsed in [
            parse_number_fast(&overflowing, None),
            parse_quantity_fast(&format!("{overflowing}kg"), None),
        ] {
            assert!(parsed.best.is_none());
            assert!(
                parsed
                    .findings
                    .skipped
                    .iter()
                    .any(|issue| issue.reason == NON_FINITE_REASON)
            );
        }
    }

    #[test]
    fn reports_descending_ranges_without_reordering_them() {
        for (input, from, to) in [
            ("from 10kg to 2kg", 10.0, 2.0),
            ("10-5 kg", 10.0, 5.0),
            ("100〜50m", 100.0, 50.0),
            ("10 ± -3 mm", 0.013, 0.007),
        ] {
            let parsed = parse(input, None);
            let best = parsed.best.as_ref().expect(input);
            assert_eq!(best.kind, Kind::Range, "{input}");
            let range = best.range.as_ref().expect(input);
            // Endpoints are preserved exactly as written; silently swapping
            // them would itself be a loss.
            crate::test_util::assert_close(range.from.value.unwrap(), from);
            crate::test_util::assert_close(range.to.value.unwrap(), to);

            let issue = parsed
                .findings
                .ambiguities
                .iter()
                .find(|issue| issue.reason == DESCENDING_RANGE_REASON)
                .unwrap_or_else(|| panic!("descending finding for {input}"));
            assert_eq!(issue.code, IssueCode::AmbiguousNumber, "{input}");
            assert_eq!(issue.candidate_count, Some(2), "{input}");
            assert_eq!(issue.span.text, input, "{input}");
        }
    }

    #[test]
    fn leaves_ascending_ranges_unflagged() {
        for input in ["from 2kg to 10kg", "5-10 kg", "50〜100m", "10 ± 3 mm"] {
            let parsed = parse(input, None);
            assert_eq!(parsed.best.as_ref().expect(input).kind, Kind::Range);
            assert!(
                !parsed
                    .findings
                    .ambiguities
                    .iter()
                    .any(|issue| issue.reason == DESCENDING_RANGE_REASON),
                "{input}"
            );
        }
    }
}
