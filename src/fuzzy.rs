use crate::*;

pub(crate) fn parse_qualified_reading(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    let (qualifier, rest) = strip_approximate_qualifier(text)?;
    let mut reading = parse_endpoint(rest, ctx)?;
    mark_approximate(&mut reading);
    Some(ParsedReading {
        reading,
        approximations: vec![approximation_with_span(
            qualifier,
            "Approximate qualifier was preserved as an approximation finding.",
            span_in(text, qualifier),
        )],
    })
}

pub(crate) fn strip_approximate_qualifier(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    for prefix in [
        "approximately ",
        "approx. ",
        "approx ",
        "around ",
        "roughly ",
        "about ",
    ] {
        if let Some(rest) = strip_prefix_ascii_case(trimmed, prefix)
            && !rest.trim().is_empty()
        {
            return Some((trimmed.get(..prefix.len())?.trim(), rest.trim()));
        }
    }
    if let Some(rest) = trimmed.strip_prefix('約')
        && !rest.trim().is_empty()
    {
        return Some(("約", rest.trim()));
    }
    for suffix in [" (approx.)", " approx.", " approximately"] {
        if let Some(rest) = strip_suffix_ascii_case(trimmed, suffix)
            && !rest.trim().is_empty()
        {
            return Some((
                trimmed.get(trimmed.len() - suffix.len()..)?.trim(),
                rest.trim(),
            ));
        }
    }
    None
}

pub(crate) fn parse_fuzzy_reading(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    if let Some(rest) = strip_prefix_ascii_case(text.trim(), "a few ")
        && !rest.trim().is_empty()
    {
        let from = parse_endpoint(&format!("2 {}", rest.trim()), ctx)?;
        let to = parse_endpoint(&format!("4 {}", rest.trim()), ctx)?;
        if from.kind == to.kind && from.dimension == to.dimension {
            let mut reading = Reading::range(from, to, 0.72);
            mark_approximate(&mut reading);
            return Some(ParsedReading {
                reading,
                approximations: vec![approximation_with_span(
                    "a few",
                    "Fuzzy small-count phrase normalized to a 2 to 4 range.",
                    span_in(text, "a few"),
                )],
            });
        }
    }

    parse_custom_fuzzy_profile(text, ctx).or_else(|| parse_fuzzy_temperature(text, ctx))
}

pub(crate) fn parse_custom_fuzzy_profile(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    let normalized = normalize_alias(text);
    for profile in &ctx.fuzzy_profiles {
        if !ctx.expected_dimensions.allows(profile.dimension) {
            continue;
        }
        let Some(target_unit) = unit_by_alias(&profile.unit) else {
            continue;
        };
        if target_unit.dimension != profile.dimension {
            continue;
        }
        for term in &profile.terms {
            if normalize_alias(&term.term) != normalized {
                continue;
            }
            let mut reading = Reading::range(
                Reading::quantity(
                    term.low * target_unit.factor,
                    target_unit.canonical_unit,
                    profile.dimension,
                    target_unit.provenance,
                    target_unit.approximate,
                    0.72,
                ),
                Reading::quantity(
                    term.high * target_unit.factor,
                    target_unit.canonical_unit,
                    profile.dimension,
                    target_unit.provenance,
                    target_unit.approximate,
                    0.72,
                ),
                0.72,
            );
            mark_approximate(&mut reading);
            return Some(ParsedReading {
                reading,
                approximations: vec![approximation_with_span(
                    text,
                    "Custom fuzzy vocabulary normalized to a configured range.",
                    span(text),
                )],
            });
        }
    }
    None
}

