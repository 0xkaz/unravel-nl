use crate::*;

/// True when the whole text is a number and a token the registry resolves to
/// something that is not a duration.
///
/// `5 W` is five watts: `W` is the registry's watt, while the compact grammar
/// below reads a bare `w` as a week. A declared registry unit outranks that
/// informal single-letter idiom — the same rule [`spaced_registry_unit`] and
/// [`closed_registry_unit`] apply — and without it the reading depended on the
/// entry point, because the fast quantity dispatch runs this grammar before the
/// registry lookup while `parse` runs the registry first.
///
/// Tokens the registry also reads as time are deliberately left alone: `5 h`,
/// `5 d`, `5 s` and `5 min` mean the same thing either way, so there is nothing
/// to decide and no reason to move them onto a different grammar.
fn registry_unit_outranks_duration(text: &str) -> bool {
    split_number_unit(text.trim()).is_some_and(|(_, unit_text)| {
        unit_by_alias(unit_text).is_some_and(|unit| unit.dimension != Dimension::Time)
    })
}

pub(crate) fn parse_duration(text: &str) -> Option<Reading> {
    if registry_unit_outranks_duration(text) {
        return None;
    }
    let stripped = text.trim().to_ascii_lowercase();
    if stripped == "an hour and a half" || stripped == "one hour and a half" {
        return Some(Reading::quantity(
            5400.0,
            "s",
            Dimension::Time,
            Provenance::SiMultiple,
            false,
            0.97,
        ));
    }

    if let Some(seconds) = parse_iso_duration(&stripped) {
        return Some(Reading::quantity(
            seconds,
            "s",
            Dimension::Time,
            Provenance::SiMultiple,
            false,
            0.94,
        ));
    }

    if let Some(seconds) = parse_compact_duration(&stripped) {
        return Some(Reading::quantity(
            seconds,
            "s",
            Dimension::Time,
            Provenance::SiMultiple,
            false,
            0.94,
        ));
    }

    for (suffix, factor) in [
        ("minutes", 60.0),
        ("minute", 60.0),
        ("mins", 60.0),
        ("min", 60.0),
        ("hours", 3600.0),
        ("hour", 3600.0),
        ("hrs", 3600.0),
        ("hr", 3600.0),
        ("h", 3600.0),
        ("日", 86_400.0),
        ("days", 86_400.0),
        ("day", 86_400.0),
        ("d", 86_400.0),
    ] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value * factor,
                "s",
                Dimension::Time,
                Provenance::SiMultiple,
                false,
                0.96,
            ));
        }
    }
    None
}

pub(crate) fn parse_iso_duration(text: &str) -> Option<f64> {
    let mut chars = text.trim().chars();
    if !chars.next()?.eq_ignore_ascii_case(&'P') {
        return None;
    }

    let mut seconds = 0.0;
    let mut number = String::new();
    let mut in_time = false;
    let mut saw_component = false;

    for ch in chars {
        if ch == 'T' || ch == 't' {
            if in_time || !number.is_empty() {
                return None;
            }
            in_time = true;
            continue;
        }

        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
            continue;
        }

        if number.is_empty() {
            return None;
        }

        let value = number.parse::<f64>().ok()?;
        number.clear();
        match ch.to_ascii_uppercase() {
            'W' if !in_time => seconds += value * 7.0 * 86_400.0,
            'D' if !in_time => seconds += value * 86_400.0,
            'H' if in_time => seconds += value * 3600.0,
            'M' if in_time => seconds += value * 60.0,
            'S' if in_time => seconds += value,
            _ => return None,
        }
        saw_component = true;
    }

    if !number.is_empty() {
        return None;
    }
    saw_component.then_some(seconds)
}

