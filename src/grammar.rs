//! The one place the grammars and the order they are tried in are written down.
//!
//! Every entry point used to hand-write its own sequence of `if let Some(..)`
//! attempts. Two hand-written sequences are two grammars, and they disagreed:
//! `parse("3 m 5 m")` refused the run as a malformed compound while
//! `parse_quantity_fast("3 m 5 m")` read it as 480 seconds, both with an empty
//! findings channel, so no caller could tell which one had lied.
//!
//! So the order is defined exactly once, in [`GRAMMAR_ORDER`]. A narrow entry
//! point still reads fewer grammars than the broad one — that is what makes it
//! narrow, and it is the documented reason to prefer it — but it expresses that
//! as a *subset* through [`Entry::reads`] and walks the same order. It cannot
//! reorder what it does read, because it never sees an order of its own.

use crate::*;

/// One grammar the parser knows how to try.
///
/// The `dispatch` match over this enum is exhaustive, so a new grammar cannot
/// be added without both giving it a body and placing it in [`GRAMMAR_ORDER`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Grammar {
    /// `about 20kg`, `約3m` — an explicit approximation qualifier.
    Qualified,
    /// A caller-supplied fuzzy profile term such as `ちょい`.
    Fuzzy,
    /// `3/4` — a slash form that is either a date or a fraction.
    SlashDateOrFraction,
    /// `3/4/2026` — a three-part numeric date.
    NumericSlashDate,
    /// `next friday`, `来週金曜日`.
    RelativeDate,
    /// `10 ± 3 mm`.
    PlusMinusRange,
    /// `under 5 kg`, `5kg以下`.
    UpperBoundRange,
    /// `5-10 kg`, `2〜3日`, `from 2kg to 10kg`.
    Range,
    /// `5 kg to lb`.
    Conversion,
    /// `5尺3寸`.
    JapaneseLength,
    /// `6帖`.
    TatamiArea,
    /// `10坪`.
    TsuboArea,
    /// `50㎡`.
    SquareMeter,
    /// `20°C`, `摂氏20度`.
    Temperature,
    /// `5 m 3 cm`, `2 lb 3 oz`.
    CompoundQuantity,
    /// Any unit in the registry, including caller-supplied custom units.
    RegisteredQuantity,
    /// `180cm`, `1m80`.
    MetricLength,
    /// `5 kg`, `3 lb`.
    Mass,
    /// `3pm Europe/Paris`.
    TimezoneClock,
    /// `3pm`, `15:30`.
    Clock,
    /// `1h30`, `PT1H30M`.
    Duration,
    /// `5'11"`, `5 ft 11 in`.
    FeetInches,
    /// `1.5 cups`.
    Cups,
    /// `$5`, `1,234円`.
    Currency,
    /// A number whose grouping has more than one reading, such as `1,234`.
    AmbiguousNumber,
    /// A number whose shape settles it, such as `1234` or `1.23`.
    PlainNumber,
    /// A unit spelling close enough to a known one to suggest a correction.
    TypoCorrectedQuantity,
    /// A timezone suffix this build cannot resolve — reported, not guessed.
    UnsupportedTimezone,
}

/// The single definition of the order the grammars are tried in.
///
/// Every entry point walks this slice. None of them keeps an order of its own.
pub(crate) const GRAMMAR_ORDER: [Grammar; 28] = [
    Grammar::Qualified,
    Grammar::Fuzzy,
    Grammar::SlashDateOrFraction,
    Grammar::NumericSlashDate,
    Grammar::RelativeDate,
    Grammar::PlusMinusRange,
    Grammar::UpperBoundRange,
    Grammar::Range,
    Grammar::Conversion,
    Grammar::JapaneseLength,
    Grammar::TatamiArea,
    Grammar::TsuboArea,
    Grammar::SquareMeter,
    Grammar::Temperature,
    Grammar::CompoundQuantity,
    Grammar::RegisteredQuantity,
    Grammar::MetricLength,
    Grammar::Mass,
    Grammar::TimezoneClock,
    Grammar::Clock,
    Grammar::Duration,
    Grammar::FeetInches,
    Grammar::Cups,
    Grammar::Currency,
    Grammar::AmbiguousNumber,
    Grammar::PlainNumber,
    Grammar::TypoCorrectedQuantity,
    Grammar::UnsupportedTimezone,
];

