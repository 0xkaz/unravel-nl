use crate::*;

pub(crate) fn parse_recurrence(text: &str) -> Option<Reading> {
    let trimmed = text.trim();
    if is_supported_rrule(trimmed) {
        return Some(Reading::recurrence(trimmed, 0.99));
    }

    let lowered = trimmed.to_ascii_lowercase();
    if let Some(bysetpos) = parse_english_business_day_recurrence(&lowered) {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYSETPOS={bysetpos};BYDAY=MO,TU,WE,TH,FR"),
            0.8,
        ));
    }
    if let Some(bysetpos) = parse_japanese_business_day_recurrence(trimmed) {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYSETPOS={bysetpos};BYDAY=MO,TU,WE,TH,FR"),
            0.8,
        ));
    }
    if let Some(day_text) = lowered.strip_prefix("monthly on the ") {
        if let Some(byday) = parse_english_ordinal_weekday(day_text.trim()) {
            return Some(Reading::recurrence(
                &format!("FREQ=MONTHLY;BYDAY={byday}"),
                0.84,
            ));
        }
        let day = parse_ordinal_month_day(day_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYMONTHDAY={day}"),
            0.88,
        ));
    }
    if let Some(day_text) = lowered.strip_prefix("every month on the ") {
        if let Some(byday) = parse_english_ordinal_weekday(day_text.trim()) {
            return Some(Reading::recurrence(
                &format!("FREQ=MONTHLY;BYDAY={byday}"),
                0.84,
            ));
        }
        let day = parse_ordinal_month_day(day_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYMONTHDAY={day}"),
            0.88,
        ));
    }
    if let Some(byday) = trimmed
        .strip_prefix("毎月")
        .and_then(parse_japanese_ordinal_weekday)
    {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYDAY={byday}"),
            0.84,
        ));
    }
    if let Some(day_text) = trimmed
        .strip_prefix("毎月")
        .and_then(|tail| tail.strip_suffix('日'))
    {
        let day = parse_whole_i64(day_text.trim())?;
        if (1..=31).contains(&day) {
            return Some(Reading::recurrence(
                &format!("FREQ=MONTHLY;BYMONTHDAY={day}"),
                0.88,
            ));
        }
    }

    let rrule = if matches!(lowered.as_str(), "every day" | "daily") || trimmed == "毎日" {
        "FREQ=DAILY"
    } else if matches!(lowered.as_str(), "every month" | "monthly") || trimmed == "毎月" {
        "FREQ=MONTHLY"
    } else if let Some(weekday_text) = lowered.strip_prefix("every ") {
        return parse_english_every_recurrence(weekday_text.trim());
    } else if let Some(weekday_text) = trimmed.strip_prefix("毎週") {
        let day = recurrence_weekday(weekday_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;BYDAY={day}"),
            0.9,
        ));
    } else if let Some(weekday_text) = trimmed.strip_prefix("每周") {
        let day = recurrence_weekday(weekday_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;BYDAY={day}"),
            0.9,
        ));
    } else {
        return None;
    };
    Some(Reading::recurrence(rrule, 0.92))
}

pub(crate) fn is_supported_rrule(text: &str) -> bool {
    matches!(text, "FREQ=DAILY" | "FREQ=MONTHLY")
        || text
            .strip_prefix("FREQ=DAILY;INTERVAL=")
            .is_some_and(valid_positive_i64)
        || text
            .strip_prefix("FREQ=WEEKLY;INTERVAL=")
            .is_some_and(valid_positive_i64)
        || text
            .strip_prefix("FREQ=WEEKLY;INTERVAL=")
            .is_some_and(valid_weekly_interval_byday)
        || text
            .strip_prefix("FREQ=MONTHLY;INTERVAL=")
            .is_some_and(valid_positive_i64)
        || text
            .strip_prefix("FREQ=MONTHLY;BYMONTHDAY=")
            .is_some_and(valid_month_day)
        || text
            .strip_prefix("FREQ=MONTHLY;BYDAY=")
            .is_some_and(valid_monthly_byday)
        || text
            .strip_prefix("FREQ=MONTHLY;BYSETPOS=")
            .is_some_and(valid_monthly_business_day)
        || text
            .strip_prefix("FREQ=WEEKLY;BYDAY=")
            .is_some_and(valid_weekly_byday)
}

