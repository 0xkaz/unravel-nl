use crate::*;

pub(crate) fn parse_conversion_request(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (source, target_unit) = split_once_ascii_case(text.trim(), " to ")?;
    let source = parse_endpoint(source, ctx)?;
    let target_unit = target_unit.trim();

    if let Some(reading) = convert_registered_reading(&source, target_unit) {
        return Some(reading);
    }

    let value = source.value?;
    let source_unit = source.unit.as_deref()?;

    match (source.dimension?, source_unit, target_unit) {
        (Dimension::Length, "m", "cm") => Some(Reading::quantity(
            value / CM_M,
            "cm",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.95,
        )),
        (Dimension::Length, "m", "m") => Some(Reading::quantity(
            value,
            "m",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.95,
        )),
        (Dimension::Length, "m", "in") => Some(Reading::quantity(
            value / INCH_M,
            "in",
            Dimension::Length,
            Provenance::InternationalExact,
            false,
            0.95,
        )),
        (Dimension::Length, "m", "ft") => Some(Reading::quantity(
            value / FOOT_M,
            "ft",
            Dimension::Length,
            Provenance::InternationalExact,
            false,
            0.95,
        )),
        (Dimension::Mass, "kg", "lb" | "lbs") => Some(Reading::quantity(
            value / LB_KG,
            "lb",
            Dimension::Mass,
            Provenance::InternationalExact,
            false,
            0.95,
        )),
        (Dimension::Mass, "kg", "kg") => Some(Reading::quantity(
            value,
            "kg",
            Dimension::Mass,
            Provenance::SiMultiple,
            false,
            0.95,
        )),
        (Dimension::Currency, unit, target) => {
            let target = normalize_currency_code(target);
            if unit == target {
                return Some(currency_reading(value, unit, 0.96));
            }
            let rate = ctx
                .currency_rates
                .iter()
                .find(|rate| rate.from == unit && rate.to == target)?;
            Some(currency_reading(value * rate.factor, &target, 0.91))
        }
        _ => None,
    }
}

pub(crate) fn parse_feet_inches(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let trimmed = text.trim();
    let lowered = trimmed.to_ascii_lowercase();
    let (reading, has_inches) = feet_inches_reading(&lowered)?;
    // `5 ft2` is the registry's square foot, not 5 ft 2 in — see
    // `spaced_registry_unit`, and `closed_registry_unit` for the `5ft2` written
    // closed up. Only the two-part form is guarded: `5 ft` carries no second
    // number and so no competing reading.
    //
    // The lookups read the text as written: a registry alias may be
    // case-sensitive, and asking about the lowercased text answers for a
    // different unit than the one the writer named.
    if has_inches
        && (spaced_registry_unit(trimmed, ctx.unit_registry)
            || closed_registry_unit(trimmed, ctx.unit_registry))
    {
        return None;
    }
    Some(reading)
}

/// Feet and inches, over text that is already trimmed and lowercased, paired
/// with whether an inches part was actually written.
pub(crate) fn feet_inches_reading(lowered: &str) -> Option<(Reading, bool)> {
    let ft_pos = lowered
        .find("ft")
        .or_else(|| lowered.find("feet"))
        .or_else(|| lowered.find('\''))?;
    // Two apostrophes are a declared inch alias, not an empty inches part
    // following a foot mark. Let the registry own that spelling instead of
    // guessing five feet through this compound grammar.
    if lowered[ft_pos..].starts_with("''") {
        return None;
    }
    let feet = parse_number(lowered[..ft_pos].trim())?;
    let rest = lowered[ft_pos..]
        .trim_start_matches("feet")
        .trim_start_matches("ft")
        .trim_start_matches('\'')
        .trim();
    let has_inches = !rest.is_empty();
    let inches = if rest.is_empty() {
        0.0
    } else {
        let cleaned = rest
            .trim_end_matches("inches")
            .trim_end_matches("inch")
            .trim_end_matches("in")
            .trim_end_matches('"')
            .trim();
        // The inches part carries no unit of its own, so it has to be a bare
        // count for the idiom to supply one: `5ft-11` is not five foot eleven
        // less something, it is a shape this grammar does not read.
        if !unsigned_lower_place(cleaned) {
            return None;
        }
        let inches = parse_number(cleaned)?;
        // `1 ft 234 in` is refused for spilling out of the foot above it, and
        // this is the same compound written with an apostrophe.
        if !lower_place_stays_inside(inches * INCH_M, FOOT_M) {
            return None;
        }
        inches
    };

    Some((
        Reading::quantity(
            feet * FOOT_M + inches * INCH_M,
            "m",
            Dimension::Length,
            Provenance::InternationalExact,
            false,
            0.97,
        ),
        has_inches,
    ))
}