pub(crate) fn parse_compact_duration(text: &str) -> Option<f64> {
    if !text.trim().starts_with(|ch: char| ch.is_ascii_digit()) {
        return None;
    }

    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.is_empty() {
        return None;
    }

    let mut rest = compact.as_str();
    let mut seconds = 0.0;
    let mut saw_component = false;
    let mut last_unit: Option<&str> = None;

    while !rest.is_empty() {
        let number_end = rest
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_digit() || *ch == '.')
            .map(|(idx, ch)| idx + ch.len_utf8())
            .last()
            .unwrap_or(0);
        if number_end == 0 {
            return None;
        }

        let value = rest[..number_end].parse::<f64>().ok()?;
        rest = &rest[number_end..];
        if rest.is_empty() {
            if last_unit == Some("h") {
                seconds += value * 60.0;
                saw_component = true;
                break;
            }
            return None;
        }

        let unit_end = rest
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphabetic())
            .map(|(idx, ch)| idx + ch.len_utf8())
            .last()
            .unwrap_or(0);
        if unit_end == 0 {
            return None;
        }

        let unit = &rest[..unit_end];
        let factor = match unit {
            "w" => 7.0 * 86_400.0,
            "d" => 86_400.0,
            "h" | "hr" | "hrs" => 3600.0,
            "m" | "min" | "mins" => 60.0,
            "s" | "sec" | "secs" => 1.0,
            _ => return None,
        };
        seconds += value * factor;
        saw_component = true;
        last_unit = Some(if matches!(unit, "hr" | "hrs") {
            "h"
        } else {
            unit
        });
        rest = &rest[unit_end..];
    }

    saw_component.then_some(seconds)
}

pub(crate) fn parse_clock_time(text: &str) -> Option<Reading> {
    let seconds = parse_clock_seconds(text)?;
    Some(Reading::quantity(
        seconds,
        "s",
        Dimension::Time,
        Provenance::TradeCustom,
        false,
        0.92,
    ))
}

/// Reads a wall-clock time with a timezone suffix, normalized to UTC.
///
/// Returns the reading together with the civil day shift the conversion
/// implied: `-1` when the UTC instant falls on the previous day, `1` when it
/// falls on the next. The reading carries only a seconds-of-day value, so the
/// caller has to surface that shift as a finding rather than let it vanish.
pub(crate) fn parse_timezone_clock_time(text: &str, ctx: &ParseCtx) -> Option<(Reading, i64)> {
    let trimmed = text.trim();
    let zone = trimmed.split_whitespace().last()?;
    let head = trimmed.strip_suffix(zone)?.trim_end();
    let seconds = parse_clock_seconds(head)?;
    let offset = timezone_offset_seconds(zone)
        .or_else(|| iana_timezone_offset_seconds(zone, seconds, ctx.reference_date))?;
    let shifted = seconds - f64::from(offset);
    let day_shift = (shifted / 86_400.0).floor() as i64;
    let utc_seconds = modulo_day(shifted);
    let mut reading = Reading::quantity(
        utc_seconds,
        "s",
        Dimension::Time,
        Provenance::TradeCustom,
        false,
        0.9,
    );
    reading.timezone = Some("UTC".to_owned());
    Some((reading, day_shift))
}

pub(crate) fn unsupported_timezone_suffix(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let timezone = trimmed.split_whitespace().last()?;
    let head = trimmed.strip_suffix(timezone)?.trim_end();
    parse_clock_time(head)?;
    is_timezone_token(timezone).then_some(timezone)
}

pub(crate) fn timezone_offset_seconds(text: &str) -> Option<i32> {
    match text {
        "UTC" | "GMT" => Some(0),
        "EST" => Some(-5 * 3600),
        "EDT" => Some(-4 * 3600),
        "CST" => Some(-6 * 3600),
        "CDT" => Some(-5 * 3600),
        "MST" => Some(-7 * 3600),
        "MDT" => Some(-6 * 3600),
        "PST" => Some(-8 * 3600),
        "PDT" => Some(-7 * 3600),
        "JST" => Some(9 * 3600),
        "KST" => Some(9 * 3600),
        "CET" => Some(3600),
        "CEST" => Some(2 * 3600),
        "BST" => Some(3600),
        "AEST" => Some(10 * 3600),
        "AEDT" => Some(11 * 3600),
        _ => parse_utc_offset_seconds(text),
    }
}

