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

/// Records the approximation a unit definition declares about itself.
///
/// A [`CustomUnit`] built with `.approximate(true)`, and a registry unit whose
/// definition carries `approximate: true` (`month`, `year`), both hand back a
/// reading with `approximate: Some(true)`. Without this the reading would reach
/// the caller with empty [`Findings`], which the crate's no-silent-loss
/// guarantee forbids: an approximate value must be reported, not merely
/// flagged. Readings that are exact, and readings whose approximation some
/// other grammar already reported (a fuzzy qualifier, a shakkanhō conversion),
/// do not reach here.
pub(crate) fn note_unit_approximation(parsed: &mut Parsed, text: &str, reading: &Reading) {
    if reading.approximate != Some(true) {
        return;
    }
    parsed.findings.approximations.push(approximation_with_span(
        text,
        "Unit definition declares its conversion approximate.",
        span(text),
    ));
}

/// What a run of same-dimension quantities turned out to be.
///
/// A compound states a sum; a run that only looks like one states nothing, and
/// the difference has to reach the caller rather than being collapsed into
/// `None` alongside `3 yd 2 kg`, which is a different failure.
pub(crate) enum CompoundOutcome {
    /// Not a run of `<number> <unit>` pairs at all, or not one dimension.
    NotCompound,
    /// A run of pairs that does not descend into strictly smaller places.
    Malformed(&'static str),
    /// A compound whose sum the input states.
    Reading(Reading),
}

pub(crate) fn parse_compound_registered_quantity_ctx(
    text: &str,
    ctx: &ParseCtx,
) -> CompoundOutcome {
    parse_compound_registered_quantity_with_format(text, ctx.number_format)
}

/// Reads `5 m 3 cm`, `2 lb 3 oz`, `4 stone 6 lb` — and refuses `3 m 5 m`.
///
/// A compound quantity states a sum because each part names a strictly smaller
/// place of the same measurement and stays inside the place above it. Three
/// shapes fail that and were previously added up anyway, reporting a total the
/// text does not write:
///
/// - a repeated unit (`3 m 5 m`, `5 kg 3 kg`) — two separate measurements, not
///   two places of one;
/// - an ascending run (`3 cm 5 m`) — no notation writes the small place first;
/// - a part that overflows its place (`1 m 300 cm`) — 300 cm is not a remainder
///   of a metre.
pub(crate) fn parse_compound_registered_quantity_with_format(
    text: &str,
    number_format: NumberFormat,
) -> CompoundOutcome {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 4 || !parts.len().is_multiple_of(2) {
        return CompoundOutcome::NotCompound;
    }

    let mut total = 0.0;
    let mut dimension = None;
    let mut canonical_unit = None;
    let mut provenance = Provenance::SiMultiple;
    let mut approximate = false;
    let mut previous_factor: Option<f64> = None;

    // The rule below is stated once and asked for by name, because the
    // apostrophe idiom in `currency.rs` writes the same compound with
    // punctuation instead of unit words and has to answer it the same way.
    // While the two carried the rule separately, `1 ft 234 in` was refused and
    // `1'234` — the same sum — was returned as a confident reading.

    for pair in parts.chunks_exact(2) {
        let Some(value) = parse_number_with_format(pair[0], number_format) else {
            return CompoundOutcome::NotCompound;
        };
        let Some(unit) = unit_by_alias(pair[1]) else {
            return CompoundOutcome::NotCompound;
        };
        if let Some(current_dimension) = dimension {
            if current_dimension != unit.dimension || canonical_unit != Some(unit.canonical_unit) {
                return CompoundOutcome::NotCompound;
            }
        } else {
            dimension = Some(unit.dimension);
            canonical_unit = Some(unit.canonical_unit);
            provenance = unit.provenance;
        }
        if let Some(previous) = previous_factor {
            if unit.factor >= previous {
                return CompoundOutcome::Malformed(
                    "a compound has to descend into smaller units; this run repeats or climbs",
                );
            }
            if !lower_place_stays_inside(value * unit.factor, previous) {
                return CompoundOutcome::Malformed(
                    "a compound part has to stay inside the place above it",
                );
            }
        }
        previous_factor = Some(unit.factor);
        total += value * unit.factor;
        approximate |= unit.approximate;
    }

    let (Some(canonical_unit), Some(dimension)) = (canonical_unit, dimension) else {
        return CompoundOutcome::NotCompound;
    };
    CompoundOutcome::Reading(Reading::quantity(
        total,
        canonical_unit,
        dimension,
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

/// True when the text is a number, a space, and a unit token the registry knows.
///
/// This is what separates `5 m3` from `1m80`. The compound-height idiom is
/// written closed up — `1m80`, `5ft 11`, `180cm` — so the space, when there is
/// one, sits *after* the unit and never before it. A space before a token the
/// registry already resolves is therefore not that idiom at all: `m3` is the
/// volume alias and `ft2` the area one, and reading them as metres-and-
/// centimetres or feet-and-inches is not a competing reading anyone wrote, it
/// is the split point guessing.
///
/// Deliberately narrow: two whitespace-separated tokens only, so the compound
/// forms that *do* put a space between their parts (`5 ft 11`, `3 yd 2 ft`,
/// `2 lb 3 oz`) are untouched.
pub(crate) fn spaced_registry_unit(text: &str) -> bool {
    let mut parts = text.split_whitespace();
    let (Some(number_text), Some(unit_text), None) = (parts.next(), parts.next(), parts.next())
    else {
        return false;
    };
    parse_number(number_text).is_some() && unit_by_alias(unit_text).is_some()
}

/// True when the text is a number written closed up against a unit token the
/// registry knows: `5m3`, `5ft2`.
///
/// `1m80` has exactly the same shape, and the lookup is the only thing that
/// tells the two apart: `m3` is the registry's cubic metre and `ft2` its square
/// foot, while `m80` is not a unit at all. The registry entry leads, because a
/// declared unit outranks a guess at where a compound splits — the same rule
/// [`spaced_registry_unit`] applies to `5 m3`.
///
/// Unlike the spaced form, though, the compound reading here is one someone
/// could have meant, so it is not dropped: the entry points report it as an
/// alternative with an [`IssueCode::AmbiguousUnit`] finding — see
/// [`closed_compound_alternative`] and `report_closed_compound_alternative`.
pub(crate) fn closed_registry_unit(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.contains(char::is_whitespace) {
        return false;
    }
    split_number_unit(trimmed).is_some_and(|(_, unit_text)| unit_by_alias(unit_text).is_some())
}

/// The reading a closed-up `<number><alias>` text also has as a compound.
///
/// This is the loser of the rule in [`closed_registry_unit`], kept so the entry
/// points can report it instead of dropping it. `None` when the text is not
/// that shape, or has no competing compound reading at all.
pub(crate) fn closed_compound_alternative(text: &str) -> Option<Reading> {
    if !closed_registry_unit(text) {
        return None;
    }
    let lowered = text.trim().to_ascii_lowercase();
    metric_compound_reading(&lowered).or_else(|| {
        feet_inches_reading(&lowered)
            .and_then(|(reading, has_inches)| has_inches.then_some(reading))
    })
}

/// Whether a closed-up compound's second part is written as a count of the
/// smaller unit — the `80` of `1m80`, the `11` of `5ft 11`.
///
/// These idioms do not write the smaller unit; the idiom supplies it. That only
/// works for a count, and a signed number is not one. `3m-20` used to read as
/// 2.8 m — the `-20` taken as minus twenty centimetres, inventing the unit and
/// the subtraction at once — while the explicit `3m-20cm` was refused, so the
/// asymmetry punished writing the unit down. Neither reads now: a sign puts the
/// text outside the idiom, and it has to find another reading or none.
pub(crate) fn unsigned_lower_place(text: &str) -> bool {
    let mut saw_digit = false;
    text.chars().all(|ch| {
        if ch.is_ascii_digit() {
            saw_digit = true;
            true
        } else {
            matches!(ch, '.' | ',' | '_')
        }
    }) && saw_digit
}

/// The shape of the metres-and-centimetres idiom: a `m` with something on each
/// side of it, and no whitespace after it.
///
/// Deliberately structural, and deliberately wider than what
/// [`metric_compound_reading`] will actually read: text of this shape belongs to
/// this grammar whether or not the grammar can read it, so `5 mM` — millimolar,
/// which lowercases into the same shape — is not handed to the suffix table
/// below to be read as millimetres.
fn metric_compound_shape(stripped: &str) -> bool {
    stripped
        .split_once('m')
        .is_some_and(|(meters, centimeters)| {
            !meters.is_empty()
                && !centimeters.is_empty()
                && !centimeters.contains(char::is_whitespace)
        })
}

/// Whether the lower part of a compound stays inside the place above it, both
/// already in canonical units. Twelve inches is a foot, not part of one, so a
/// reading that spells `1` and `12` and hands back two feet has added a place
/// the text does not have.
pub(crate) fn lower_place_stays_inside(lower_canonical: f64, place_above: f64) -> bool {
    // A part that exactly fills the place above is a carry, not a part: twelve
    // inches is the foot, as a hundred centimetres is the metre. The margin is
    // here because the two do not round the same way — `100 * 0.01` lands on
    // 1.0 and refuses, while `12 * 0.0254` lands just under 0.3048 and would
    // otherwise be let through, so the same rule would answer differently
    // depending on which units the caller happened to write.
    lower_canonical.abs() < place_above * (1.0 - 1e-9)
}

/// The metres-and-centimetres idiom itself — `1m80` as 1.8 m — over text that
/// is already trimmed and lowercased.
pub(crate) fn metric_compound_reading(stripped: &str) -> Option<Reading> {
    if !metric_compound_shape(stripped) {
        return None;
    }
    let (meters, centimeters) = stripped.split_once('m')?;
    // The centimetres are not written as centimetres — the idiom supplies the
    // unit — so the part has to be an unsigned count for that to be a reading of
    // the text rather than an invention on top of it.
    if !unsigned_lower_place(centimeters.trim()) {
        return None;
    }
    let meters = parse_number(meters.trim())?;
    let centimeters = parse_number(centimeters.trim())?;
    Some(Reading::quantity(
        meters + centimeters * CM_M,
        "m",
        Dimension::Length,
        Provenance::SiMultiple,
        false,
        0.97,
    ))
}

pub(crate) fn parse_metric_length(text: &str) -> Option<Reading> {
    let trimmed = text.trim();
    let stripped = trimmed.to_ascii_lowercase();
    // `5 m3` is the registry's cubic metre, not 5 m + 3 cm. Without this the
    // reading depended on which entry point the caller used, because the
    // fast path runs this parser before the registry lookup.
    //
    // The shape test reads the lowercased text — the idiom is case-blind — but
    // the registry lookups must see the text as written, because some aliases
    // are case-sensitive: `mM` is the registry's millimolar, and asking about
    // the lowercased `mm` instead answers for the millimetre, leaving the
    // concentration unprotected.
    if metric_compound_shape(&stripped) {
        // A token the registry knows takes the text off this grammar entirely.
        // Falling through to the suffix table instead would read the *lowercased*
        // text there, which is how `5 mM` — millimolar — came back as five
        // millimetres: the registry's answer is the one both entry points give.
        //
        // `5m3` is the registry's cubic metre; the metres-and-centimetres
        // reading survives as the alternative the entry points report through
        // [`closed_compound_alternative`].
        if spaced_registry_unit(trimmed) || closed_registry_unit(trimmed) {
            return None;
        }
        // The idiom claims text of this shape outright: `5mm` splits as `5` and
        // `m`, whose second half is no number, and falling through to the suffix
        // table below would read it as a compound that was never written.
        return metric_compound_reading(&stripped);
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
    fn one_non_ascii_mark_is_never_promoted_to_a_unit_typo() {
        for input in ["5 €", "5 💡", "5 米"] {
            let parsed = parse(input, None);
            if input.ends_with('米') {
                assert_eq!(
                    parsed.best.as_ref().and_then(|best| best.unit.as_deref()),
                    Some("m")
                );
            } else {
                assert!(parsed.best.is_none(), "{input}: {:?}", parsed.best);
                assert!(
                    parsed.suggestions.is_empty(),
                    "{input}: {:?}",
                    parsed.suggestions
                );
            }
        }
    }

    #[test]
    fn a_number_word_is_not_a_hidden_compound_place() {
        let parsed = parse("5mA", None);
        let best = parsed.best.as_ref().expect("milliamp reading");
        assert_eq!(best.unit.as_deref(), Some("A"));
        assert_eq!(best.dimension, Some(Dimension::Current));
        assert!(parsed.alternatives.is_empty());

        // The natural-language number remains supported where it is actually
        // a separate number phrase rather than a suffix split guessed by the
        // compound-height idiom.
        assert_eq!(
            parse_number_fast("a", None).best.and_then(|r| r.value),
            Some(1.0)
        );
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