pub(crate) fn parse_fuzzy_temperature(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    // Opt-in: a bare "hot" is a temperature only where the caller said the
    // field holds one, so this stays off until Temperature is declared.
    if !ctx.expected_dimensions.contains(Dimension::Temperature) {
        return None;
    }

    let normalized = text.trim().to_ascii_lowercase();
    let (label, low, high) = if text.contains("暑い") {
        ("暑い", 27.0, 35.0)
    } else if text.contains("暖か") {
        ("暖か", 20.0, 27.0)
    } else if text.contains("寒い") {
        ("寒い", 0.0, 10.0)
    } else if normalized.contains("hot") {
        ("hot", 27.0, 35.0)
    } else if normalized.contains("warm") {
        ("warm", 20.0, 27.0)
    } else if normalized.contains("cold") {
        ("cold", 0.0, 10.0)
    } else {
        return None;
    };

    let mut reading = Reading::range(
        temperature_celsius(low, 0.68),
        temperature_celsius(high, 0.68),
        0.68,
    );
    mark_approximate(&mut reading);
    Some(ParsedReading {
        reading,
        approximations: vec![approximation_with_span(
            label,
            "Fuzzy temperature phrase normalized to a broad Celsius range.",
            span_in(text, label),
        )],
    })
}

pub(crate) fn parse_plus_minus_range(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (left, right) = text
        .split_once('±')
        .or_else(|| split_once_ascii_case(text, "+/-"))?;
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        return None;
    }

    let left_suffix = unit_suffix(left, ctx);
    let right_suffix = unit_suffix(right, ctx);
    let left_owned;
    let right_owned;
    let center_text = if left_suffix.is_none() {
        if let Some(suffix) = right_suffix {
            left_owned = format!("{left}{suffix}");
            left_owned.as_str()
        } else {
            left
        }
    } else {
        left
    };
    let delta_text = if right_suffix.is_none() {
        if let Some(suffix) = left_suffix {
            right_owned = format!("{right}{suffix}");
            right_owned.as_str()
        } else {
            right
        }
    } else {
        right
    };

    let center = parse_endpoint(center_text, ctx)?;
    let delta = parse_endpoint(delta_text, ctx)?;
    let (center_value, delta_value) = (center.value?, delta.value?);
    if center.kind != Kind::Quantity
        || delta.kind != Kind::Quantity
        || center.dimension != delta.dimension
        || center.unit != delta.unit
    {
        return None;
    }

    let unit = center.unit.as_deref()?;
    let dimension = center.dimension?;
    let provenance = center.provenance.unwrap_or(Provenance::TradeCustom);
    let approximate = center.approximate.unwrap_or(false) || delta.approximate.unwrap_or(false);
    Some(Reading::range(
        Reading::quantity(
            center_value - delta_value,
            unit,
            dimension,
            provenance,
            approximate,
            0.93,
        ),
        Reading::quantity(
            center_value + delta_value,
            unit,
            dimension,
            provenance,
            approximate,
            0.93,
        ),
        0.93,
    ))
}

pub(crate) fn parse_upper_bound_range(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let trimmed = text.trim();
    let rest = ["less than ", "under ", "below ", "up to ", "at most "]
        .into_iter()
        .find_map(|prefix| strip_prefix_ascii_case(trimmed, prefix))
        .or_else(|| trimmed.strip_prefix("最大"))
        .or_else(|| trimmed.strip_prefix("上限"))
        .or_else(|| trimmed.strip_prefix('≤'))
        .or_else(|| trimmed.strip_prefix('<'))
        .or_else(|| {
            ["以下", "未満", "まで"]
                .into_iter()
                .find_map(|suffix| trimmed.strip_suffix(suffix))
        })?
        .trim();
    if rest.is_empty() {
        return None;
    }

    let to = parse_endpoint(rest, ctx)?;
    if to.kind != Kind::Quantity || to.value? < 0.0 {
        return None;
    }
    let from = zero_like_quantity(&to)?;
    Some(Reading::range(from, to, 0.86))
}

pub(crate) fn zero_like_quantity(reading: &Reading) -> Option<Reading> {
    Some(Reading::quantity(
        0.0,
        reading.unit.as_deref()?,
        reading.dimension?,
        reading.provenance.unwrap_or(Provenance::TradeCustom),
        reading.approximate.unwrap_or(false),
        0.86,
    ))
}

pub(crate) fn mark_approximate(reading: &mut Reading) {
    reading.approximate = Some(true);
    if let Some(confidence) = reading.confidence.as_mut() {
        *confidence *= 0.9;
    }
    if let Some(range) = reading.range.as_mut() {
        mark_approximate(&mut range.from);
        mark_approximate(&mut range.to);
    }
}

pub(crate) fn strip_prefix_ascii_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let candidate = text.get(..prefix.len())?;
    candidate
        .eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