pub(crate) fn parse_utc_offset_seconds(text: &str) -> Option<i32> {
    let offset = text
        .strip_prefix("UTC")
        .or_else(|| text.strip_prefix("GMT"))?;
    let (sign, signless) = if let Some(signless) = offset.strip_prefix('+') {
        (1, signless)
    } else if let Some(signless) = offset.strip_prefix('-') {
        (-1, signless)
    } else {
        return None;
    };
    let (hours, minutes) = signless.split_once(':').unwrap_or((signless, "00"));
    if hours.is_empty()
        || hours.len() > 2
        || minutes.len() != 2
        || !hours.chars().all(|ch| ch.is_ascii_digit())
        || !minutes.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let hours = hours.parse::<i32>().ok()?;
    let minutes = minutes.parse::<i32>().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Resolves an IANA zone offset for a wall-clock time on the reference day.
///
/// Every step is fallible on purpose. `ParseCtx::reference_date` is built by
/// [`Date::new`], which range-checks month and day independently and so accepts
/// days the calendar does not have (`2026-02-31`), and its year is an `i32`
/// while `jiff` only models `-9999..=9999`. Both are rejected here — through
/// [`checked_date`] and [`to_jiff_date`], the same gates the date parser uses —
/// rather than handed to a panicking `jiff` constructor. Returning `None` lets
/// the caller fall through to the `TimezoneUnsupported` skipped finding, so the
/// input is reported rather than dropped or crashed on.
#[cfg(feature = "timezones-jiff")]
pub(crate) fn iana_timezone_offset_seconds(
    zone: &str,
    seconds: f64,
    reference_date: Option<Date>,
) -> Option<i32> {
    if !is_iana_timezone_name(zone) {
        return None;
    }
    let date = reference_date?;
    let date = to_jiff_date(checked_date(date.year, date.month, date.day)?)?;
    if !seconds.is_finite() {
        return None;
    }
    let clock_seconds = seconds.round() as i64;
    let hour = i8::try_from(clock_seconds / 3600).ok()?;
    let minute = i8::try_from((clock_seconds % 3600) / 60).ok()?;
    let second = i8::try_from(clock_seconds % 60).ok()?;
    let time = jiff::civil::Time::new(hour, minute, second, 0).ok()?;
    let datetime = date.to_datetime(time);
    let (canonical_name, data) = jiff_tzdb::get(zone)?;
    let timezone = jiff::tz::TimeZone::tzif(canonical_name, data).ok()?;
    datetime
        .to_zoned(timezone)
        .ok()
        .map(|zoned| zoned.offset().seconds())
}

#[cfg(not(feature = "timezones-jiff"))]
pub(crate) fn iana_timezone_offset_seconds(
    _zone: &str,
    _seconds: f64,
    _reference_date: Option<Date>,
) -> Option<i32> {
    None
}

pub(crate) fn modulo_day(seconds: f64) -> f64 {
    seconds.rem_euclid(86_400.0)
}

pub(crate) fn is_timezone_token(text: &str) -> bool {
    if timezone_offset_seconds(text).is_some() {
        return true;
    }
    if is_iana_timezone_name(text) {
        return true;
    }
    if matches!(
        text,
        "UTC"
            | "GMT"
            | "EST"
            | "EDT"
            | "CST"
            | "CDT"
            | "MST"
            | "MDT"
            | "PST"
            | "PDT"
            | "JST"
            | "KST"
            | "CET"
            | "CEST"
            | "BST"
            | "AEST"
            | "AEDT"
    ) {
        return true;
    }

    let Some(offset) = text
        .strip_prefix("UTC")
        .or_else(|| text.strip_prefix("GMT"))
    else {
        return false;
    };
    let Some(signless) = offset
        .strip_prefix('+')
        .or_else(|| offset.strip_prefix('-'))
    else {
        return false;
    };
    let (hours, minutes) = signless.split_once(':').unwrap_or((signless, "00"));
    hours.len() <= 2
        && !hours.is_empty()
        && minutes.len() == 2
        && hours.chars().all(|ch| ch.is_ascii_digit())
        && minutes.chars().all(|ch| ch.is_ascii_digit())
}

pub(crate) fn is_iana_timezone_name(text: &str) -> bool {
    text.contains('/')
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '+'))
}

