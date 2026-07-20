//! Completion candidates for input fields.
//!
//! [`complete`] completes the token being typed; [`complete_readings`] ranks
//! whole interpretations of what has been typed so far.

use crate::*;

/// Completes the unit, date, time, or currency token at the end of `prefix`.
///
/// Candidates are drawn from the unit registry, any [`ParseCtx::custom_units`],
/// and the built-in date/time/currency vocabularies, then ranked by how much of
/// the candidate the prefix already covers (an exact match scores `1.0`). At
/// most 24 candidates are returned.
///
/// Returns an empty vector when there is no token to complete.
pub fn complete(prefix: &str, ctx: Option<ParseCtx>) -> Vec<Completion> {
    let ctx = ctx.unwrap_or_default();
    let Some(raw_prefix) = completion_prefix(prefix) else {
        return Vec::new();
    };
    let normalized_prefix = normalize_alias(raw_prefix);
    if normalized_prefix.is_empty() {
        return Vec::new();
    }

    let mut completions = Vec::new();
    for unit in UNIT_DEFS {
        for alias in unit.aliases {
            push_completion(
                &mut completions,
                CompletionCandidate {
                    value: alias,
                    canonical: Some(unit.id),
                    kind: CompletionKind::Unit,
                    dimension: Some(unit.dimension),
                },
                &normalized_prefix,
                &ctx,
            );
        }
    }

    for custom in &ctx.custom_units {
        for alias in &custom.aliases {
            push_owned_completion(
                &mut completions,
                alias,
                Some(&custom.id),
                CompletionKind::Unit,
                Some(custom.dimension),
                &normalized_prefix,
                &ctx,
            );
        }
    }

    for (value, canonical, dimension) in LEGACY_UNIT_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: Some(canonical),
                kind: CompletionKind::Unit,
                dimension: Some(*dimension),
            },
            &normalized_prefix,
            &ctx,
        );
    }

    for value in DATE_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: None,
                kind: CompletionKind::Date,
                dimension: None,
            },
            &normalized_prefix,
            &ctx,
        );
    }

    for value in TIME_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: None,
                kind: CompletionKind::Time,
                dimension: Some(Dimension::Time),
            },
            &normalized_prefix,
            &ctx,
        );
    }

    for (value, canonical) in CURRENCY_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: Some(canonical),
                kind: CompletionKind::Currency,
                dimension: Some(Dimension::Currency),
            },
            &normalized_prefix,
            &ctx,
        );
    }

    completions.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.value.cmp(&right.value))
            .then_with(|| left.kind.as_str().cmp(right.kind.as_str()))
    });
    completions.truncate(24);
    completions
}

/// Ranks the plausible whole readings of `text`, for a preview or picker UI.
///
/// The best reading and its alternatives are returned with their parse
/// confidence as the score and a `reason` naming where each came from. When the
/// text is a bare number, candidate units are fanned out so the user can pick
/// the unit they meant instead of the parser assuming one.
pub fn complete_readings(text: &str, ctx: Option<ParseCtx>) -> Vec<CompletionReading> {
    let ctx = ctx.unwrap_or_default();
    let parsed = parse(text, Some(ctx.clone()));
    let mut completions = Vec::new();
    if let Some(best) = parsed.best {
        completions.push(CompletionReading {
            text: text.to_owned(),
            score: best.confidence.unwrap_or(0.0),
            reading: best,
            reason: "best".to_owned(),
        });
    }
    for alternative in parsed.alternatives {
        completions.push(CompletionReading {
            text: text.to_owned(),
            score: alternative.confidence.unwrap_or(0.0),
            reading: alternative,
            reason: "alternative".to_owned(),
        });
    }

    if let Some(value) = parse_number_ctx(text, &ctx) {
        push_unit_fanout(text, value, &ctx, &mut completions);
    }

    completions.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.text.cmp(&right.text))
    });
    completions.truncate(24);
    completions
}