pub(crate) fn parse_english_every_recurrence(text: &str) -> Option<Reading> {
    if let Some((base, count)) = split_recurrence_count(text) {
        let day = recurrence_weekday(base.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;BYDAY={day};COUNT={count}"),
            0.86,
        ));
    }

    if let Some(weekday_text) = text.strip_prefix("other ") {
        let day = recurrence_weekday(weekday_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;INTERVAL=2;BYDAY={day}"),
            0.84,
        ));
    }
    if let Some(day_text) = text.strip_prefix("month on the ")
        && let Some(byday) = parse_english_ordinal_weekday(day_text.trim())
    {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYDAY={byday}"),
            0.84,
        ));
    }

    let mut parts = text.split_whitespace();
    let first = parts.next()?;
    let second = parts.next();
    if parts.next().is_none()
        && let Some(unit) = second
    {
        let interval = parse_whole_i64(first)?;
        if interval <= 0 {
            return None;
        }
        let freq = match unit {
            "day" | "days" => "DAILY",
            "week" | "weeks" => "WEEKLY",
            "month" | "months" => "MONTHLY",
            _ => return None,
        };
        return Some(Reading::recurrence(
            &format!("FREQ={freq};INTERVAL={interval}"),
            0.88,
        ));
    }

    let day = recurrence_weekday(text.trim())?;
    Some(Reading::recurrence(
        &format!("FREQ=WEEKLY;BYDAY={day}"),
        0.9,
    ))
}

pub(crate) fn split_recurrence_count(text: &str) -> Option<(&str, i64)> {
    let (base, count_text) = text.rsplit_once(" for ")?;
    let count = count_text
        .strip_suffix(" times")
        .or_else(|| count_text.strip_suffix(" occurrences"))
        .or_else(|| count_text.strip_suffix(" occurrence"))?;
    let count = parse_whole_i64(count.trim())?;
    (count > 0).then_some((base, count))
}

pub(crate) fn parse_ordinal_month_day(text: &str) -> Option<i64> {
    let lower = text.trim().to_ascii_lowercase();
    let number_text = lower
        .strip_suffix("st")
        .or_else(|| lower.strip_suffix("nd"))
        .or_else(|| lower.strip_suffix("rd"))
        .or_else(|| lower.strip_suffix("th"))
        .unwrap_or(lower.as_str());
    let day = parse_whole_i64(number_text.trim())?;
    (1..=31).contains(&day).then_some(day)
}

pub(crate) fn parse_english_ordinal_weekday(text: &str) -> Option<String> {
    let text = text.strip_suffix(" of the month").unwrap_or(text).trim();
    let (ordinal_text, weekday_text) = text.split_once(' ')?;
    let ordinal = parse_recurrence_ordinal(ordinal_text)?;
    let weekday = recurrence_weekday(weekday_text.trim())?;
    Some(format!("{ordinal}{weekday}"))
}

pub(crate) fn parse_japanese_ordinal_weekday(text: &str) -> Option<String> {
    let text = text.strip_prefix('第')?;
    let digit_end = text
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_digit())
        .map(|(idx, _)| idx)?;
    let ordinal = parse_whole_i64(&text[..digit_end])?;
    if !(1..=5).contains(&ordinal) {
        return None;
    }
    let weekday = recurrence_weekday(text[digit_end..].trim())?;
    Some(format!("{ordinal}{weekday}"))
}

pub(crate) fn parse_recurrence_ordinal(text: &str) -> Option<String> {
    let ordinal = match text {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "last" => return Some("-1".to_owned()),
        _ => parse_ordinal_month_day(text)?,
    };
    (1..=5).contains(&ordinal).then(|| ordinal.to_string())
}

pub(crate) fn parse_english_business_day_recurrence(text: &str) -> Option<String> {
    let text = text
        .strip_prefix("every ")
        .or_else(|| text.strip_prefix("monthly on the "))
        .or_else(|| text.strip_prefix("every month on the "))?;
    let text = text.strip_suffix(" of the month").unwrap_or(text).trim();
    let ordinal_text = text
        .strip_suffix(" business day")
        .or_else(|| text.strip_suffix(" business days"))?
        .trim();
    parse_recurrence_ordinal(ordinal_text)
}

pub(crate) fn parse_japanese_business_day_recurrence(text: &str) -> Option<String> {
    let text = text.strip_prefix("毎月第")?;
    let ordinal_text = text
        .strip_suffix("営業日")
        .or_else(|| text.strip_suffix("業務日"))?;
    let ordinal = parse_whole_i64(ordinal_text)?;
    (1..=5).contains(&ordinal).then(|| ordinal.to_string())
}

pub(crate) fn valid_positive_i64(text: &str) -> bool {
    parse_whole_i64(text).is_some_and(|value| value > 0)
}

pub(crate) fn valid_month_day(text: &str) -> bool {
    parse_whole_i64(text).is_some_and(|value| (1..=31).contains(&value))
}