pub(crate) fn parse_clock_seconds(text: &str) -> Option<f64> {
    if let Some(seconds) = parse_japanese_clock_seconds(text) {
        return Some(seconds);
    }

    let lowered = text.trim().to_ascii_lowercase();
    let compact: String = lowered.chars().filter(|ch| !ch.is_whitespace()).collect();

    if compact == "noon" {
        return Some(12.0 * 3600.0);
    }
    if compact == "midnight" {
        return Some(0.0);
    }

    let (body, meridiem) = if let Some(body) = compact.strip_suffix("am") {
        (body, Some("am"))
    } else if let Some(body) = compact.strip_suffix("pm") {
        (body, Some("pm"))
    } else {
        (compact.as_str(), None)
    };

    let (hour_text, minute_text) = body.split_once(':').unwrap_or((body, "0"));
    if hour_text.is_empty()
        || minute_text.is_empty()
        || !hour_text.chars().all(|ch| ch.is_ascii_digit())
        || !minute_text.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    let mut hour = hour_text.parse::<u8>().ok()?;
    let minute = minute_text.parse::<u8>().ok()?;
    if minute > 59 {
        return None;
    }

    match meridiem {
        Some("am") => {
            if hour == 0 || hour > 12 {
                return None;
            }
            if hour == 12 {
                hour = 0;
            }
        }
        Some("pm") => {
            if hour == 0 || hour > 12 {
                return None;
            }
            if hour != 12 {
                hour += 12;
            }
        }
        Some(_) => return None,
        None => {
            if !body.contains(':') || hour > 23 {
                return None;
            }
        }
    }

    Some(f64::from(hour) * 3600.0 + f64::from(minute) * 60.0)
}

