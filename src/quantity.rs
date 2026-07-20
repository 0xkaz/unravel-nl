use crate::*;

pub(crate) fn parse_japanese_length(text: &str) -> Option<Reading> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    for (suffix, factor) in [("間半", KEN_M), ("尺半", SHAKU_M)] {
        if let Some(number_text) = compact.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                (value + 0.5) * factor,
                "m",
                Dimension::Length,
                Provenance::JapaneseStatute,
                true,
                0.94,
            ));
        }
    }

    let mut number = String::new();
    let mut meters = 0.0;
    let mut saw_unit = false;

    for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
        if ch.is_ascii_digit() || ch == '.' || ch == ',' {
            number.push(ch);
            continue;
        }

        let value = parse_number(&number)?;
        number.clear();
        match ch {
            '尺' => {
                meters += value * SHAKU_M;
                saw_unit = true;
            }
            '寸' => {
                meters += value * SUN_M;
                saw_unit = true;
            }
            '間' => {
                meters += value * KEN_M;
                saw_unit = true;
            }
            _ => return None,
        }
    }

    if saw_unit && number.is_empty() {
        Some(Reading::quantity(
            meters,
            "m",
            Dimension::Length,
            Provenance::JapaneseStatute,
            true,
            0.98,
        ))
    } else {
        None
    }
}

pub(crate) fn parse_tatami_area(text: &str) -> Option<Reading> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    for suffix in ["帖半", "畳半"] {
        if let Some(number_text) = compact.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                (value + 0.5) * TATAMI_M2,
                "m2",
                Dimension::Area,
                Provenance::TradeCustom,
                true,
                0.92,
            ));
        }
    }

    let suffix = if text.ends_with('帖') {
        "帖"
    } else if text.ends_with('畳') {
        "畳"
    } else {
        return None;
    };
    let number_text = text.trim_end_matches(suffix);
    let value = parse_number(number_text.trim())?;
    Some(Reading::quantity(
        value * TATAMI_M2,
        "m2",
        Dimension::Area,
        Provenance::TradeCustom,
        true,
        0.94,
    ))
}

pub(crate) fn parse_tsubo_area(text: &str) -> Option<Reading> {
    let number_text = text.strip_suffix("坪")?;
    let value = parse_number(number_text.trim())?;
    Some(Reading::quantity(
        value * TSUBO_M2,
        "m2",
        Dimension::Area,
        Provenance::JapaneseStatute,
        true,
        0.94,
    ))
}

pub(crate) fn parse_square_meter(text: &str) -> Option<Reading> {
    let stripped = text
        .strip_prefix("延床")
        .or_else(|| text.strip_prefix("延べ床"))
        .unwrap_or(text)
        .trim();

    for suffix in ["㎡", "m2", "m^2", "平米"] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value,
                "m2",
                Dimension::Area,
                Provenance::SiMultiple,
                false,
                0.99,
            ));
        }
    }
    None
}

pub(crate) fn parse_temperature(text: &str) -> Option<Reading> {
    let stripped = text.trim();
    if let Some(value) = stripped
        .strip_prefix("摂氏")
        .and_then(|rest| rest.strip_suffix('度'))
        .and_then(parse_number)
    {
        return Some(temperature_celsius(value, 0.95));
    }
    if let Some(value) = stripped
        .strip_prefix("華氏")
        .and_then(|rest| rest.strip_suffix('度'))
        .and_then(parse_number)
    {
        return Some(temperature_celsius(fahrenheit_to_celsius(value), 0.95));
    }

    for suffix in [
        "degrees celsius",
        "degree celsius",
        "celsius",
        "°c",
        "℃",
        "c",
    ] {
        if let Some(value) = strip_suffix_ascii_case(stripped, suffix).and_then(parse_number) {
            return Some(temperature_celsius(value, 0.95));
        }
    }
    for suffix in [
        "degrees fahrenheit",
        "degree fahrenheit",
        "fahrenheit",
        "°f",
        "℉",
        "f",
    ] {
        if let Some(value) = strip_suffix_ascii_case(stripped, suffix).and_then(parse_number) {
            return Some(temperature_celsius(fahrenheit_to_celsius(value), 0.93));
        }
    }
    for suffix in ["kelvin", "kelvins", "k"] {
        if let Some(value) = strip_suffix_ascii_case(stripped, suffix).and_then(parse_number) {
            return Some(temperature_celsius(value - 273.15, 0.93));
        }
    }

    None
}