pub(crate) fn push_unit_fanout(
    text: &str,
    value: f64,
    ctx: &ParseCtx,
    completions: &mut Vec<CompletionReading>,
) {
    let mut units = units_for_completion_fanout(ctx);
    units.truncate(12);
    for unit in units {
        push_completion_reading_if_new(
            completions,
            CompletionReading {
                text: format!("{} {}", text.trim(), unit.id),
                reading: Reading::quantity(
                    value * unit.factor,
                    unit.canonical_unit,
                    unit.dimension,
                    unit.provenance,
                    unit.approximate,
                    0.45,
                ),
                score: 0.45,
                reason: "unit_fanout".to_owned(),
            },
        );
    }

    for unit in &ctx.custom_units {
        if let Some(expected) = ctx.expected_dimension
            && expected != unit.dimension
        {
            continue;
        }
        let mut reading = Reading::quantity(
            value * unit.factor,
            &unit.canonical_unit,
            unit.dimension,
            Provenance::TradeCustom,
            unit.approximate,
            0.42,
        );
        reading.custom_kind = unit.kind_id.clone();
        push_completion_reading_if_new(
            completions,
            CompletionReading {
                text: format!("{} {}", text.trim(), unit.id),
                reading,
                score: 0.42,
                reason: "custom_unit_fanout".to_owned(),
            },
        );
    }
}

pub(crate) fn push_completion_reading_if_new(
    completions: &mut Vec<CompletionReading>,
    completion: CompletionReading,
) {
    if completions
        .iter()
        .any(|existing| existing.text == completion.text && existing.reading == completion.reading)
    {
        return;
    }
    completions.push(completion);
}

pub(crate) fn units_for_completion_fanout(ctx: &ParseCtx) -> Vec<&'static UnitDef> {
    let dimension = ctx.expected_dimension;
    let mut units = Vec::new();
    for unit in UNIT_DEFS {
        if dimension.is_none_or(|expected| expected == unit.dimension) {
            units.push(unit);
        }
    }
    units
}

pub(crate) const LEGACY_UNIT_COMPLETIONS: &[(&str, &str, Dimension)] = &[
    ("shaku", "shaku", Dimension::Length),
    ("尺", "shaku", Dimension::Length),
    ("sun", "sun", Dimension::Length),
    ("寸", "sun", Dimension::Length),
    ("ken", "ken", Dimension::Length),
    ("間", "ken", Dimension::Length),
    ("tsubo", "tsubo", Dimension::Area),
    ("坪", "tsubo", Dimension::Area),
    ("tatami", "tatami", Dimension::Area),
    ("帖", "tatami", Dimension::Area),
    ("畳", "tatami", Dimension::Area),
    ("celsius", "C", Dimension::Temperature),
    ("°C", "C", Dimension::Temperature),
    ("℃", "C", Dimension::Temperature),
    ("fahrenheit", "F", Dimension::Temperature),
    ("°F", "F", Dimension::Temperature),
    ("℉", "F", Dimension::Temperature),
    ("kelvin", "K", Dimension::Temperature),
    ("摂氏", "C", Dimension::Temperature),
    ("華氏", "F", Dimension::Temperature),
];

pub(crate) const DATE_COMPLETIONS: &[&str] = &[
    "today",
    "tomorrow",
    "yesterday",
    "next monday",
    "next tuesday",
    "next wednesday",
    "next thursday",
    "next friday",
    "next saturday",
    "next sunday",
    "this friday",
    "last friday",
    "mañana",
    "demain",
    "amanhã",
    "明天",
    "今日",
    "明日",
    "昨日",
    "来週月曜日",
    "来週火曜日",
    "来週水曜日",
    "来週木曜日",
    "来週金曜日",
    "来週土曜日",
    "来週日曜日",
    "下周五",
];

pub(crate) const TIME_COMPLETIONS: &[&str] = &["noon", "midnight", "午後3時", "午前9時"];

pub(crate) const CURRENCY_COMPLETIONS: &[(&str, &str)] = &[
    ("USD", "USD"),
    ("EUR", "EUR"),
    ("GBP", "GBP"),
    ("JPY", "JPY"),
    ("dollar", "USD"),
    ("dollars", "USD"),
    ("bucks", "USD"),
    ("euro", "EUR"),
    ("euros", "EUR"),
    ("pound", "GBP"),
    ("pounds", "GBP"),
    ("quid", "GBP"),
    ("yen", "JPY"),
    ("円", "JPY"),
    ("pence", "GBP"),
    ("cents", "USD"),
];