pub(crate) fn parse_japanese_clock_seconds(text: &str) -> Option<f64> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    let (body, meridiem) = if let Some(body) = compact.strip_prefix("午前") {
        (body, Some("am"))
    } else if let Some(body) = compact.strip_prefix("午後") {
        (body, Some("pm"))
    } else {
        (compact.as_str(), None)
    };

    let (hour_text, minute_tail) = body.split_once('時')?;
    if hour_text.is_empty() {
        return None;
    }
    let mut hour = hour_text.parse::<u8>().ok()?;
    let minute = if minute_tail.is_empty() {
        0
    } else if minute_tail == "半" {
        30
    } else {
        minute_tail.strip_suffix('分')?.parse::<u8>().ok()?
    };
    if minute > 59 {
        return None;
    }

    match meridiem {
        Some("am") => {
            if hour == 0 || hour > 12 {
                return None;
            }
            if hour == 12 {
                hour = 0;
            }
        }
        Some("pm") => {
            if hour == 0 || hour > 12 {
                return None;
            }
            if hour != 12 {
                hour += 12;
            }
        }
        Some(_) => return None,
        None => {
            if hour > 23 {
                return None;
            }
        }
    }

    Some(f64::from(hour) * 3600.0 + f64::from(minute) * 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn parses_natural_duration_phrase() {
        let parsed = parse("an hour and a half", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("s"));
        assert_eq!(best.dimension, Some(Dimension::Time));
        assert_close(best.value.unwrap(), 5400.0);
    }

    #[test]
    fn parses_iso_duration_forms() {
        let hour_half = parse("PT1H30M", None).best.expect("iso duration");
        assert_eq!(hour_half.unit.as_deref(), Some("s"));
        assert_eq!(hour_half.dimension, Some(Dimension::Time));
        assert_close(hour_half.value.unwrap(), 5400.0);

        let days = parse("P2D", None).best.expect("day duration");
        assert_close(days.value.unwrap(), 172_800.0);
    }

    #[test]
    fn parses_compact_duration_forms() {
        let hour_half = parse("1h30", None).best.expect("compact duration");
        assert_eq!(hour_half.unit.as_deref(), Some("s"));
        assert_eq!(hour_half.dimension, Some(Dimension::Time));
        assert_close(hour_half.value.unwrap(), 5400.0);

        let days_hours = parse("2d4h", None).best.expect("compound duration");
        assert_close(days_hours.value.unwrap(), 187_200.0);
    }

    #[test]
    fn parses_clock_time_forms() {
        let afternoon = parse("3pm", None).best.expect("clock time");
        assert_eq!(afternoon.unit.as_deref(), Some("s"));
        assert_eq!(afternoon.dimension, Some(Dimension::Time));
        assert_close(afternoon.value.unwrap(), 15.0 * 3600.0);

        let twenty_four = parse("14:30", None).best.expect("24h clock");
        assert_close(twenty_four.value.unwrap(), 14.0 * 3600.0 + 30.0 * 60.0);

        let noon = parse("noon", None).best.expect("noon");
        assert_close(noon.value.unwrap(), 12.0 * 3600.0);

        let japanese_afternoon = parse("午後3時", None).best.expect("Japanese afternoon");
        assert_eq!(japanese_afternoon.unit.as_deref(), Some("s"));
        assert_eq!(japanese_afternoon.dimension, Some(Dimension::Time));
        assert_close(japanese_afternoon.value.unwrap(), 15.0 * 3600.0);

        let japanese_morning = parse("午前9時30分", None).best.expect("Japanese morning");
        assert_close(japanese_morning.value.unwrap(), 9.5 * 3600.0);

        let japanese_half = parse("午後3時半", None).best.expect("Japanese half hour");
        assert_close(japanese_half.value.unwrap(), 15.5 * 3600.0);
    }

    #[test]
    fn parses_timezone_qualified_clock_to_utc() {
        let parsed = parse("3pm EST", None);
        let best = parsed.best.expect("timezone clock");
        assert_eq!(best.unit.as_deref(), Some("s"));
        assert_eq!(best.dimension, Some(Dimension::Time));
        assert_eq!(best.timezone.as_deref(), Some("UTC"));
        assert_close(best.value.unwrap(), 20.0 * 3600.0);

        let tokyo = parse("9:30 JST", None).best.expect("JST clock");
        assert_eq!(tokyo.timezone.as_deref(), Some("UTC"));
        assert_close(tokyo.value.unwrap(), 30.0 * 60.0);
    }

    #[cfg(feature = "timezones-jiff")]
    #[test]
    fn parses_iana_timezone_with_explicit_reference_date() {
        let summer = parse(
            "3pm Europe/Paris",
            Some(ParseCtx {
                reference_date: Date::new(2026, 7, 20),
                ..ParseCtx::default()
            }),
        )
        .best
        .expect("summer IANA timezone");
        assert_eq!(summer.timezone.as_deref(), Some("UTC"));
        assert_close(summer.value.unwrap(), 13.0 * 3600.0);

        let winter = parse(
            "3pm Europe/Paris",
            Some(ParseCtx {
                reference_date: Date::new(2026, 1, 20),
                ..ParseCtx::default()
            }),
        )
        .best
        .expect("winter IANA timezone");
        assert_eq!(winter.timezone.as_deref(), Some("UTC"));
        assert_close(winter.value.unwrap(), 14.0 * 3600.0);
    }

    /// A reference date the calendar does not have, or a year `jiff` cannot
    /// model, must not reach a panicking `jiff` constructor.
    ///
    /// `Date::new` range-checks month and day independently, so a caller can
    /// hand in `2026-02-31`, and its year is an `i32` while `jiff` models only
    /// `-9999..=9999`. Both used to panic inside `jiff::civil::date`. The
    /// timezone is simply unresolvable against such a date, so the input falls
    /// through to the existing `TimezoneUnsupported` finding.
    #[cfg(feature = "timezones-jiff")]
    #[test]
    fn impossible_reference_dates_do_not_panic() {
        let impossible = [
            Date::new(2026, 2, 31),
            Date::new(2026, 2, 30),
            Date::new(2026, 4, 31),
            Date::new(2026, 6, 31),
            Date::new(20000, 1, 1),
        ];
        for reference_date in impossible {
            assert!(reference_date.is_some(), "Date::new accepts these today");
            let ctx = ParseCtx {
                reference_date,
                ..ParseCtx::default()
            };

            let parsed = parse("3pm America/New_York", Some(ctx.clone()));
            assert!(parsed.best.is_none(), "{reference_date:?}");
            assert!(
                parsed
                    .findings
                    .skipped
                    .iter()
                    .any(|issue| issue.code == IssueCode::TimezoneUnsupported),
                "{reference_date:?} must be reported, not dropped: {:?}",
                parsed.findings
            );

            // The completion route reaches the same code and used to panic too.
            let completions = complete_readings("3pm America/New_York", Some(ctx));
            assert!(
                completions
                    .iter()
                    .all(|candidate| candidate.reading.timezone.as_deref() != Some("UTC")),
                "{reference_date:?}"
            );
        }
    }

    #[test]
    fn rejects_unsupported_timezone_policy() {
        let parsed = parse("3pm Europe/Paris", None);
        assert!(parsed.best.is_none());
        assert_eq!(
            parsed.findings.skipped[0].code,
            IssueCode::TimezoneUnsupported
        );
        assert_eq!(parsed.findings.skipped[0].ref_text, "Europe/Paris");
        assert_eq!(parsed.findings.skipped[0].span.start, 4);
        assert_eq!(parsed.findings.skipped[0].span.end, 16);
    }

    #[test]
    fn reports_the_civil_day_shift_a_timezone_conversion_implies() {
        // 1:00 JST is 16:00 UTC on the PREVIOUS civil day. The reading carries
        // only a seconds-of-day value, so the day shift has to be surfaced.
        let parsed = parse("1:00 JST", None);
        let best = parsed.best.as_ref().expect("a reading");

        assert_eq!(best.value, Some(57_600.0));
        assert_eq!(best.timezone.as_deref(), Some("UTC"));
        assert_eq!(parsed.findings.approximations.len(), 1);
        assert_eq!(
            parsed.findings.approximations[0].code,
            IssueCode::Approximation
        );
        assert!(
            parsed.findings.approximations[0]
                .reason
                .contains("previous civil day")
        );
    }

    #[test]
    fn does_not_flag_a_conversion_that_stays_on_the_same_day() {
        let parsed = parse("9:30 UTC", None);

        assert_eq!(parsed.best.as_ref().and_then(|r| r.value), Some(34_200.0));
        assert!(parsed.findings.approximations.is_empty());
    }

    #[test]
    fn reports_a_forward_day_shift_too() {
        // 23:00 in a negative-offset zone lands on the NEXT UTC civil day.
        let parsed = parse("23:00 EST", None);
        let best = parsed.best.as_ref().expect("a reading");

        assert_eq!(best.value, Some(14_400.0));
        assert_eq!(parsed.findings.approximations.len(), 1);
        assert!(
            parsed.findings.approximations[0]
                .reason
                .contains("next civil day")
        );
    }
}