pub(crate) fn temperature_celsius(value: f64, confidence: f64) -> Reading {
    Reading::quantity(
        value,
        "C",
        Dimension::Temperature,
        Provenance::InternationalExact,
        false,
        confidence,
    )
}

pub(crate) fn fahrenheit_to_celsius(value: f64) -> f64 {
    (value - 32.0) * 5.0 / 9.0
}

pub(crate) fn strip_suffix_ascii_case<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let prefix_len = text.len().checked_sub(suffix.len())?;
    let prefix = text.get(..prefix_len)?;
    let actual_suffix = text.get(prefix_len..)?;
    actual_suffix
        .eq_ignore_ascii_case(suffix)
        .then_some(prefix.trim())
}

pub(crate) fn parse_registered_quantity(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (number_text, unit_text) = split_number_unit(text)?;
    let value = parse_number_ctx(number_text, ctx)?;
    if let Some(unit) = unit_by_alias(unit_text) {
        return Some(Reading::quantity(
            value * unit.factor,
            unit.canonical_unit,
            unit.dimension,
            unit.provenance,
            unit.approximate,
            0.98,
        ));
    }
    let unit = custom_unit_by_alias(unit_text, ctx)?;
    let mut reading = Reading::quantity(
        value * unit.factor,
        &unit.canonical_unit,
        unit.dimension,
        Provenance::TradeCustom,
        unit.approximate,
        0.93,
    );
    reading.custom_kind = unit.kind_id.clone();
    Some(reading)
}

pub(crate) fn parse_compound_registered_quantity_ctx(
    text: &str,
    ctx: &ParseCtx,
) -> Option<Reading> {
    parse_compound_registered_quantity_with_format(text, ctx.number_format)
}

pub(crate) fn parse_compound_registered_quantity_with_format(
    text: &str,
    number_format: NumberFormat,
) -> Option<Reading> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 4 || !parts.len().is_multiple_of(2) {
        return None;
    }

    let mut total = 0.0;
    let mut dimension = None;
    let mut canonical_unit = None;
    let mut provenance = Provenance::SiMultiple;
    let mut approximate = false;

    for pair in parts.chunks_exact(2) {
        let value = parse_number_with_format(pair[0], number_format)?;
        let unit = unit_by_alias(pair[1])?;
        if let Some(current_dimension) = dimension {
            if current_dimension != unit.dimension || canonical_unit != Some(unit.canonical_unit) {
                return None;
            }
        } else {
            dimension = Some(unit.dimension);
            canonical_unit = Some(unit.canonical_unit);
            provenance = unit.provenance;
        }
        total += value * unit.factor;
        approximate |= unit.approximate;
    }

    Some(Reading::quantity(
        total,
        canonical_unit?,
        dimension?,
        provenance,
        approximate,
        0.94,
    ))
}

pub(crate) fn parse_typo_corrected_quantity_ctx(
    text: &str,
    ctx: &ParseCtx,
) -> Option<(Reading, Suggestion, String)> {
    let (number_text, unit_text) = split_number_unit(text)?;
    let value = parse_number_ctx(number_text, ctx)?;
    let suggestion = suggest_unit(unit_text)?;
    let corrected = unit_by_alias(&suggestion.to)?;
    let reading = Reading::quantity(
        value * corrected.factor,
        corrected.canonical_unit,
        corrected.dimension,
        corrected.provenance,
        corrected.approximate,
        0.82,
    );
    Some((reading, suggestion, unit_text.to_owned()))
}

pub(crate) fn split_number_unit(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    let mut seen_digit = false;
    for (idx, ch) in trimmed.char_indices() {
        if is_number_prefix_char(ch) {
            seen_digit = true;
            continue;
        }
        if matches!(ch, '+' | '-') && idx == 0 {
            continue;
        }
        if seen_digit && matches!(ch, '.' | ',' | '_' | '/' | '½' | '¼' | '¾') {
            continue;
        }
        if seen_digit {
            let (number_text, unit_text) = trimmed.split_at(idx);
            let unit_text = unit_text.trim();
            if !unit_text.is_empty() && parse_number(number_text.trim()).is_some() {
                return Some((number_text.trim(), unit_text));
            }
        }
        return None;
    }
    None
}

