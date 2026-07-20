use crate::*;
use std::borrow::Cow;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct InputFeatures {
    pub(crate) maybe_number: bool,
    pub(crate) maybe_quantity: bool,
    pub(crate) maybe_compound_quantity: bool,
    pub(crate) maybe_japanese_length: bool,
    pub(crate) maybe_tatami: bool,
    pub(crate) maybe_tsubo: bool,
    pub(crate) maybe_area: bool,
    pub(crate) maybe_temperature: bool,
    pub(crate) maybe_metric_length: bool,
    pub(crate) maybe_mass: bool,
    pub(crate) maybe_duration: bool,
    pub(crate) maybe_clock: bool,
    pub(crate) maybe_timezone_clock: bool,
    pub(crate) maybe_feet_inches: bool,
    pub(crate) maybe_cups: bool,
    pub(crate) maybe_currency: bool,
    pub(crate) maybe_conversion: bool,
    pub(crate) maybe_range: bool,
    pub(crate) maybe_date: bool,
    pub(crate) maybe_recurrence: bool,
    pub(crate) maybe_suggestion: bool,
    pub(crate) has_slash: bool,
}

impl InputFeatures {
    pub(crate) fn new(text: &str) -> Self {
        let trimmed = text.trim();
        let lower_cow = ascii_lower_cow(trimmed);
        let lower = lower_cow.as_ref();
        let has_ascii_digit = trimmed.as_bytes().iter().any(u8::is_ascii_digit);
        let has_cjk_number = trimmed.chars().any(is_cjk_number_char);
        let has_number_word = lower.split_whitespace().any(is_english_number_word_like);
        let has_number = has_ascii_digit
            || has_cjk_number
            || has_number_word
            || lower
                .split(|ch: char| !ch.is_ascii_alphabetic() && ch != '-')
                .any(|word| small_number_word(word).is_some() || matches!(word, "a" | "an"));
        let has_ascii_alpha =
            trimmed.is_ascii() && trimmed.bytes().any(|byte| byte.is_ascii_alphabetic());
        let maybe_date = lower.starts_with("next ")
            || lower.starts_with("last ")
            || lower.starts_with("this ")
            || lower.starts_with("in ")
            || lower.ends_with(" ago")
            || matches!(lower, "today" | "tomorrow" | "yesterday")
            || [
                "mañana",
                "pasado mañana",
                "demain",
                "amanhã",
                "vendredi prochain",
                "sexta-feira que vem",
                "viernes próximo",
                "viernes proximo",
                "后天",
            ]
            .iter()
            .any(|token| lower == *token || lower.contains(token))
            || trimmed.contains('日')
            || trimmed.contains('週')
            || trimmed.contains('周')
            || (has_ascii_digit && (trimmed.contains('-') || trimmed.contains('/')))
            || matches!(
                trimmed,
                "今日" | "明日" | "昨日" | "一昨日" | "明天" | "昨天" | "前天"
            );
        let maybe_recurrence = lower.starts_with("every ")
            || lower.starts_with("monthly")
            || lower.starts_with("daily")
            || lower.starts_with("freq=")
            || trimmed.starts_with("毎")
            || trimmed.starts_with("每");
        let maybe_clock = lower.contains("am")
            || lower.contains("pm")
            || lower == "noon"
            || lower == "midnight"
            || trimmed.contains(':')
            || trimmed.contains('時');
        let maybe_currency = trimmed.starts_with(['$', '€', '£', '¥', '￥'])
            || [
                "usd", "eur", "gbp", "jpy", "bucks", "dollars", "euros", "pounds", "yen", "円",
                "cent", "cents", "pence",
            ]
            .iter()
            .any(|token| lower.contains(token));
        let maybe_temperature = trimmed.contains('°')
            || trimmed.contains('℃')
            || trimmed.contains('℉')
            || trimmed.contains("摂氏")
            || trimmed.contains("華氏")
            || ["celsius", "fahrenheit", "kelvin"]
                .iter()
                .any(|token| lower.contains(token))
            || (has_number && lower.ends_with(['c', 'f', 'k']));
        let maybe_range = trimmed.contains('±')
            || lower.contains("+/-")
            || lower.contains(" to ")
            || lower.contains("between ")
            || lower.contains("from ")
            || trimmed.contains(['〜', '～'])
            || trimmed.contains("..")
            || trimmed.contains('≤')
            || trimmed.contains('<')
            || trimmed.contains('-')
            || ["less than ", "under ", "below ", "up to ", "at most "]
                .iter()
                .any(|prefix| lower.starts_with(prefix))
            || trimmed.ends_with("以下")
            || trimmed.ends_with("未満")
            || trimmed.ends_with("まで");
        let maybe_duration = lower.starts_with('p')
            || lower.contains("hour")
            || lower.contains("minute")
            || lower.contains("min")
            || lower.contains("day")
            || lower.contains("week")
            || lower.contains("few ")
            || lower.contains("an hour")
            || (has_number
                && [
                    "h", "hr", "hrs", "m", "min", "mins", "s", "sec", "secs", "d",
                ]
                .iter()
                .any(|unit| lower.contains(unit)))
            || trimmed.ends_with('日');
        let maybe_quantity = has_number
            || trimmed.starts_with('約')
            || lower.starts_with("about ")
            || lower.starts_with("around ")
            || lower.starts_with("roughly ")
            || lower.starts_with("approximately ");

        Self {
            maybe_number: has_number,
            maybe_quantity,
            maybe_compound_quantity: maybe_quantity && trimmed.split_whitespace().count() >= 4,
            maybe_japanese_length: maybe_quantity && trimmed.contains(['尺', '寸', '間']),
            maybe_tatami: maybe_quantity && trimmed.contains(['帖', '畳']),
            maybe_tsubo: maybe_quantity && trimmed.contains('坪'),
            maybe_area: maybe_quantity
                && (trimmed.contains('㎡')
                    || trimmed.contains('²')
                    || lower.contains("m2")
                    || lower.contains("m^2")
                    || trimmed.contains("平米")
                    || trimmed.contains("平方米")),
            maybe_temperature,
            maybe_metric_length: maybe_quantity
                && (["cm", "mm", "in", "inch", "inches", "ft", "feet", "m"]
                    .iter()
                    .any(|suffix| lower.ends_with(suffix))
                    || lower.contains('m')),
            maybe_mass: (maybe_quantity
                && [
                    "kg",
                    "kilogram",
                    "kilograms",
                    "lb",
                    "lbs",
                    "pound",
                    "pounds",
                    "ounce",
                    "ounces",
                    "oz",
                    "g",
                ]
                .iter()
                .any(|suffix| lower.ends_with(suffix)))
                || ["公斤", "千克", "キログラム", "キロ"]
                    .iter()
                    .any(|suffix| trimmed.ends_with(suffix)),
            maybe_duration,
            maybe_clock,
            maybe_timezone_clock: maybe_clock && trimmed.split_whitespace().count() >= 2,
            maybe_feet_inches: maybe_quantity
                && (lower.contains("ft") || lower.contains("feet") || trimmed.contains('\'')),
            maybe_cups: maybe_quantity && (lower.ends_with("cup") || lower.ends_with("cups")),
            maybe_currency,
            maybe_conversion: lower.contains(" to "),
            maybe_range,
            maybe_date,
            maybe_recurrence,
            maybe_suggestion: has_ascii_alpha && trimmed.len() <= 160,
            has_slash: trimmed.contains('/'),
        }
    }
}