/// Surfaces a single separator that could be grouping or a decimal point.
///
/// Only under [`NumberFormat::Auto`]: an explicit `DotDecimal` or
/// `CommaDecimal` is the caller resolving the ambiguity, and there is nothing
/// left to report. The dot and the comma are treated the same way, because
/// under `Auto` they are equally undecidable — `1.234` is a thousands-grouped
/// `1234` exactly as much as `1,234` is, and dot grouping is accepted
/// elsewhere (`1.234.567`).
pub(crate) fn parse_ambiguous_number(text: &str, ctx: &ParseCtx) -> Option<AmbiguousParse> {
    if ctx.number_format != NumberFormat::Auto {
        return None;
    }
    if text.contains(',') {
        return ambiguous_separator_number(text, ',');
    }
    if text.contains('.') {
        return ambiguous_separator_number(text, '.');
    }
    None
}

pub(crate) fn ambiguous_separator_number(text: &str, separator: char) -> Option<AmbiguousParse> {
    if text.matches(separator).count() != 1 {
        return None;
    }
    let grouped_is_valid = if separator == ',' {
        valid_grouped_number(text)
    } else {
        valid_dot_grouped_number(text)
    };
    if !grouped_is_valid {
        return None;
    }

    let best = parse_number(text)?;
    let grouped = text.replace(separator, "").parse::<f64>().ok()?;
    let decimal = text
        .replace(separator, ".")
        .parse::<f64>()
        .ok()
        .filter(|value| *value != grouped)?;
    // `parse_number` already picked a reading; the other one becomes the
    // alternative so neither is silently discarded.
    let alternative = if best == grouped { decimal } else { grouped };
    let reason = if separator == ',' {
        "Comma can be read as a thousands separator or a decimal separator."
    } else {
        "Dot can be read as a thousands separator or a decimal separator."
    };
    Some(AmbiguousParse {
        best: Some(Reading::number(best, 0.64)),
        alternatives: vec![Reading::number(alternative, 0.56)],
        ambiguity: ambiguity(text, reason, Some(2), IssueCode::AmbiguousNumber),
    })
}

/// Locates the one grouping-ambiguous number inside a longer reading.
///
/// [`parse_ambiguous_number`] answers the question for text that is *nothing
/// but* a number. A quantity carries the same undecidable number in its numeric
/// part — `1.234 kg` is `1.234 kg` exactly as much as it is `1234 kg` — so the
/// numeric token is located here and asked the same question, and the same
/// `Auto`-only rule applies: a declared [`NumberFormat`] settles it.
///
/// Exactly one candidate is required. Two ambiguous numbers in one input means
/// a range or a compound, and those are reported per endpoint rather than
/// blended into a single finding that names neither.
pub(crate) fn ambiguous_number_token<'a>(
    text: &'a str,
    ctx: &ParseCtx,
) -> Option<(usize, &'a str, AmbiguousParse)> {
    let mut found: Option<(usize, &str, AmbiguousParse)> = None;
    for (start, token) in numeric_tokens(text) {
        let Some(ambiguous) = parse_ambiguous_number(token, ctx) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some((start, token, ambiguous));
    }
    found
}

/// Splits out the maximal runs of digits and separators, with their offsets.
fn numeric_tokens(text: &str) -> Vec<(usize, &str)> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (idx, ch) in text.char_indices() {
        let numeric = ch.is_ascii_digit() || matches!(ch, '.' | ',');
        match (numeric, start) {
            (true, None) => start = Some(idx),
            (false, Some(begin)) => {
                tokens.push((begin, &text[begin..idx]));
                start = None;
            }
            _ => {}
        }
    }
    if let Some(begin) = start {
        tokens.push((begin, &text[begin..]));
    }
    tokens
}

pub(crate) fn parse_range(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (left, right) = split_range_text(text)?;
    let right_suffix = unit_suffix(right, ctx);
    let left_with_unit;
    let left_text = if right_suffix.is_some() && unit_suffix(left, ctx).is_none() {
        left_with_unit = format!("{}{}", left.trim(), right_suffix?);
        left_with_unit.as_str()
    } else {
        left.trim()
    };

    let from = parse_endpoint(left_text, ctx)?;
    let to = parse_endpoint(right.trim(), ctx)?;
    // Endpoints must agree on kind, dimension, and canonical unit: a range whose
    // ends measure different things is not a range, and inventing a conversion
    // between dimensions is not this parser's job. Matches the check
    // `parse_plus_minus_range` already applies.
    if from.kind != to.kind || from.dimension != to.dimension || from.unit != to.unit {
        return None;
    }
    Some(Reading::range(from, to, 0.94))
}