pub(crate) struct CompletionCandidate<'a> {
    pub(crate) value: &'a str,
    pub(crate) canonical: Option<&'a str>,
    pub(crate) kind: CompletionKind,
    pub(crate) dimension: Option<Dimension>,
}

pub(crate) fn completion_prefix(input: &str) -> Option<&str> {
    input.split_whitespace().last()
}

pub(crate) fn push_completion(
    completions: &mut Vec<Completion>,
    candidate: CompletionCandidate<'_>,
    normalized_prefix: &str,
    ctx: &ParseCtx,
) {
    push_owned_completion(
        completions,
        candidate.value,
        candidate.canonical,
        candidate.kind,
        candidate.dimension,
        normalized_prefix,
        ctx,
    );
}

pub(crate) fn push_owned_completion(
    completions: &mut Vec<Completion>,
    value: &str,
    canonical: Option<&str>,
    kind: CompletionKind,
    dimension: Option<Dimension>,
    normalized_prefix: &str,
    ctx: &ParseCtx,
) {
    if !completion_allowed(kind, dimension, ctx) {
        return;
    }
    let normalized_value = normalize_alias(value);
    if !normalized_value.starts_with(normalized_prefix) {
        return;
    }
    let score = completion_score(normalized_prefix, &normalized_value);
    if completions.iter().any(|existing| {
        existing.kind == kind
            && existing.value == value
            && existing.canonical.as_deref() == canonical
    }) {
        return;
    }
    completions.push(Completion {
        value: value.to_owned(),
        canonical: canonical.map(str::to_owned),
        kind,
        dimension,
        score,
    });
}

pub(crate) fn completion_allowed(
    kind: CompletionKind,
    dimension: Option<Dimension>,
    ctx: &ParseCtx,
) -> bool {
    if let Some(expected_dimension) = ctx.expected_dimension
        && dimension != Some(expected_dimension)
    {
        return false;
    }

    match ctx.expect {
        Some(Kind::Date) => kind == CompletionKind::Date,
        Some(Kind::Recurrence) => false,
        Some(Kind::Number) => false,
        Some(Kind::Quantity) => matches!(
            kind,
            CompletionKind::Unit | CompletionKind::Time | CompletionKind::Currency
        ),
        Some(Kind::Range) | None => true,
    }
}

pub(crate) fn completion_score(prefix: &str, value: &str) -> f64 {
    if prefix == value {
        1.0
    } else {
        0.6 + 0.4 * prefix.len() as f64 / value.len().max(prefix.len()) as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completes_units_dates_and_custom_units() {
        let metric = complete("10 met", None);
        assert_eq!(metric[0].value, "meter");
        assert_eq!(metric[0].canonical.as_deref(), Some("m"));
        assert_eq!(metric[0].kind, CompletionKind::Unit);
        assert_eq!(metric[0].dimension, Some(Dimension::Length));

        let date = complete(
            "tom",
            Some(ParseCtx {
                expect: Some(Kind::Date),
                ..ParseCtx::default()
            }),
        );
        assert!(date.iter().any(|item| item.value == "tomorrow"));
        assert!(date.iter().all(|item| item.kind == CompletionKind::Date));

        let area = complete(
            "坪",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                expected_dimension: Some(Dimension::Area),
                ..ParseCtx::default()
            }),
        );
        assert!(
            area.iter()
                .any(|item| item.value == "坪" && item.canonical.as_deref() == Some("tsubo"))
        );

        let custom = complete(
            "smo",
            Some(ParseCtx {
                custom_units: vec![CustomUnit::new(
                    "smoot",
                    "m",
                    &["smoot", "smoots"],
                    Dimension::Length,
                    1.7018,
                )],
                ..ParseCtx::default()
            }),
        );
        assert!(
            custom
                .iter()
                .any(|item| item.value == "smoot" && item.canonical.as_deref() == Some("smoot"))
        );

        let temperature = complete(
            "cel",
            Some(ParseCtx {
                expected_dimension: Some(Dimension::Temperature),
                ..ParseCtx::default()
            }),
        );
        assert!(
            temperature
                .iter()
                .any(|item| item.value == "celsius" && item.canonical.as_deref() == Some("C"))
        );
    }
}