pub(crate) fn is_number_prefix_char(ch: char) -> bool {
    ch.is_ascii_digit() || is_cjk_number_char(ch)
}

pub(crate) fn parse_metric_length(text: &str) -> Option<Reading> {
    let stripped = text.trim().to_ascii_lowercase();
    if let Some((meters, centimeters)) = stripped.split_once('m')
        && !meters.is_empty()
        && !centimeters.is_empty()
        && !centimeters.contains(char::is_whitespace)
    {
        let meters = parse_number(meters.trim())?;
        let centimeters = parse_number(centimeters.trim())?;
        return Some(Reading::quantity(
            meters + centimeters * CM_M,
            "m",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.97,
        ));
    }

    for (suffix, factor) in [
        ("cm", CM_M),
        ("mm", 0.001),
        ("in", INCH_M),
        ("inch", INCH_M),
        ("inches", INCH_M),
        ("ft", FOOT_M),
        ("feet", FOOT_M),
        ("m", 1.0),
    ] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value * factor,
                "m",
                Dimension::Length,
                Provenance::SiMultiple,
                false,
                0.99,
            ));
        }
    }
    None
}

pub(crate) fn parse_mass(text: &str) -> Option<Reading> {
    let stripped = text.trim().to_ascii_lowercase();
    if let Some((pounds_text, ounces_tail)) = stripped.split_once(" lb ") {
        let ounces_text = ounces_tail
            .strip_suffix(" oz")
            .or_else(|| ounces_tail.strip_suffix(" ounce"))
            .or_else(|| ounces_tail.strip_suffix(" ounces"))?;
        let pounds = parse_number(pounds_text.trim())?;
        let ounces = parse_number(ounces_text.trim())?;
        return Some(Reading::quantity(
            pounds * LB_KG + ounces * OZ_KG,
            "kg",
            Dimension::Mass,
            Provenance::InternationalExact,
            false,
            0.96,
        ));
    }

    for (suffix, factor) in [
        ("kg", 1.0),
        ("kilograms", 1.0),
        ("kilogram", 1.0),
        ("公斤", 1.0),
        ("キログラム", 1.0),
        ("キロ", 1.0),
        ("lbs", LB_KG),
        ("lb", LB_KG),
        ("pounds", LB_KG),
        ("pound", LB_KG),
        ("ounces", OZ_KG),
        ("ounce", OZ_KG),
        ("oz", OZ_KG),
        ("g", 0.001),
    ] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value * factor,
                "kg",
                Dimension::Mass,
                Provenance::SiMultiple,
                false,
                0.98,
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn parses_temperature_forms() {
        let celsius = parse("20°C", None).best.expect("celsius");
        assert_eq!(celsius.unit.as_deref(), Some("C"));
        assert_eq!(celsius.dimension, Some(Dimension::Temperature));
        assert_close(celsius.value.unwrap(), 20.0);

        let fahrenheit = parse("68 F", None).best.expect("fahrenheit");
        assert_eq!(fahrenheit.unit.as_deref(), Some("C"));
        assert_close(fahrenheit.value.unwrap(), 20.0);

        let kelvin = parse("293.15 K", None).best.expect("kelvin");
        assert_eq!(kelvin.dimension, Some(Dimension::Temperature));
        assert_close(kelvin.value.unwrap(), 20.0);

        let japanese = parse("摂氏20度", None).best.expect("japanese celsius");
        assert_eq!(japanese.dimension, Some(Dimension::Temperature));
        assert_close(japanese.value.unwrap(), 20.0);

        let japanese_f = parse("華氏68度", None).best.expect("japanese fahrenheit");
        assert_close(japanese_f.value.unwrap(), 20.0);

        assert_eq!(humanize(&celsius, None), "20 °C");
        let round_trip_text = humanize(
            &japanese,
            Some(HumanizeCtx {
                locale: Some(Locale::Ja),
            }),
        );
        assert_eq!(round_trip_text, "摂氏20度");
        assert_close(
            parse(&round_trip_text, None)
                .best
                .expect("temperature round-trip")
                .value
                .unwrap(),
            20.0,
        );
    }

    #[test]
    fn parses_shaku_and_sun() {
        let parsed = parse(
            "5尺3寸",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Quantity);
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_eq!(best.provenance, Some(Provenance::JapaneseStatute));
        assert_eq!(best.approximate, Some(true));
        assert_close(best.value.unwrap(), 53.0 / 33.0);
        assert_eq!(parsed.findings.approximations.len(), 1);
        assert!(parsed.findings.skipped.is_empty());
    }

    #[test]
    fn parses_tatami_area() {
        let parsed = parse(
            "6帖",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m2"));
        assert_eq!(best.dimension, Some(Dimension::Area));
        assert_eq!(best.provenance, Some(Provenance::TradeCustom));
        assert_close(best.value.unwrap(), 9.72);
        assert_eq!(
            humanize(
                &best,
                Some(HumanizeCtx {
                    locale: Some(Locale::Ja)
                })
            ),
            "6帖 (approx.)"
        );
    }

    #[test]
    fn parses_gross_floor_square_meters() {
        let parsed = parse(
            "延床100㎡",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m2"));
        assert_eq!(best.dimension, Some(Dimension::Area));
        assert_eq!(best.provenance, Some(Provenance::SiMultiple));
        assert_eq!(best.approximate, Some(false));
        assert_close(best.value.unwrap(), 100.0);
    }

    #[test]
    fn parses_lingo_readme_metric_length_examples() {
        let cm = parse("180cm", None).best.expect("cm reading");
        assert_eq!(cm.unit.as_deref(), Some("m"));
        assert_close(cm.value.unwrap(), 1.8);

        let compound = parse("1m80", None).best.expect("compound reading");
        assert_eq!(compound.unit.as_deref(), Some("m"));
        assert_close(compound.value.unwrap(), 1.8);
    }

    #[test]
    fn parses_comma_decimal_mass() {
        let parsed = parse("1,5 kg", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("kg"));
        assert_eq!(best.dimension, Some(Dimension::Mass));
        assert_close(best.value.unwrap(), 1.5);
    }

    #[test]
    fn parses_compound_imperial_mass() {
        let parsed = parse("2 lb 3 oz", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("kg"));
        assert_close(best.value.unwrap(), 0.992_233_375);
    }

    #[test]
    fn typo_corrects_units_in_forgiving_mode() {
        let parsed = parse("5 meterz", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_close(best.value.unwrap(), 5.0);
        assert_eq!(parsed.suggestions[0].from, "meterz");
        assert_eq!(parsed.suggestions[0].to, "m");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::TypoCorrected
        );
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "meterz");
        assert_eq!(parsed.findings.ambiguities[0].span.start, 2);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 8);
    }

    #[test]
    fn typo_corrects_japanese_units_in_forgiving_mode() {
        let parsed = parse("10平目", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m2"));
        assert_eq!(best.dimension, Some(Dimension::Area));
        assert_close(best.value.unwrap(), 10.0);
        assert_eq!(parsed.suggestions[0].from, "平目");
        assert_eq!(parsed.suggestions[0].to, "m2");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::TypoCorrected
        );

        let confirm = parse(
            "10平目",
            Some(ParseCtx {
                strictness: Strictness::Confirm,
                ..ParseCtx::default()
            }),
        );
        assert!(confirm.best.is_none());
        assert_eq!(confirm.suggestions[0].to, "m2");
        assert_eq!(confirm.findings.skipped[0].code, IssueCode::TypoCorrected);
    }

    #[test]
    fn confirm_mode_requires_typo_confirmation() {
        let parsed = parse(
            "5 meterz",
            Some(ParseCtx {
                strictness: Strictness::Confirm,
                ..ParseCtx::default()
            }),
        );
        assert!(parsed.best.is_none());
        assert_eq!(parsed.suggestions[0].to, "m");
        assert_eq!(parsed.findings.skipped[0].code, IssueCode::TypoCorrected);
        assert_eq!(parsed.findings.skipped[0].ref_text, "meterz");
        assert_eq!(parsed.findings.skipped[0].span.start, 2);
        assert_eq!(parsed.findings.skipped[0].span.end, 8);
    }
}