/// Reports the reading question a range endpoint carries in its own right.
///
/// [`parse_range`] reduces each endpoint to a single [`Reading`] through
/// [`parse_endpoint`], which takes the first reading of a three-part slash date
/// and drops the competing one. Standing alone, `5/6/2026` reports an
/// `AmbiguousDate`; as a range endpoint the same text used to commit silently
/// to one order. This restores the report.
///
/// The endpoints themselves stay exactly as parsed, and no alternative range is
/// produced. Re-reading both ends independently would multiply into up to four
/// candidate ranges, most of them combinations no reader intends (nobody writes
/// `5/6/2026 to 7/8/2026` meaning May 6th to August 7th), and the crate's
/// promise is that nothing is dropped *silently* — not that every combination
/// is enumerated. The finding names the endpoint that is in question and
/// carries its span, so a caller can ask about exactly that fragment.
///
/// A grouping-ambiguous number is reported the same way and for the same
/// reason: `1.234-2.345 kg` settled two undecidable numbers at a factor of a
/// thousand each, and said nothing. Each endpoint that carries one gets its own
/// finding, spanning the number rather than the whole endpoint.
pub(crate) fn range_endpoint_ambiguities(text: &str, ctx: &ParseCtx) -> Vec<Ambiguity> {
    let Some((left, right)) = split_range_text(text) else {
        return Vec::new();
    };
    let mut ambiguities = Vec::new();
    // Each endpoint is located in the tail past the one before it. Both ends can
    // be written identically — `1.234-1.234` — and a search from the start would
    // put the right endpoint's finding on the left endpoint's text, reporting
    // the same fragment twice and never naming the one that is still in
    // question.
    let mut searched_to = 0;
    for endpoint in [left, right].into_iter().map(str::trim) {
        let endpoint_span = span_in_from(text, endpoint, searched_to);
        searched_to = endpoint_span.end;
        if let Some(ambiguous) = parse_ambiguous_numeric_slash_date(endpoint, ctx) {
            let mut ambiguity = ambiguous.ambiguity;
            ambiguity.span = endpoint_span;
            ambiguities.push(ambiguity);
            continue;
        }
        let Some((offset, token, ambiguous)) = ambiguous_number_token(endpoint, ctx) else {
            continue;
        };
        let start = endpoint_span.start + offset;
        let mut ambiguity = ambiguous.ambiguity;
        ambiguity.span = span_slice(text, start, start + token.len());
        ambiguities.push(ambiguity);
    }
    ambiguities
}

pub(crate) fn split_range_text(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    if let Some(inner) = trimmed.strip_prefix("between ") {
        return inner.split_once(" and ");
    }
    if let Some(inner) = trimmed.strip_prefix("from ") {
        return inner.split_once(" to ");
    }
    for separator in ["〜", "～", " to ", ".."] {
        if let Some((left, right)) = trimmed.split_once(separator) {
            return non_empty_pair(left, right);
        }
    }
    if let Some((left, right)) = split_clock_hyphen_range(trimmed) {
        return Some((left, right));
    }
    split_ascii_hyphen_range(trimmed)
}

pub(crate) fn split_clock_hyphen_range(text: &str) -> Option<(&str, &str)> {
    let (left, right) = text.split_once('-')?;
    if parse_clock_seconds(left).is_some() && parse_clock_seconds(right).is_some() {
        non_empty_pair(left, right)
    } else {
        None
    }
}

pub(crate) fn split_ascii_hyphen_range(text: &str) -> Option<(&str, &str)> {
    let mut previous = None;
    for (idx, ch) in text.char_indices() {
        if ch != '-' {
            previous = Some(ch);
            continue;
        }
        let next = text[idx + 1..].chars().next();
        if previous?.is_ascii_digit() && next?.is_ascii_digit() {
            return non_empty_pair(&text[..idx], &text[idx + 1..]);
        }
    }
    None
}

