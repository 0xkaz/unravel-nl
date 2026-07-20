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
        let month = numerator as u8;
        let day = denominator as u8;
        if let Some(date) = Date::new(reference_date.year, month, day) {
            alternatives.push(Reading::date(date, 0.51));
        }
    }

    if alternatives.is_empty() {
        return None;
    }

    Some(AmbiguousParse {
        best: Some(fraction),
        alternatives,
        ambiguity: ambiguity(
            text,
            "Slash expression can be read as a fraction or a month/day date.",
            Some(2),
            IssueCode::AmbiguousDate,
        ),
    })
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
