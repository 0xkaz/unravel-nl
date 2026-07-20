use crate::*;

pub(crate) fn parse_ambiguous_slash_date_or_fraction(
    text: &str,
    ctx: &ParseCtx,
) -> Option<AmbiguousParse> {
    let (left, right) = text.split_once('/')?;
    if left.is_empty() || right.is_empty() {
        return None;
    }
    let numerator = parse_number(left.trim())?;
    let denominator = parse_number(right.trim())?;
    if denominator == 0.0 {
        return None;
    }

    let fraction = Reading::number(numerator / denominator, 0.55);
    let mut alternatives = Vec::new();
    if let Some(reference_date) = ctx.reference_date {
        let first = date_component(numerator);
        let second = date_component(denominator);
        // `en-GB` writes the day first; every other locale writes the month
        // first. Both orders stay on the table, because a two-part slash date
        // carries nothing that settles the question — the competing order is
        // offered as its own alternative rather than dropped.
        let orders = if ctx.locale == Some(Locale::EnGb) {
            [(second, first), (first, second)]
        } else {
            [(first, second), (second, first)]
        };
        let mut confidence = 0.51;
        for (month, day) in orders {
            let (Some(month), Some(day)) = (month, day) else {
                continue;
            };
            // A day the calendar does not have is not a reading of the input.
            let Some(date) = checked_date(reference_date.year, month, day) else {
                continue;
            };
            let iso = date.iso();
            if alternatives
                .iter()
                .any(|reading: &Reading| reading.date.as_deref() == Some(iso.as_str()))
            {
                continue;
            }
            alternatives.push(Reading::date(date, confidence));
            confidence -= 0.02;
        }
    }

    if alternatives.is_empty() {
        return None;
    }

    let reason = if alternatives.len() > 1 {
        "Slash expression can be read as a fraction or a date, and the date can be read day-first or month-first."
    } else {
        "Slash expression can be read as a fraction or a calendar date."
    };

    let candidate_count = alternatives.len() + 1;
    Some(AmbiguousParse {
        best: Some(fraction),
        alternatives,
        ambiguity: ambiguity(
            text,
            reason,
            Some(candidate_count),
            IssueCode::AmbiguousDate,
        ),
    })
}

/// Reads one side of a slash expression as a calendar day or month number.
///
/// Only whole numbers in `1..=31` can name a month or a day, so anything else
/// (a fraction, a zero, an out-of-range count) is not a date component.
pub(crate) fn date_component(value: f64) -> Option<u8> {
    if value.fract() != 0.0 || !(1.0..=31.0).contains(&value) {
        return None;
    }
    Some(value as u8)
}

/// Builds a [`Date`] only when the calendar actually has that day.
///
/// [`Date::new`] range-checks month and day independently, so it accepts
/// `2026-02-31`. Handing such a date back as a reading would be inventing a
/// value the input cannot denote, so construction sites that build a date from
/// loose numbers go through this instead. Deliberately dependency-free: the
/// `dates-jiff` feature is off by default and this check must hold regardless.
pub(crate) fn checked_date(year: i32, month: u8, day: u8) -> Option<Date> {
    if day < 1 || day > days_in_month(year, month)? {
        return None;
    }
    Date::new(year, month, day)
}

/// Returns the number of days in `month`, or `None` when `month` is not 1..=12.
pub(crate) fn days_in_month(year: i32, month: u8) -> Option<u8> {
    Some(match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return None,
    })
}