pub(crate) fn non_empty_pair<'a>(left: &'a str, right: &'a str) -> Option<(&'a str, &'a str)> {
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        None
    } else {
        Some((left, right))
    }
}

pub(crate) fn unit_suffix<'a>(text: &str, ctx: &'a ParseCtx) -> Option<&'a str> {
    let trimmed = text.trim();
    let builtin_suffixes = [
        "㎡", "m^2", "m2", "平米", "帖", "畳", "坪", "cm", "mm", "m", "kg", "g", "minutes",
        "minute", "mins", "min", "hours", "hour", "hrs", "hr", "days", "day", "日",
    ];
    let all_builtin_suffixes = || {
        builtin_suffixes.into_iter().chain(
            UNIT_DEFS
                .iter()
                .flat_map(|unit| unit.aliases.iter().copied()),
        )
    };
    let mut best = all_builtin_suffixes()
        .filter(|suffix| trimmed.ends_with(suffix))
        .max_by_key(|suffix| suffix.len())
        .or_else(|| {
            all_builtin_suffixes()
                .filter(|suffix| ends_with_ascii_case(trimmed, suffix))
                .max_by_key(|suffix| suffix.len())
        });

    for unit in &ctx.custom_units {
        for suffix in
            core::iter::once(unit.id.as_str()).chain(unit.aliases.iter().map(String::as_str))
        {
            if (trimmed.ends_with(suffix)
                || best.is_none() && ends_with_ascii_case(trimmed, suffix))
                && best.is_none_or(|current| suffix.len() > current.len())
            {
                best = Some(suffix);
            }
        }
    }

    best
}