pub(crate) fn valid_weekly_byday(text: &str) -> bool {
    if let Some((day, count_text)) = text.split_once(";COUNT=") {
        return matches!(day, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU")
            && valid_positive_i64(count_text);
    }
    matches!(text, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU")
}

pub(crate) fn valid_weekly_interval_byday(text: &str) -> bool {
    let Some((interval_text, byday)) = text.split_once(";BYDAY=") else {
        return false;
    };
    valid_positive_i64(interval_text) && valid_weekly_byday(byday)
}

pub(crate) fn valid_monthly_byday(text: &str) -> bool {
    if text.len() < 3 {
        return false;
    }
    let Some((ordinal_text, weekday_text)) = text.split_at_checked(text.len() - 2) else {
        return false;
    };
    matches!(weekday_text, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU")
        && matches!(ordinal_text, "-1" | "1" | "2" | "3" | "4" | "5")
}

pub(crate) fn valid_monthly_business_day(text: &str) -> bool {
    let Some((bysetpos, byday)) = text.split_once(";BYDAY=") else {
        return false;
    };
    matches!(bysetpos, "-1" | "1" | "2" | "3" | "4" | "5") && byday == "MO,TU,WE,TH,FR"
}

pub(crate) fn recurrence_weekday(text: &str) -> Option<&'static str> {
    match text {
        "monday" | "mon" | "月曜日" | "月曜" | "月" | "周一" | "星期一" | "一" => {
            Some("MO")
        }
        "tuesday" | "tue" | "tues" | "火曜日" | "火曜" | "火" | "周二" | "星期二" | "二" => {
            Some("TU")
        }
        "wednesday" | "wed" | "水曜日" | "水曜" | "水" | "周三" | "星期三" | "三" => {
            Some("WE")
        }
        "thursday" | "thu" | "thur" | "thurs" | "木曜日" | "木曜" | "木" | "周四" | "星期四"
        | "四" => Some("TH"),
        "friday" | "fri" | "金曜日" | "金曜" | "金" | "周五" | "星期五" | "五" => {
            Some("FR")
        }
        "saturday" | "sat" | "土曜日" | "土曜" | "土" | "周六" | "星期六" | "六" => {
            Some("SA")
        }
        "sunday" | "sun" | "日曜日" | "日曜" | "日" | "周日" | "星期日" | "星期天" | "天" => {
            Some("SU")
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_no_reading(text: &str) {
        let parsed = parse(text, None);
        assert!(parsed.best.is_none(), "unexpected reading for {text:?}");
        assert!(
            !parsed.findings.skipped.is_empty(),
            "expected a skipped finding for {text:?}"
        );
    }

    #[test]
    fn monthly_byday_with_multibyte_tail_does_not_panic() {
        // `あ` is 3 bytes, so a naive byte split lands mid-character.
        assert!(!valid_monthly_byday("あ"));
        assert!(!valid_monthly_byday("あい"));
        assert!(!valid_monthly_byday("1あ"));
        assert!(!valid_monthly_byday("€uro"));
        assert!(!valid_monthly_byday("1€"));
        assert!(!valid_monthly_byday("1𝍄"));
        assert!(!valid_monthly_byday("𝍄"));
    }

    #[test]
    fn parse_multibyte_monthly_byday_yields_no_reading() {
        assert_no_reading("FREQ=MONTHLY;BYDAY=あ");
        assert_no_reading("FREQ=MONTHLY;BYDAY=あい");
        assert_no_reading("FREQ=MONTHLY;BYDAY=1あ");
        assert_no_reading("FREQ=MONTHLY;BYDAY=1€");
        assert_no_reading("FREQ=MONTHLY;BYDAY=1𝍄");
    }

    #[test]
    fn parse_recurrence_fast_multibyte_monthly_byday_yields_no_reading() {
        let parsed = parse_recurrence_fast("FREQ=MONTHLY;BYDAY=あ", None);
        assert!(parsed.best.is_none());
        assert!(!parsed.findings.skipped.is_empty());
    }

    #[test]
    fn valid_monthly_byday_still_accepts_supported_rrules() {
        assert!(valid_monthly_byday("2MO"));
        assert!(valid_monthly_byday("-1FR"));
        assert!(valid_monthly_byday("5SU"));
        assert!(!valid_monthly_byday("6MO"));
        assert!(!valid_monthly_byday("2XX"));
    }

    #[test]
    fn ordinary_recurrence_inputs_still_parse() {
        let rrule = parse("FREQ=MONTHLY;BYDAY=2MO", None).best.expect("rrule");
        assert_eq!(rrule.kind, Kind::Recurrence);
        assert_eq!(rrule.recurrence.as_deref(), Some("FREQ=MONTHLY;BYDAY=2MO"));

        let weekly = parse("every monday", None).best.expect("weekly");
        assert_eq!(weekly.kind, Kind::Recurrence);
        assert_eq!(weekly.recurrence.as_deref(), Some("FREQ=WEEKLY;BYDAY=MO"));
    }
}