pub(crate) fn normalize_input_cow(text: &str) -> Cow<'_, str> {
    if !text.chars().any(needs_input_normalization) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(normalize_input(text))
}

pub(crate) fn ascii_lower_cow(text: &str) -> Cow<'_, str> {
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(text.to_ascii_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

pub(crate) fn is_english_number_word_like(word: &str) -> bool {
    word.split('-').filter(|part| !part.is_empty()).all(|part| {
        small_number_word(part).is_some()
            || matches!(part, "hundred" | "thousand" | "a" | "an" | "and")
    })
}

pub(crate) fn normalize_input(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
            '\u{00A0}' | '\u{202F}' | '\u{2009}' | '\u{2007}' => normalized.push(' '),
            '０'..='９' => {
                let digit = (ch as u32) - ('０' as u32);
                normalized.push(char::from_u32(('0' as u32) + digit).unwrap_or(ch));
            }
            'Ａ'..='Ｚ' => {
                let letter = (ch as u32) - ('Ａ' as u32);
                normalized.push(char::from_u32(('A' as u32) + letter).unwrap_or(ch));
            }
            'ａ'..='ｚ' => {
                let letter = (ch as u32) - ('ａ' as u32);
                normalized.push(char::from_u32(('a' as u32) + letter).unwrap_or(ch));
            }
            '．' | '。' => normalized.push('.'),
            '，' | '、' if looks_numeric_separator(&normalized) => normalized.push(','),
            '＋' => normalized.push('+'),
            '－' | '−' | '–' => normalized.push('-'),
            '／' => normalized.push('/'),
            '＊' | '×' => normalized.push('*'),
            '＾' => normalized.push('^'),
            '％' => normalized.push('%'),
            '　' => normalized.push(' '),
            '㍍' => normalized.push('m'),
            '㌢' => normalized.push_str("cm"),
            '㍉' => normalized.push_str("mm"),
            '㌔' => normalized.push_str("キロ"),
            '㌘' => normalized.push('g'),
            '㎏' => normalized.push_str("kg"),
            '㎎' => normalized.push_str("mg"),
            '㎜' => normalized.push_str("mm"),
            '㎝' => normalized.push_str("cm"),
            '㎞' => normalized.push_str("km"),
            '㏄' => normalized.push_str("cc"),
            _ => normalized.push(ch),
        }
    }
    normalized
}

pub(crate) fn needs_input_normalization(ch: char) -> bool {
    matches!(
        ch,
        '\u{200B}'
            | '\u{200C}'
            | '\u{200D}'
            | '\u{FEFF}'
            | '\u{00A0}'
            | '\u{202F}'
            | '\u{2009}'
            | '\u{2007}'
            | '０'..='９'
            | 'Ａ'..='Ｚ'
            | 'ａ'..='ｚ'
            | '．'
            | '。'
            | '，'
            | '＋'
            | '－'
            | '−'
            | '–'
            | '／'
            | '＊'
            | '×'
            | '＾'
            | '％'
            | '　'
            | '㍍'
            | '㌢'
            | '㍉'
            | '㌔'
            | '㌘'
            | '㎏'
            | '㎎'
            | '㎜'
            | '㎝'
            | '㎞'
            | '㏄'
    )
}

pub(crate) fn looks_numeric_separator(prefix: &str) -> bool {
    prefix
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch.is_ascii_digit())
}

pub(crate) fn is_cjk_number_char(ch: char) -> bool {
    cjk_digit(ch).is_some() || matches!(ch, '十' | '百' | '千' | '万' | '億' | '兆')
}