pub(crate) fn parse_cups(text: &str, ctx: &ParseCtx) -> Option<(Reading, Vec<Reading>, Ambiguity)> {
    let lowered = text.trim().to_ascii_lowercase();
    let unit_text = if lowered.ends_with("cups") {
        "cups"
    } else if lowered.ends_with("cup") {
        "cup"
    } else {
        return None;
    };
    let number_text = lowered.strip_suffix(unit_text)?.trim();
    let value = parse_number_ctx(number_text, ctx)?;

    let us = Reading::quantity(
        value * US_CUP_L,
        "L",
        Dimension::Volume,
        Provenance::TradeCustom,
        true,
        0.72,
    );
    let uk = Reading::quantity(
        value * UK_CUP_L,
        "L",
        Dimension::Volume,
        Provenance::TradeCustom,
        true,
        0.68,
    );
    let metric = Reading::quantity(
        value * METRIC_CUP_L,
        "L",
        Dimension::Volume,
        Provenance::TradeCustom,
        true,
        0.66,
    );

    let (best, alternatives) = match ctx.locale.as_ref() {
        Some(Locale::EnGb) => (uk, vec![us, metric]),
        Some(Locale::Ja) | Some(Locale::EnUs) => (us, vec![metric, uk]),
        _ => (us, vec![metric, uk]),
    };

    Some((
        best,
        alternatives,
        ambiguity_with_span(
            unit_text,
            "Cup volume depends on locale (US, imperial, or metric cup).",
            Some(3),
            IssueCode::AmbiguousUnit,
            span_token_in(text, unit_text),
        ),
    ))
}

pub(crate) fn parse_currency(
    text: &str,
    ctx: &ParseCtx,
) -> Option<(Reading, Vec<Reading>, Option<Ambiguity>)> {
    let trimmed = text.trim();

    for (prefix, code) in [
        ("US$", "USD"),
        ("USD", "USD"),
        ("usd", "USD"),
        ("EUR", "EUR"),
        ("eur", "EUR"),
        ("GBP", "GBP"),
        ("gbp", "GBP"),
        ("JPY", "JPY"),
        ("jpy", "JPY"),
        ("€", "EUR"),
        ("£", "GBP"),
        ("¥", "JPY"),
        ("￥", "JPY"),
    ] {
        if let Some(number_text) = strip_prefix_currency(trimmed, prefix) {
            let value = parse_number_ctx(number_text.trim(), ctx)?;
            return Some((currency_reading(value, code, 0.95), Vec::new(), None));
        }
    }

    for (suffix, code) in [
        ("USD", "USD"),
        ("usd", "USD"),
        ("EUR", "EUR"),
        ("eur", "EUR"),
        ("GBP", "GBP"),
        ("gbp", "GBP"),
        ("JPY", "JPY"),
        ("jpy", "JPY"),
        ("dollars", "USD"),
        ("dollar", "USD"),
        ("bucks", "USD"),
        ("buck", "USD"),
        ("euros", "EUR"),
        ("euro", "EUR"),
        ("pounds", "GBP"),
        ("pound", "GBP"),
        ("quid", "GBP"),
        ("yen", "JPY"),
        ("円", "JPY"),
    ] {
        if let Some(number_text) = strip_suffix_currency(trimmed, suffix) {
            let value = parse_number_ctx(number_text.trim(), ctx)?;
            return Some((currency_reading(value, code, 0.95), Vec::new(), None));
        }
    }

    for (suffix, code) in [
        ("pence", "GBP"),
        ("penny", "GBP"),
        ("cents usd", "USD"),
        ("cent usd", "USD"),
        ("euro cents", "EUR"),
        ("euro cent", "EUR"),
    ] {
        if let Some(number_text) = strip_suffix_currency(trimmed, suffix) {
            let value = parse_number_ctx(number_text.trim(), ctx)?;
            return Some((
                currency_reading(value / 100.0, code, 0.93),
                Vec::new(),
                None,
            ));
        }
    }

    if let Some(number_text) =
        strip_suffix_currency(trimmed, "cents").or_else(|| strip_suffix_currency(trimmed, "cent"))
    {
        let value = parse_number_ctx(number_text.trim(), ctx)?;
        let best = currency_reading(value / 100.0, "USD", 0.67);
        let alternatives = vec![currency_reading(value / 100.0, "EUR", 0.58)];
        let fragment = if trimmed.ends_with('s') {
            "cents"
        } else {
            "cent"
        };
        let ambiguity = ambiguity_with_span(
            fragment,
            "Cent minor unit needs currency context.",
            Some(2),
            IssueCode::AmbiguousCurrency,
            span_token_in(trimmed, fragment),
        );
        return Some((best, alternatives, Some(ambiguity)));
    }

    if let Some(number_text) = trimmed.strip_prefix('$') {
        let value = parse_number_ctx(number_text.trim(), ctx)?;
        let best = currency_reading(value, "USD", 0.74);
        let alternatives = vec![
            currency_reading(value, "CAD", 0.61),
            currency_reading(value, "AUD", 0.59),
        ];
        let ambiguity = ambiguity_with_span(
            "$",
            "Dollar symbol can refer to multiple currencies without locale or market context.",
            Some(3),
            IssueCode::AmbiguousCurrency,
            span_token_in(trimmed, "$"),
        );
        return Some((best, alternatives, Some(ambiguity)));
    }

    None
}