pub(crate) fn ends_with_ascii_case(text: &str, suffix: &str) -> bool {
    if text.ends_with(suffix) {
        return true;
    }
    if text.len() < suffix.len() || !suffix.is_ascii() {
        return false;
    }
    text.get(text.len() - suffix.len()..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

pub(crate) fn parse_endpoint(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let normalized = normalize_input_cow(text);
    let text = normalized.trim();
    let features = InputFeatures::new(text);

    if features.maybe_date
        && let Some(reading) = parse_relative_date(text, ctx)
    {
        return Some(reading);
    }
    if features.maybe_japanese_length
        && let Some(reading) = parse_japanese_length(text)
    {
        return Some(reading);
    }
    if features.maybe_tatami
        && let Some(reading) = parse_tatami_area(text)
    {
        return Some(reading);
    }
    if features.maybe_tsubo
        && let Some(reading) = parse_tsubo_area(text)
    {
        return Some(reading);
    }
    if features.maybe_area
        && let Some(reading) = parse_square_meter(text)
    {
        return Some(reading);
    }
    if features.maybe_temperature
        && let Some(reading) = parse_temperature(text)
    {
        return Some(reading);
    }
    if features.maybe_quantity
        && let Some(reading) = parse_registered_quantity(text, ctx)
    {
        return Some(reading);
    }
    if features.maybe_metric_length
        && let Some(reading) = parse_metric_length(text)
    {
        return Some(reading);
    }
    if features.maybe_mass
        && let Some(reading) = parse_mass(text)
    {
        return Some(reading);
    }
    if features.maybe_clock
        && let Some(reading) = parse_clock_time(text)
    {
        return Some(reading);
    }
    if features.maybe_duration
        && let Some(reading) = parse_duration(text)
    {
        return Some(reading);
    }
    if features.maybe_feet_inches
        && let Some(reading) = parse_feet_inches(text)
    {
        return Some(reading);
    }
    if features.maybe_currency
        && let Some((best, _, _)) = parse_currency(text, ctx)
    {
        return Some(best);
    }
    if features.maybe_number {
        return parse_plain_number(text);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn parses_japanese_area_range() {
        let parsed = parse(
            "100-120㎡",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("m2"));
        assert_eq!(range.to.unit.as_deref(), Some("m2"));
        assert_close(range.from.value.unwrap(), 100.0);
        assert_close(range.to.value.unwrap(), 120.0);
    }

    #[test]
    fn parses_japanese_duration_range() {
        let parsed = parse(
            "2〜3日",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.dimension, Some(Dimension::Time));
        assert_eq!(range.to.dimension, Some(Dimension::Time));
        assert_close(range.from.value.unwrap(), 172_800.0);
        assert_close(range.to.value.unwrap(), 259_200.0);
    }

    /// A slash-date range must not swallow the day-first/month-first question.
    ///
    /// `parse("5/6/2026", ..)` reports an `AmbiguousDate`; as a range endpoint
    /// the same text used to commit to one order with no finding at all, and
    /// under `en-GB` it silently committed to the other order.
    #[cfg(feature = "dates-jiff")]
    #[test]
    fn slash_date_range_endpoints_keep_their_ambiguity() {
        let ctx = ParseCtx {
            reference_date: Date::new(2026, 7, 19),
            ..ParseCtx::default()
        };
        let parsed = parse("5/6/2026 to 7/8/2026", Some(ctx.clone()));
        let best = parsed.best.as_ref().expect("a range");
        let range = best.range.as_ref().expect("range endpoints");
        assert_eq!(range.from.date.as_deref(), Some("2026-05-06"));
        assert_eq!(range.to.date.as_deref(), Some("2026-07-08"));

        let ambiguities: Vec<_> = parsed
            .findings
            .ambiguities
            .iter()
            .filter(|issue| issue.code == IssueCode::AmbiguousDate)
            .collect();
        assert_eq!(
            ambiguities.len(),
            2,
            "both endpoints have two readings: {:?}",
            parsed.findings
        );
        assert_eq!(ambiguities[0].ref_text, "5/6/2026");
        assert_eq!(ambiguities[0].span.start, 0);
        assert_eq!(ambiguities[1].ref_text, "7/8/2026");
        assert_eq!(ambiguities[1].span.start, 12);

        // `en-GB` reads the other order, and must report the choice just the same.
        let gb = parse(
            "5/6/2026 to 7/8/2026",
            Some(ParseCtx {
                locale: Some(Locale::EnGb),
                ..ctx.clone()
            }),
        );
        let range = gb
            .best
            .as_ref()
            .and_then(|best| best.range.as_ref())
            .expect("a range");
        assert_eq!(range.from.date.as_deref(), Some("2026-06-05"));
        assert_eq!(range.to.date.as_deref(), Some("2026-08-07"));
        assert_eq!(
            gb.findings
                .ambiguities
                .iter()
                .filter(|issue| issue.code == IssueCode::AmbiguousDate)
                .count(),
            2
        );

        // An endpoint only one order can name is not ambiguous and reports nothing.
        let unambiguous = parse("5/25/2026 to 6/26/2026", Some(ctx));
        assert!(
            unambiguous.findings.ambiguities.is_empty(),
            "{:?}",
            unambiguous.findings
        );
    }

    #[test]
    fn parses_between_mass_range() {
        let parsed = parse("between 5 and 10 kg", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("kg"));
        assert_eq!(range.to.unit.as_deref(), Some("kg"));
        assert_close(range.from.value.unwrap(), 5.0);
        assert_close(range.to.value.unwrap(), 10.0);
    }

    #[test]
    fn parses_clock_time_ranges() {
        let parsed = parse("3pm-4pm", None);
        let best = parsed.best.expect("time slot");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.dimension, Some(Dimension::Time));
        assert_eq!(range.to.dimension, Some(Dimension::Time));
        assert_close(range.from.value.unwrap(), 15.0 * 3600.0);
        assert_close(range.to.value.unwrap(), 16.0 * 3600.0);
    }

    #[test]
    fn parses_tolerance_and_bound_ranges() {
        let tolerance = parse("10 ± 0.5 mm", None).best.expect("tolerance");
        assert_eq!(tolerance.kind, Kind::Range);
        let range = tolerance.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("m"));
        assert_close(range.from.value.unwrap(), 0.0095);
        assert_close(range.to.value.unwrap(), 0.0105);

        let upper = parse("under 10 minutes", None).best.expect("upper bound");
        assert_eq!(upper.kind, Kind::Range);
        let range = upper.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("s"));
        assert_close(range.from.value.unwrap(), 0.0);
        assert_close(range.to.value.unwrap(), 600.0);

        let japanese_upper = parse("10mm以下", None).best.expect("Japanese upper bound");
        let range = japanese_upper.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("m"));
        assert_close(range.from.value.unwrap(), 0.0);
        assert_close(range.to.value.unwrap(), 0.01);
    }

    #[test]
    fn parses_approximate_and_fuzzy_readings() {
        let approx = parse("about 20C", None);
        let best = approx.best.expect("approximate temperature");
        assert_eq!(best.dimension, Some(Dimension::Temperature));
        assert_eq!(best.approximate, Some(true));
        assert_eq!(approx.findings.approximations[0].ref_text, "about");

        let japanese_approx = parse("約20kg", None);
        let best = japanese_approx.best.expect("Japanese approximate mass");
        assert_eq!(best.dimension, Some(Dimension::Mass));
        assert_eq!(best.approximate, Some(true));
        assert_eq!(japanese_approx.findings.approximations[0].ref_text, "約");

        let strict = parse(
            "about 20C",
            Some(ParseCtx {
                strictness: Strictness::Strict,
                ..ParseCtx::default()
            }),
        );
        assert!(strict.best.is_none());
        assert_eq!(strict.findings.skipped[0].code, IssueCode::Approximation);

        let few = parse("a few minutes", None);
        let range = few.best.expect("few range").range.expect("range");
        assert_close(range.from.value.unwrap(), 120.0);
        assert_close(range.to.value.unwrap(), 240.0);
        assert_eq!(few.findings.approximations[0].ref_text, "a few");

        let hot = parse(
            "it's hot",
            Some(ParseCtx {
                expected_dimensions: DimensionSet::from(Dimension::Temperature),
                ..ParseCtx::default()
            }),
        );
        let range = hot.best.expect("hot range").range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("C"));
        assert_close(range.from.value.unwrap(), 27.0);
        assert_close(range.to.value.unwrap(), 35.0);

        let japanese_hot = parse(
            "今日は暑い",
            Some(ParseCtx {
                expected_dimensions: DimensionSet::from(Dimension::Temperature),
                ..ParseCtx::default()
            }),
        );
        let range = japanese_hot
            .best
            .expect("Japanese hot range")
            .range
            .expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("C"));
        assert_close(range.from.value.unwrap(), 27.0);
        assert_close(range.to.value.unwrap(), 35.0);
        assert_eq!(japanese_hot.findings.approximations[0].ref_text, "暑い");
    }

    #[test]
    fn refuses_ranges_whose_endpoints_have_different_dimensions() {
        for input in [
            "5kg to 10m",
            "5kg〜10m",
            "between 5kg and 10m",
            "3m to 5kg",
            "from 10kg to 2m",
        ] {
            let parsed = parse(input, None);
            // No range may be built across dimensions, and no conversion
            // between them may be invented.
            assert!(
                parsed
                    .best
                    .as_ref()
                    .is_none_or(|best| best.kind != Kind::Range),
                "{input}: {:?}",
                parsed.best
            );
            // The loss is reported rather than dropped.
            assert!(
                !parsed.findings.skipped.is_empty()
                    || !parsed.findings.ambiguities.is_empty()
                    || parsed.best.is_some(),
                "{input}"
            );
            if parsed.best.is_none() {
                assert_eq!(
                    parsed.findings.skipped[0].code,
                    IssueCode::NoValue,
                    "{input}"
                );
            }
        }
    }

    #[test]
    fn refuses_ranges_whose_endpoints_have_different_units() {
        let ctx = ParseCtx::default();
        // Same dimension, different canonical unit is also refused: two
        // currencies are not endpoints of one range.
        assert!(parse_range("10 USD to 20 JPY", &ctx).is_none());
        assert!(parse("10 USD to 20 JPY", None).best.is_none());
        // Same dimension and same canonical unit still builds a range.
        let range = parse_range("5cm to 10m", &ctx).expect("length range");
        assert_eq!(range.kind, Kind::Range);
        let range = range.range.expect("endpoints");
        assert_eq!(range.from.unit.as_deref(), Some("m"));
        assert_eq!(range.to.unit.as_deref(), Some("m"));
        assert_close(range.from.value.unwrap(), 0.05);
        assert_close(range.to.value.unwrap(), 10.0);
    }
}