/// An entry point, named by the subset of grammars it reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Entry {
    /// [`parse`] — every grammar.
    General,
    /// [`parse_quantity_fast`] — measurement grammars only.
    Quantity,
    /// [`parse_number_fast`] and the editor's bare-number fallback.
    Number,
    /// [`parse_date_fast`].
    Date,
}

impl Entry {
    /// Every entry point, so a test can enumerate them without a second list.
    #[cfg(test)]
    pub(crate) const ALL: [Entry; 4] =
        [Entry::General, Entry::Quantity, Entry::Number, Entry::Date];

    /// Whether this entry point reads `grammar` at all.
    ///
    /// This is a membership question, never an ordering one: the answer cannot
    /// move a grammar earlier or later than [`GRAMMAR_ORDER`] puts it.
    pub(crate) fn reads(self, grammar: Grammar) -> bool {
        match self {
            Entry::General => true,
            Entry::Quantity => matches!(
                grammar,
                Grammar::Qualified
                    | Grammar::Fuzzy
                    | Grammar::JapaneseLength
                    | Grammar::TatamiArea
                    | Grammar::TsuboArea
                    | Grammar::SquareMeter
                    | Grammar::Temperature
                    | Grammar::CompoundQuantity
                    | Grammar::RegisteredQuantity
                    | Grammar::MetricLength
                    | Grammar::Mass
                    | Grammar::Clock
                    | Grammar::Duration
                    | Grammar::FeetInches
                    | Grammar::Cups
                    | Grammar::Currency
                    | Grammar::TypoCorrectedQuantity
            ),
            Entry::Number => matches!(grammar, Grammar::AmbiguousNumber | Grammar::PlainNumber),
            Entry::Date => matches!(grammar, Grammar::NumericSlashDate | Grammar::RelativeDate),
        }
    }

    /// What a result with no reading says when this entry point read nothing.
    pub(crate) fn no_match_reason(self) -> &'static str {
        match self {
            Entry::General => "no supported reading matched",
            Entry::Quantity => "no supported quantity matched",
            Entry::Number => "no supported number matched",
            Entry::Date => "no supported date matched",
        }
    }
}

/// Where a plain number's context-implied millimetre alternative comes from.
///
/// The broad entry points take it from the caller's declaration; the dimension
/// editor takes it from the label next to the candidate. Only the source of the
/// dimension differs, so the two stay one grammar with two sinks rather than
/// two copies of the number grammar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlainNumberSink {
    /// Decided by [`ParseCtx::expect`] and [`ParseCtx::expected_dimensions`].
    Contextual,
    /// Decided by the dimension label the editor scanner found.
    EditorDimension(Dimension),
}

impl PlainNumberSink {
    pub(crate) fn set(self, text: &str, ctx: &ParseCtx, reading: Reading, parsed: &mut Parsed) {
        match self {
            PlainNumberSink::Contextual => set_plain_number_result(text, ctx, reading, parsed),
            PlainNumberSink::EditorDimension(dimension) => {
                set_editor_plain_number_result(text, dimension, reading, parsed)
            }
        }
    }
}