pub(crate) fn currency_reading(value: f64, unit: &str, confidence: f64) -> Reading {
    Reading::quantity(
        value,
        unit,
        Dimension::Currency,
        Provenance::TradeCustom,
        false,
        confidence,
    )
}

pub(crate) fn strip_prefix_currency<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_ascii() {
        let candidate = text.get(..prefix.len())?;
        candidate
            .eq_ignore_ascii_case(prefix)
            .then(|| &text[prefix.len()..])
    } else {
        text.strip_prefix(prefix)
    }
}

pub(crate) fn strip_suffix_currency<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    if suffix.is_ascii() {
        if text.len() < suffix.len() {
            return None;
        }
        let start = text.len() - suffix.len();
        let candidate = text.get(start..)?;
        candidate
            .eq_ignore_ascii_case(suffix)
            .then(|| &text[..start])
    } else {
        text.strip_suffix(suffix)
    }
}

pub(crate) fn normalize_currency_code(code: &str) -> String {
    code.trim().to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn parses_simple_conversion_request() {
        let parsed = parse("72 in to cm", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("cm"));
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_close(best.value.unwrap(), 182.88);
    }

    #[test]
    fn parses_feet_inches() {
        let parsed = parse(
            "5ft 11",
            Some(ParseCtx {
                locale: Some(Locale::EnUs),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.provenance, Some(Provenance::InternationalExact));
        assert_close(best.value.unwrap(), 1.8034);
    }

    #[test]
    fn surfaces_cup_locale_ambiguity() {
        let parsed = parse(
            "1.5 cups",
            Some(ParseCtx {
                locale: Some(Locale::En),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("L"));
        assert_close(best.value.unwrap(), 1.5 * US_CUP_L);
        assert_eq!(parsed.alternatives.len(), 2);
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(3));
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "cups");
        assert_eq!(parsed.findings.ambiguities[0].span.start, 4);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 8);
    }

    #[test]
    fn parses_currency_codes_and_symbols() {
        let usd = parse("USD 12.34", None).best.expect("usd");
        assert_eq!(usd.unit.as_deref(), Some("USD"));
        assert_eq!(usd.dimension, Some(Dimension::Currency));
        assert_close(usd.value.unwrap(), 12.34);

        let yen = parse("¥1,234", None).best.expect("yen");
        assert_eq!(yen.unit.as_deref(), Some("JPY"));
        assert_close(yen.value.unwrap(), 1234.0);
    }

    #[test]
    fn surfaces_dollar_currency_ambiguity() {
        let parsed = parse("$12", None);
        let best = parsed.best.expect("dollar");
        assert_eq!(best.unit.as_deref(), Some("USD"));
        assert_eq!(best.dimension, Some(Dimension::Currency));
        assert_eq!(parsed.alternatives.len(), 2);
        assert_eq!(parsed.alternatives[0].unit.as_deref(), Some("CAD"));
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousCurrency
        );
        assert_eq!(parsed.findings.ambiguities[0].span.start, 0);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 1);
    }

    #[test]
    fn parses_currency_slang_and_minor_units() {
        let bucks = parse("12 bucks", None).best.expect("bucks");
        assert_eq!(bucks.unit.as_deref(), Some("USD"));
        assert_close(bucks.value.unwrap(), 12.0);
        assert_eq!(humanize(&bucks, None), "USD 12");

        let pence = parse("99 pence", None).best.expect("pence");
        assert_eq!(pence.unit.as_deref(), Some("GBP"));
        assert_close(pence.value.unwrap(), 0.99);

        let cents = parse("50 cents", None);
        let best = cents.best.expect("cents");
        assert_eq!(best.unit.as_deref(), Some("USD"));
        assert_close(best.value.unwrap(), 0.5);
        assert_eq!(cents.alternatives[0].unit.as_deref(), Some("EUR"));
        assert_eq!(
            cents.findings.ambiguities[0].code,
            IssueCode::AmbiguousCurrency
        );
        assert_eq!(cents.findings.ambiguities[0].ref_text, "cents");
        assert_eq!(cents.findings.ambiguities[0].span.start, 3);
        assert_eq!(cents.findings.ambiguities[0].span.end, 8);
    }

    #[test]
    fn converts_currency_with_supplied_rate() {
        let parsed = parse(
            "USD 10 to JPY",
            Some(ParseCtx {
                currency_rates: vec![CurrencyRate::new("USD", "JPY", 150.0)],
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("converted currency");
        assert_eq!(best.unit.as_deref(), Some("JPY"));
        assert_eq!(best.dimension, Some(Dimension::Currency));
        assert_close(best.value.unwrap(), 1500.0);

        let without_rate = parse("USD 10 to JPY", None);
        assert!(without_rate.best.is_none());
        assert_eq!(without_rate.findings.skipped[0].code, IssueCode::NoValue);
    }
}