/// Proleptic Gregorian leap-year rule, matching the calendar `jiff` uses.
pub(crate) fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn parse_relative_date(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    use jiff::{ToSpan, civil::Date as JiffDate};

    let reference = ctx.reference_date?;
    let base = to_jiff_date(reference)?;
    let lowered = text.trim().to_ascii_lowercase();

    if lowered == "today" {
        return Some(Reading::date(reference, 0.99));
    }

    if text == "今日" {
        return Some(Reading::date(reference, 0.99));
    }

    if lowered == "yesterday" || text == "昨日" || text == "昨天" {
        return from_jiff_date(base.checked_sub(1.day()).ok()?)
            .map(|date| Reading::date(date, 0.98));
    }

    if text == "一昨日" || text == "前天" {
        return from_jiff_date(base.checked_sub(2.days()).ok()?)
            .map(|date| Reading::date(date, 0.97));
    }

    if lowered == "tomorrow"
        || text == "mañana"
        || text == "demain"
        || text == "amanhã"
        || text == "明天"
    {
        return from_jiff_date(base.tomorrow().ok()?).map(|date| Reading::date(date, 0.98));
    }

    if text == "明日" {
        return from_jiff_date(base.tomorrow().ok()?).map(|date| Reading::date(date, 0.98));
    }

    if text == "明後日" || text == "后天" || text == "後天" || text == "pasado mañana" {
        return from_jiff_date(base.checked_add(2.days()).ok()?)
            .map(|date| Reading::date(date, 0.97));
    }

    if let Some(days_text) = lowered.strip_prefix("in ").and_then(|tail| {
        tail.strip_suffix(" days")
            .or_else(|| tail.strip_suffix(" day"))
    }) {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_add(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(days_text) = lowered
        .strip_suffix(" days ago")
        .or_else(|| lowered.strip_suffix(" day ago"))
    {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_sub(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(days_text) = text.strip_suffix("日後") {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_add(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(days_text) = text.strip_suffix("日前") {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_sub(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_prefix("next ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_suffix(" prochain") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_suffix(" que vem") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_prefix("this ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return date_in_current_week(base, weekday).map(|date| Reading::date(date, 0.95));
    }

    if let Some(weekday_text) = lowered.strip_prefix("last ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(-1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.95));
    }

    if let Some(weekday_text) = text.strip_prefix("来週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_prefix("下周") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_prefix("今週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return date_in_current_week(base, weekday).map(|date| Reading::date(date, 0.95));
    }

    if let Some(weekday_text) = text.strip_prefix("先週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(-1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.95));
    }

    if let Some(date) = parse_numeric_slash_date(text, ctx) {
        return Some(Reading::date(date, 0.94));
    }

    if let Ok(date) = lowered.parse::<JiffDate>() {
        return from_jiff_date(date).map(|date| Reading::date(date, 0.99));
    }

    None
}

#[cfg(not(feature = "dates-jiff"))]
pub(crate) fn parse_relative_date(_text: &str, _ctx: &ParseCtx) -> Option<Reading> {
    None
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn to_jiff_date(date: Date) -> Option<jiff::civil::Date> {
    jiff::civil::Date::new(
        i16::try_from(date.year).ok()?,
        i8::try_from(date.month).ok()?,
        i8::try_from(date.day).ok()?,
    )
    .ok()
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn from_jiff_date(date: jiff::civil::Date) -> Option<Date> {
    Date::new(
        date.year().into(),
        date.month().try_into().ok()?,
        date.day().try_into().ok()?,
    )
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn parse_numeric_slash_date(text: &str, ctx: &ParseCtx) -> Option<Date> {
    let mut parts = text.trim().split('/');
    let first = parse_whole_i64(parts.next()?.trim())?;
    let second = parse_whole_i64(parts.next()?.trim())?;
    let year = parse_whole_i64(parts.next()?.trim())?;
    if parts.next().is_some() {
        return None;
    }
    let year = i32::try_from(year).ok()?;
    let (month, day) = if ctx.locale == Some(Locale::EnGb) {
        (second, first)
    } else {
        (first, second)
    };
    let date = jiff::civil::Date::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
    )
    .ok()?;
    from_jiff_date(date)
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn date_in_current_week(
    base: jiff::civil::Date,
    weekday: jiff::civil::Weekday,
) -> Option<Date> {
    use jiff::ToSpan;

    let delta = (weekday_number(weekday) - weekday_number(base.weekday()) + 7) % 7;
    let date = base.checked_add(i64::from(delta).days()).ok()?;
    from_jiff_date(date)
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn weekday_number(weekday: jiff::civil::Weekday) -> i32 {
    match weekday {
        jiff::civil::Weekday::Monday => 1,
        jiff::civil::Weekday::Tuesday => 2,
        jiff::civil::Weekday::Wednesday => 3,
        jiff::civil::Weekday::Thursday => 4,
        jiff::civil::Weekday::Friday => 5,
        jiff::civil::Weekday::Saturday => 6,
        jiff::civil::Weekday::Sunday => 7,
    }
}

#[cfg(feature = "dates-jiff")]
pub(crate) fn parse_weekday(text: &str) -> Option<jiff::civil::Weekday> {
    match text {
        "monday" | "mon" => Some(jiff::civil::Weekday::Monday),
        "tuesday" | "tue" | "tues" => Some(jiff::civil::Weekday::Tuesday),
        "wednesday" | "wed" => Some(jiff::civil::Weekday::Wednesday),
        "thursday" | "thu" | "thur" | "thurs" => Some(jiff::civil::Weekday::Thursday),
        "friday" | "fri" => Some(jiff::civil::Weekday::Friday),
        "saturday" | "sat" => Some(jiff::civil::Weekday::Saturday),
        "sunday" | "sun" => Some(jiff::civil::Weekday::Sunday),
        "lunes" => Some(jiff::civil::Weekday::Monday),
        "martes" => Some(jiff::civil::Weekday::Tuesday),
        "miércoles" | "miercoles" => Some(jiff::civil::Weekday::Wednesday),
        "jueves" => Some(jiff::civil::Weekday::Thursday),
        "viernes" => Some(jiff::civil::Weekday::Friday),
        "sábado" | "sabado" => Some(jiff::civil::Weekday::Saturday),
        "domingo" => Some(jiff::civil::Weekday::Sunday),
        "lundi" => Some(jiff::civil::Weekday::Monday),
        "mardi" => Some(jiff::civil::Weekday::Tuesday),
        "mercredi" => Some(jiff::civil::Weekday::Wednesday),
        "jeudi" => Some(jiff::civil::Weekday::Thursday),
        "vendredi" => Some(jiff::civil::Weekday::Friday),
        "samedi" => Some(jiff::civil::Weekday::Saturday),
        "dimanche" => Some(jiff::civil::Weekday::Sunday),
        "segunda-feira" | "segunda" => Some(jiff::civil::Weekday::Monday),
        "terça-feira" | "terca-feira" | "terça" | "terca" => Some(jiff::civil::Weekday::Tuesday),
        "quarta-feira" | "quarta" => Some(jiff::civil::Weekday::Wednesday),
        "quinta-feira" | "quinta" => Some(jiff::civil::Weekday::Thursday),
        "sexta-feira" | "sexta" => Some(jiff::civil::Weekday::Friday),
        "月曜日" | "月曜" | "月" => Some(jiff::civil::Weekday::Monday),
        "火曜日" | "火曜" | "火" => Some(jiff::civil::Weekday::Tuesday),
        "水曜日" | "水曜" | "水" => Some(jiff::civil::Weekday::Wednesday),
        "木曜日" | "木曜" | "木" => Some(jiff::civil::Weekday::Thursday),
        "金曜日" | "金曜" | "金" => Some(jiff::civil::Weekday::Friday),
        "土曜日" | "土曜" | "土" => Some(jiff::civil::Weekday::Saturday),
        "日曜日" | "日曜" | "日" => Some(jiff::civil::Weekday::Sunday),
        "周一" | "星期一" | "一" => Some(jiff::civil::Weekday::Monday),
        "周二" | "星期二" | "二" => Some(jiff::civil::Weekday::Tuesday),
        "周三" | "星期三" | "三" => Some(jiff::civil::Weekday::Wednesday),
        "周四" | "星期四" | "四" => Some(jiff::civil::Weekday::Thursday),
        "周五" | "星期五" | "五" => Some(jiff::civil::Weekday::Friday),
        "周六" | "星期六" | "六" => Some(jiff::civil::Weekday::Saturday),
        "周日" | "星期日" | "星期天" | "天" => Some(jiff::civil::Weekday::Sunday),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn surfaces_slash_fraction_date_ambiguity() {
        let parsed = parse(
            "5/6",
            Some(ParseCtx {
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(parsed.best.as_ref().unwrap().kind, Kind::Number);
        assert_close(parsed.best.as_ref().unwrap().value.unwrap(), 5.0 / 6.0);
        assert_eq!(parsed.alternatives[0].kind, Kind::Date);
        assert_eq!(parsed.alternatives[0].date.as_deref(), Some("2026-05-06"));
        // Three readings now compete: the fraction, the month-first date, and
        // the day-first date that used to be dropped.
        assert_eq!(parsed.alternatives[1].date.as_deref(), Some("2026-06-05"));
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(3));
    }

    #[test]
    fn never_emits_a_date_the_calendar_does_not_have() {
        for input in ["2/31", "2/30", "4/31", "11/31", "6/31", "9/31"] {
            let parsed = parse(
                input,
                Some(ParseCtx {
                    reference_date: Date::new(2026, 7, 19),
                    ..ParseCtx::default()
                }),
            );
            for reading in parsed.best.iter().chain(parsed.alternatives.iter()) {
                assert_eq!(reading.date, None, "{input}: {reading:?}");
            }
            // The input is still accounted for rather than dropped in silence.
            assert!(
                parsed.best.is_some()
                    || !parsed.findings.skipped.is_empty()
                    || !parsed.findings.ambiguities.is_empty(),
                "{input}"
            );
        }
    }

    #[test]
    fn leap_day_is_accepted_only_in_a_leap_year() {
        let leap = parse(
            "2/29",
            Some(ParseCtx {
                reference_date: Date::new(2024, 7, 19),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(leap.alternatives[0].date.as_deref(), Some("2024-02-29"));

        let common = parse(
            "2/29",
            Some(ParseCtx {
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        for reading in common.best.iter().chain(common.alternatives.iter()) {
            assert_eq!(reading.date, None, "{reading:?}");
        }

        assert_eq!(days_in_month(2000, 2), Some(29));
        assert_eq!(days_in_month(1900, 2), Some(28));
        assert_eq!(days_in_month(2026, 13), None);
        assert_eq!(checked_date(2026, 2, 31), None);
    }

    #[test]
    fn two_part_slash_dates_honour_en_gb_day_first_order() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::EnGb),
            reference_date: Date::new(2026, 7, 19),
            ..ParseCtx::default()
        });

        let parsed = parse("5/6", ctx);
        // Day-first, matching what `5/6/2026` already yields for en-GB.
        assert_eq!(parsed.alternatives[0].date.as_deref(), Some("2026-06-05"));
        // The competing month-first reading is offered, not silently dropped.
        assert_eq!(parsed.alternatives[1].date.as_deref(), Some("2026-05-06"));
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousDate
        );
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(3));
    }

    #[test]
    fn unambiguous_day_number_yields_one_date_reading() {
        // 25 cannot be a month, so only one date order is plausible.
        let parsed = parse(
            "5/25",
            Some(ParseCtx {
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.alternatives[0].date.as_deref(), Some("2026-05-25"));
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(2));
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_next_friday_with_jiff() {
        let parsed = parse(
            "next friday",
            Some(ParseCtx {
                locale: Some(Locale::En),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Date);
        assert_eq!(best.date.as_deref(), Some("2026-07-24"));
        assert!(parsed.findings.skipped.is_empty());
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_in_days_with_jiff() {
        let parsed = parse(
            "in 3 days",
            Some(ParseCtx {
                locale: Some(Locale::En),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Date);
        assert_eq!(best.date.as_deref(), Some("2026-07-22"));
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_iso_date_with_jiff() {
        let parsed = parse(
            "2026-07-19",
            Some(ParseCtx {
                locale: Some(Locale::En),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Date);
        assert_eq!(best.date.as_deref(), Some("2026-07-19"));
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_japanese_relative_dates_with_jiff() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        });

        assert_eq!(
            parse("明日", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-20")
        );
        assert_eq!(
            parse("3日後", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-22")
        );
        assert_eq!(
            parse("来週金曜日", ctx).best.unwrap().date.as_deref(),
            Some("2026-07-24")
        );
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_broader_relative_dates_with_jiff() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::En),
            reference_date: Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        });

        assert_eq!(
            parse("yesterday", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-18")
        );
        assert_eq!(
            parse("2 days ago", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-17")
        );
        assert_eq!(
            parse("this friday", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-24")
        );
        assert_eq!(
            parse("last friday", ctx).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_broader_japanese_relative_dates_with_jiff() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        });

        assert_eq!(
            parse("昨日", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-18")
        );
        assert_eq!(
            parse("一昨日", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
        assert_eq!(
            parse("2日前", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
        assert_eq!(
            parse("今週金曜日", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-24")
        );
        assert_eq!(
            parse("先週金曜日", ctx).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_japanese_date_range_with_jiff() {
        let parsed = parse(
            "今日〜明日",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.date.as_deref(), Some("2026-07-19"));
        assert_eq!(range.to.date.as_deref(), Some("2026-07-20"));
    }
}