// Records what the dispatcher did, for the tests that police it.
//
// A prose claim that every entry point walks `GRAMMAR_ORDER` stops being true
// the moment someone adds one `if let Some(..)` beside a call to `dispatch`.
// The trace makes the claim checkable: a reading that no grammar in this module
// produced leaves no winner behind, and
// `a_reading_is_only_ever_produced_by_the_one_grammar_order` fails.
#[cfg(test)]
thread_local! {
    pub(crate) static TRACE: std::cell::RefCell<Vec<Trace>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// One walk of the order: what was asked, and what settled it.
#[cfg(test)]
#[derive(Clone, Debug)]
pub(crate) struct Trace {
    pub(crate) entry: Entry,
    pub(crate) attempted: Vec<Grammar>,
    pub(crate) settled_by: Option<Grammar>,
}

/// Runs `body` with a fresh trace and returns what the dispatcher recorded.
#[cfg(test)]
pub(crate) fn traced<T>(body: impl FnOnce() -> T) -> (T, Vec<Trace>) {
    TRACE.with(|trace| trace.borrow_mut().clear());
    let value = body();
    let recorded = TRACE.with(|trace| trace.borrow().clone());
    (value, recorded)
}

/// Tries every grammar `entry` reads, in [`GRAMMAR_ORDER`], and stops at the
/// first one that has something to say about `trimmed`.
pub(crate) fn dispatch(
    entry: Entry,
    sink: PlainNumberSink,
    trimmed: &str,
    ctx: &ParseCtx,
    parsed: &mut Parsed,
) {
    #[cfg(test)]
    let mut trace = Trace {
        entry,
        attempted: Vec::new(),
        settled_by: None,
    };

    let features = InputFeatures::new(trimmed);
    for grammar in GRAMMAR_ORDER {
        if !entry.reads(grammar) {
            continue;
        }
        if !features.may_hold(grammar) {
            continue;
        }
        #[cfg(test)]
        trace.attempted.push(grammar);
        if try_grammar(grammar, sink, trimmed, ctx, parsed) {
            #[cfg(test)]
            {
                trace.settled_by = Some(grammar);
                TRACE.with(|recorded| recorded.borrow_mut().push(trace));
            }
            return;
        }
    }
    #[cfg(test)]
    TRACE.with(|recorded| recorded.borrow_mut().push(trace));

    if entry == Entry::General && features.maybe_suggestion {
        parsed.suggestions = suggestions_for(trimmed);
    }
    parsed
        .findings
        .skipped
        .push(skipped(trimmed, entry.no_match_reason()));
}

/// Runs one grammar, and says whether it settled the input.
///
/// `true` means the input is settled — either read, or refused with the refusal
/// on `parsed`. A refusal stops the walk exactly as a reading does: the input
/// was recognized, and falling through to a looser grammar would answer a
/// question the caller already had answered.
fn try_grammar(
    grammar: Grammar,
    sink: PlainNumberSink,
    trimmed: &str,
    ctx: &ParseCtx,
    parsed: &mut Parsed,
) -> bool {
    match grammar {
        Grammar::Qualified => {
            let Some(result) = parse_qualified_reading(trimmed, ctx) else {
                return false;
            };
            admit_approximate(
                Approximate::Qualified,
                ctx,
                trimmed,
                result.reading,
                result.approximations,
                parsed,
            );
            true
        }
        Grammar::Fuzzy => {
            let Some(result) = parse_fuzzy_reading(trimmed, ctx) else {
                return false;
            };
            admit_approximate(
                Approximate::Fuzzy,
                ctx,
                trimmed,
                result.reading,
                result.approximations,
                parsed,
            );
            true
        }
        Grammar::SlashDateOrFraction => {
            let Some(ambiguous) = parse_ambiguous_slash_date_or_fraction(trimmed, ctx) else {
                return false;
            };
            set_ambiguous(ambiguous, parsed);
            true
        }
        // A three-part slash date has the same day-first/month-first question
        // the two-part form has, so it is answered the same way: both readings
        // stay on the table and the choice is reported.
        Grammar::NumericSlashDate => {
            let Some(ambiguous) = parse_ambiguous_numeric_slash_date(trimmed, ctx) else {
                return false;
            };
            set_ambiguous(ambiguous, parsed);
            true
        }
        Grammar::RelativeDate => set_if_read(parse_relative_date(trimmed, ctx), parsed),
        Grammar::PlusMinusRange => {
            accept_range(parse_plus_minus_range(trimmed, ctx), trimmed, ctx, parsed)
        }
        Grammar::UpperBoundRange => {
            accept_range(parse_upper_bound_range(trimmed, ctx), trimmed, ctx, parsed)
        }
        Grammar::Range => {
            let Some(reading) = parse_range(trimmed, ctx) else {
                return false;
            };
            if !ctx.accept.ranges {
                reject_candidate(
                    parsed,
                    trimmed,
                    reading,
                    "range readings are disabled by acceptance policy",
                );
                return true;
            }
            // An endpoint that is a three-part slash date has two readings, and
            // the range collapsed it to one. Report the choice rather than let
            // it vanish.
            parsed
                .findings
                .ambiguities
                .extend(range_endpoint_ambiguities(trimmed, ctx));
            parsed.best = Some(reading);
            true
        }
        Grammar::Conversion => {
            let Some(reading) = parse_conversion_request(trimmed, ctx) else {
                return false;
            };
            if !ctx.accept.conversions {
                reject_candidate(
                    parsed,
                    trimmed,
                    reading,
                    "conversion readings are disabled by acceptance policy",
                );
                return true;
            }
            parsed.best = Some(reading);
            true
        }
        Grammar::JapaneseLength => set_approximated(
            parse_japanese_length(trimmed),
            trimmed,
            "Japanese customary length converted to SI meters.",
            parsed,
        ),
        Grammar::TatamiArea => set_approximated(
            parse_tatami_area(trimmed),
            trimmed,
            "Tatami area uses a trade-custom regional approximation of 1.62 m2.",
            parsed,
        ),
        Grammar::TsuboArea => set_approximated(
            parse_tsubo_area(trimmed),
            trimmed,
            "Tsubo area converted through Japanese customary area.",
            parsed,
        ),
        Grammar::SquareMeter => set_if_read(parse_square_meter(trimmed), parsed),
        Grammar::Temperature => set_if_read(parse_temperature(trimmed), parsed),
        Grammar::CompoundQuantity => match parse_compound_registered_quantity_ctx(trimmed, ctx) {
            CompoundOutcome::Reading(reading) => {
                if !ctx.accept.compounds {
                    reject_candidate(
                        parsed,
                        trimmed,
                        reading,
                        "compound quantity readings are disabled by acceptance policy",
                    );
                    return true;
                }
                note_unit_approximation(parsed, trimmed, &reading);
                parsed.best = Some(reading);
                true
            }
            CompoundOutcome::Malformed(reason) => {
                note_malformed_compound(parsed, trimmed, reason);
                true
            }
            CompoundOutcome::NotCompound => false,
        },
        Grammar::RegisteredQuantity => {
            let Some(reading) = parse_registered_quantity(trimmed, ctx) else {
                return false;
            };
            note_unit_approximation(parsed, trimmed, &reading);
            parsed.best = Some(reading);
            true
        }
        Grammar::MetricLength => set_if_read(parse_metric_length(trimmed), parsed),
        Grammar::Mass => set_if_read(parse_mass(trimmed), parsed),
        Grammar::TimezoneClock => {
            let Some((reading, day_shift)) = parse_timezone_clock_time(trimmed, ctx) else {
                return false;
            };
            if day_shift != 0 {
                let direction = if day_shift < 0 { "previous" } else { "next" };
                parsed.findings.approximations.push(approximation_with_span(
                    trimmed,
                    &format!(
                        "time of day only; converting to UTC moves it to the {direction} civil day"
                    ),
                    span(trimmed),
                ));
            }
            parsed.best = Some(reading);
            true
        }
        Grammar::Clock => set_if_read(parse_clock_time(trimmed), parsed),
        Grammar::Duration => set_if_read(parse_duration(trimmed), parsed),
        Grammar::FeetInches => {
            // An apostrophe is a foot mark and a Swiss digit group separator,
            // and one function answers which for every entry point.
            if let Some(readings) = apostrophe_readings(trimmed, ctx) {
                readings.report(ApostropheBest::Feet, trimmed, parsed);
                return true;
            }
            set_if_read(parse_feet_inches(trimmed), parsed)
        }
        Grammar::Cups => {
            let Some((best, alternatives, ambiguity)) = parse_cups(trimmed, ctx) else {
                return false;
            };
            parsed.best = Some(best);
            parsed.alternatives = alternatives;
            parsed.findings.ambiguities.push(ambiguity);
            true
        }
        Grammar::Currency => {
            let Some((best, alternatives, ambiguity)) = parse_currency(trimmed, ctx) else {
                return false;
            };
            parsed.best = Some(best);
            parsed.alternatives = alternatives;
            if let Some(ambiguity) = ambiguity {
                parsed.findings.ambiguities.push(ambiguity);
            }
            true
        }
        Grammar::AmbiguousNumber => {
            if let Some(readings) = apostrophe_readings(trimmed, ctx) {
                readings.report(ApostropheBest::Number, trimmed, parsed);
                return true;
            }
            let Some(ambiguous) = parse_ambiguous_number(trimmed, ctx) else {
                return false;
            };
            set_ambiguous(ambiguous, parsed);
            true
        }
        Grammar::PlainNumber => {
            let Some(reading) = parse_plain_number_ctx(trimmed, ctx) else {
                return false;
            };
            sink.set(trimmed, ctx, reading, parsed);
            true
        }
        Grammar::TypoCorrectedQuantity => {
            let Some((reading, suggestion, unit_text)) =
                parse_typo_corrected_quantity_ctx(trimmed, ctx)
            else {
                return false;
            };
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
            true
        }
        Grammar::UnsupportedTimezone => {
            let Some(timezone) = unsupported_timezone_suffix(trimmed) else {
                return false;
            };
            parsed.findings.skipped.push(skipped_with_span(
                timezone,
                "unsupported timezone conversion requires an explicit adapter policy",
                IssueCode::TimezoneUnsupported,
                span_token_in(trimmed, timezone),
            ));
            true
        }
    }
}

fn set_if_read(reading: Option<Reading>, parsed: &mut Parsed) -> bool {
    match reading {
        Some(reading) => {
            parsed.best = Some(reading);
            true
        }
        None => false,
    }
}

fn set_approximated(
    reading: Option<Reading>,
    trimmed: &str,
    reason: &str,
    parsed: &mut Parsed,
) -> bool {
    match reading {
        Some(reading) => {
            parsed
                .findings
                .approximations
                .push(approximation(trimmed, reason));
            parsed.best = Some(reading);
            true
        }
        None => false,
    }
}

fn set_ambiguous(ambiguous: AmbiguousParse, parsed: &mut Parsed) {
    parsed.best = ambiguous.best;
    parsed.alternatives = ambiguous.alternatives;
    parsed.findings.ambiguities.push(ambiguous.ambiguity);
}

fn accept_range(
    reading: Option<Reading>,
    trimmed: &str,
    ctx: &ParseCtx,
    parsed: &mut Parsed,
) -> bool {
    let Some(reading) = reading else {
        return false;
    };
    if !ctx.accept.ranges {
        reject_candidate(
            parsed,
            trimmed,
            reading,
            "range readings are disabled by acceptance policy",
        );
        return true;
    }
    parsed.best = Some(reading);
    true
}

impl InputFeatures {
    /// The cheap precheck for one grammar: whether the input can hold it at all.
    ///
    /// This only ever skips work. A grammar whose precheck is `true` is still
    /// tried and may still decline, so this cannot decide anything the grammar
    /// itself would have decided differently.
    pub(crate) fn may_hold(&self, grammar: Grammar) -> bool {
        match grammar {
            Grammar::Qualified | Grammar::Fuzzy => true,
            Grammar::SlashDateOrFraction => self.has_slash,
            Grammar::NumericSlashDate | Grammar::RelativeDate => self.maybe_date,
            Grammar::PlusMinusRange | Grammar::UpperBoundRange | Grammar::Range => self.maybe_range,
            Grammar::Conversion => self.maybe_conversion,
            Grammar::JapaneseLength => self.maybe_japanese_length,
            Grammar::TatamiArea => self.maybe_tatami,
            Grammar::TsuboArea => self.maybe_tsubo,
            Grammar::SquareMeter => self.maybe_area,
            Grammar::Temperature => self.maybe_temperature,
            Grammar::CompoundQuantity => self.maybe_compound_quantity,
            Grammar::RegisteredQuantity | Grammar::TypoCorrectedQuantity => self.maybe_quantity,
            Grammar::MetricLength => self.maybe_metric_length,
            Grammar::Mass => self.maybe_mass,
            Grammar::TimezoneClock | Grammar::UnsupportedTimezone => self.maybe_timezone_clock,
            Grammar::Clock => self.maybe_clock,
            Grammar::Duration => self.maybe_duration,
            Grammar::FeetInches => self.maybe_feet_inches,
            Grammar::Cups => self.maybe_cups,
            Grammar::Currency => self.maybe_currency,
            Grammar::AmbiguousNumber | Grammar::PlainNumber => self.maybe_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inputs chosen to reach as many grammars as possible from every entry
    /// point, including the ones that used to disagree.
    const CORPUS: [&str; 23] = [
        "3 m 5 m",
        "1'234",
        "5'11\"",
        "5尺3寸",
        "2 lb 3 oz",
        "1h30",
        "5 m 3 cm",
        "1 23 456",
        "180cm",
        "1m80",
        "100-120㎡",
        "1,234",
        "5-10 kg",
        "3pm-4pm",
        "2〜3日",
        "6帖",
        "10坪",
        "20°C",
        "$5",
        "1.5 cups",
        "next friday",
        "3pm Europe/Paris",
        "meters meters meters",
    ];

    fn every_entry_point(text: &str) -> Vec<Parsed> {
        vec![
            parse(text, None),
            parse_quantity_fast(text, None),
            parse_number_fast(text, None),
            parse_date_fast(text, None),
        ]
    }

    /// The order exists once, and every entry point's subset is drawn from it.
    ///
    /// `Entry::reads` can only answer yes or no about a grammar; it has no way
    /// to express "but try this one first", which is the whole point.
    #[test]
    fn the_grammar_order_is_written_down_exactly_once() {
        for (index, grammar) in GRAMMAR_ORDER.iter().enumerate() {
            assert_eq!(
                GRAMMAR_ORDER.iter().position(|other| other == grammar),
                Some(index),
                "{grammar:?} appears in the order twice"
            );
        }

        for entry in Entry::ALL {
            let read: Vec<Grammar> = GRAMMAR_ORDER
                .into_iter()
                .filter(|grammar| entry.reads(*grammar))
                .collect();
            assert!(!read.is_empty(), "{entry:?} reads no grammar at all");
            // A subset, never a reordering: the grammars this entry point
            // reads appear in the same relative order the one table puts them.
            assert!(
                read.windows(2).all(|pair| {
                    let earlier = GRAMMAR_ORDER.iter().position(|g| *g == pair[0]);
                    let later = GRAMMAR_ORDER.iter().position(|g| *g == pair[1]);
                    earlier < later
                }),
                "{entry:?} does not read the grammars in the one order"
            );
        }

        // Every grammar is reachable from at least one entry point, so a
        // grammar cannot be added to the order and then never tried.
        for grammar in GRAMMAR_ORDER {
            assert!(
                Entry::ALL.iter().any(|entry| entry.reads(grammar)),
                "{grammar:?} is in the order but no entry point reads it"
            );
        }
    }

    /// A reading is only ever produced by walking the one order.
    ///
    /// This is what stops the unification from decaying back into what it
    /// replaced. An entry point that grows its own `if let Some(reading) =
    /// parse_something(..)` beside the call to [`dispatch`] would set `best`
    /// with no grammar recorded as having settled the input, and this fails.
    #[test]
    fn a_reading_is_only_ever_produced_by_the_one_grammar_order() {
        for text in CORPUS {
            for (index, entry) in Entry::ALL.iter().enumerate() {
                let (parsed, traces) = traced(|| every_entry_point(text).swap_remove(index));

                assert!(
                    !traces.is_empty(),
                    "{entry:?} parsed {text:?} without walking the grammar order"
                );
                for trace in &traces {
                    // Nothing outside the entry point's declared subset was
                    // even asked, and nothing was asked out of order.
                    let mut cursor = 0;
                    for grammar in &trace.attempted {
                        assert!(
                            trace.entry.reads(*grammar),
                            "{:?} tried {grammar:?}, which it does not read",
                            trace.entry
                        );
                        let position = GRAMMAR_ORDER
                            .iter()
                            .position(|other| other == grammar)
                            .unwrap_or_else(|| panic!("{grammar:?} is not in GRAMMAR_ORDER"));
                        assert!(
                            position >= cursor,
                            "{:?} tried {grammar:?} out of the one order on {text:?}",
                            trace.entry
                        );
                        cursor = position + 1;
                    }
                }

                if parsed.best.is_some() {
                    assert!(
                        traces.iter().any(|trace| trace.settled_by.is_some()),
                        "{entry:?} read {text:?} but no grammar in the order settled it"
                    );
                }
            }
        }
    }

    /// The strictness gate is one function, and every entry point obeys it.
    ///
    /// The quantity entry point used to keep a copy that read only
    /// `ctx.accept.fuzzy`, so `Strictness::Strict` refused a fuzzy term through
    /// `parse` and returned it through `parse_quantity_fast`.
    #[test]
    fn the_approximation_gate_is_one_function_for_every_entry_point() {
        let ctx_for = |strictness: Strictness, fuzzy: bool| ParseCtx {
            strictness,
            accept: AcceptOptions {
                fuzzy,
                ..AcceptOptions::default()
            },
            fuzzy_profiles: vec![FuzzyProfile::new(
                "size",
                Dimension::Length,
                "m",
                &[FuzzyTerm::new("ちょい", 1.0, 3.0)],
            )],
            ..ParseCtx::default()
        };

        for strictness in [
            Strictness::Forgiving,
            Strictness::Confirm,
            Strictness::Strict,
        ] {
            for accept_fuzzy in [true, false] {
                for (text, kind) in [
                    ("ちょい", Approximate::Fuzzy),
                    ("about 20kg", Approximate::Qualified),
                ] {
                    let ctx = ctx_for(strictness, accept_fuzzy);
                    let expected = approximate_admission(kind, &ctx);
                    for parsed in [
                        parse(text, Some(ctx.clone())),
                        parse_quantity_fast(text, Some(ctx.clone())),
                    ] {
                        let label = format!("{text:?} {strictness:?} accept_fuzzy={accept_fuzzy}");
                        match expected {
                            Admission::Accept => {
                                assert!(parsed.best.is_some(), "{label}: expected a reading")
                            }
                            Admission::RequireConfirmation => {
                                assert!(parsed.best.is_none(), "{label}: expected a refusal");
                                assert!(
                                    parsed
                                        .findings
                                        .skipped
                                        .iter()
                                        .any(|issue| issue.code == IssueCode::Approximation),
                                    "{label}: expected an APPROXIMATION refusal"
                                );
                            }
                            Admission::RefusedByPolicy => {
                                assert!(parsed.best.is_none(), "{label}: expected a refusal");
                                assert!(
                                    parsed
                                        .findings
                                        .skipped
                                        .iter()
                                        .any(|issue| issue.code == IssueCode::RejectedByPolicy),
                                    "{label}: expected a policy refusal"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
