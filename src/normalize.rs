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
        push_normalized_char(&mut normalized, ch);
    }
    normalized
}

/// Appends the normalized form of `ch` to `normalized`.
///
/// Some rules depend on what has already been written (the numeric-separator
/// comma), so the whole prefix is passed in rather than the character alone.
/// This is the single definition of the normalization mapping: both
/// [`normalize_input`] and [`OriginalOffsets`] drive it, so the offset table can
/// never drift from the text it describes.
pub(crate) fn push_normalized_char(normalized: &mut String, ch: char) {
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
        '，' | '、' if looks_numeric_separator(normalized) => normalized.push(','),
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

/// Translates byte offsets in the normalized, trimmed text back to the original.
///
/// Grammar dispatch runs on `normalize_input_cow(text).trim()`, but every span
/// handed back to a caller has to address [`Parsed::input`], which is the
/// untouched original. Normalization changes byte lengths (`'０'` 3→1, `'㎞'`
/// 3→2, a zero-width space 3→0), so without this translation a span is at best
/// shifted and at worst lands inside a multi-byte character.
///
/// The common case — input that needs no rewriting — is represented as a plain
/// shift and allocates nothing.
pub(crate) enum OriginalOffsets {
    /// Normalization was a no-op; only `trim` moved the offsets.
    Shifted {
        /// Bytes `trim_start` removed from the front of the original.
        shift: usize,
    },
    /// Normalization rewrote the input, so offsets need a per-byte table.
    Mapped {
        /// Original offset each normalized byte offset starts at.
        starts: Vec<usize>,
        /// Original offset each normalized byte offset ends at.
        ends: Vec<usize>,
        /// Bytes `trim_start` removed from the front of the normalized text.
        base: usize,
    },
}

impl OriginalOffsets {
    /// Builds the translation table for `original`.
    ///
    /// Costs one scan and no allocation when `original` needs no normalization,
    /// which is the overwhelmingly common case.
    pub(crate) fn for_input(original: &str) -> Self {
        if !original.chars().any(needs_input_normalization) {
            return Self::Shifted {
                shift: original.len() - original.trim_start().len(),
            };
        }

        let mut normalized = String::with_capacity(original.len());
        let mut starts = Vec::with_capacity(original.len() + 1);
        let mut ends = Vec::with_capacity(original.len() + 1);
        // Highest original offset already covered by an emitted byte. An end
        // offset that lands on a character boundary stops here, so characters
        // that normalize away (zero-width spaces) fall outside the span instead
        // of being swept into it.
        let mut covered = 0usize;

        for (offset, ch) in original.char_indices() {
            let before = normalized.len();
            push_normalized_char(&mut normalized, ch);
            let emitted = normalized.len() - before;
            if emitted == 0 {
                continue;
            }
            let char_end = offset + ch.len_utf8();
            starts.push(offset);
            ends.push(covered);
            // Interior bytes of a character that expanded (`'㎞'` to `"km"`) have
            // no original offset of their own: a start rounds down to the whole
            // character, an end rounds up, so the span always covers it whole.
            for _ in 1..emitted {
                starts.push(offset);
                ends.push(char_end);
            }
            covered = char_end;
        }

        starts.push(original.len());
        ends.push(covered);

        Self::Mapped {
            starts,
            ends,
            base: normalized.len() - normalized.trim_start().len(),
        }
    }

    /// Translates a span start offset in the trimmed text to the original.
    pub(crate) fn start(&self, offset: usize) -> usize {
        match self {
            Self::Shifted { shift } => offset + shift,
            Self::Mapped { starts, base, .. } => lookup(starts, base + offset),
        }
    }

    /// Translates a span end offset in the trimmed text to the original.
    pub(crate) fn end(&self, offset: usize) -> usize {
        match self {
            Self::Shifted { shift } => offset + shift,
            Self::Mapped { ends, base, .. } => lookup(ends, base + offset),
        }
    }
}

fn lookup(table: &[usize], index: usize) -> usize {
    table
        .get(index)
        .copied()
        .or_else(|| table.last().copied())
        .unwrap_or(0)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Translates a normalized-and-trimmed span the way the finding pass does.
    fn round_trip(original: &str, start: usize, end: usize) -> (usize, usize) {
        let offsets = OriginalOffsets::for_input(original);
        (offsets.start(start), offsets.end(end))
    }

    #[test]
    fn input_needing_no_normalization_maps_by_a_plain_shift() {
        for (original, shift) in [("5 kg", 0), ("  5 kg", 2), ("\t\n5 kg", 2), ("", 0)] {
            let offsets = OriginalOffsets::for_input(original);
            assert!(
                matches!(offsets, OriginalOffsets::Shifted { .. }),
                "{original:?} should take the allocation-free path"
            );
            assert_eq!(offsets.start(0), shift, "{original:?}");
            assert_eq!(offsets.end(3), shift + 3, "{original:?}");
            // The fast path in `normalize_input_cow` is what makes it free.
            assert!(matches!(normalize_input_cow(original), Cow::Borrowed(_)));
        }
    }

    #[test]
    fn collapsing_characters_map_back_to_their_full_width() {
        // "３pm Europe/Paris" normalizes to "3pm Europe/Paris": the tail slides
        // two bytes left, so 4..16 in normalized space is 6..18 in the original.
        assert_eq!(round_trip("３pm Europe/Paris", 4, 16), (6, 18));
        // Six three-byte letters collapsing to one byte each.
        assert_eq!(round_trip("５ ｍｅｔｅｒｚ", 2, 8), (4, 22));
        assert_eq!(round_trip("１,２３４", 0, 5), (0, 13));
    }

    #[test]
    fn expanding_characters_cover_the_whole_source_character() {
        // "5㎞x" normalizes to "5kmx": normalized 1..3 is the two bytes "km",
        // which came from one three-byte character.
        assert_eq!(round_trip("5㎞x", 1, 3), (1, 4));
        // A boundary inside the expansion has no original offset of its own, so
        // a start rounds down and an end rounds up to keep the character whole.
        assert_eq!(round_trip("5㎞x", 2, 2), (1, 4));
        assert_eq!(round_trip("5㎞x", 0, 4), (0, 5));
    }

    #[test]
    fn vanishing_characters_fall_outside_the_span() {
        // "5\u{200b}kg" normalizes to "5kg". The span for "5" must not swallow
        // the zero-width space that follows it, and the span for "kg" must skip
        // past it rather than starting inside it.
        assert_eq!(round_trip("5\u{200b}kg", 0, 1), (0, 1));
        assert_eq!(round_trip("5\u{200b}kg", 1, 3), (4, 6));
        // A trailing zero-width space is not part of the trimmed text either.
        assert_eq!(round_trip("5kg\u{200b}", 0, 3), (0, 3));
    }

    #[test]
    fn trimming_is_undone_together_with_normalization() {
        // Leading whitespace and a no-break space that only becomes whitespace
        // after normalization both have to be stepped over.
        assert_eq!(round_trip(" \u{00a0}５m", 0, 2), (3, 7));
        assert_eq!(round_trip("　５m", 0, 2), (3, 7));
        assert_eq!(round_trip("５m  ", 0, 2), (0, 4));
    }

    #[test]
    fn offset_table_never_disagrees_with_the_normalized_text() {
        for original in [
            "３pm Europe/Paris",
            "５ ｍｅｔｅｒｚ",
            "１,２３４",
            "5㎞ 3 cups",
            "  ５㎏  ",
            "１０　㎞",
            "\u{feff}１０００円",
            "1\u{200b}2\u{200b}3",
            "e\u{0301}5 ㎜",
            "㌔㌢㍉㍍㌘㏄",
        ] {
            let normalized = normalize_input(original);
            let trimmed = normalized.trim();
            let offsets = OriginalOffsets::for_input(original);

            for offset in 0..=trimmed.len() {
                if !trimmed.is_char_boundary(offset) {
                    continue;
                }
                let start = offsets.start(offset);
                let end = offsets.end(offset);
                assert!(
                    original.is_char_boundary(start) && original.is_char_boundary(end),
                    "{original:?}: offset {offset} maps off a char boundary"
                );
                assert!(
                    start <= original.len() && end <= original.len(),
                    "{original:?}"
                );
            }

            // The whole trimmed text maps back onto a fragment of the original
            // that normalizes to exactly that text. It need not be
            // `original.trim()`: a leading byte-order mark is not whitespace but
            // does vanish under normalization, so it falls outside the span.
            let start = offsets.start(0);
            let end = offsets.end(trimmed.len());
            assert_eq!(
                normalize_input(&original[start..end]).trim(),
                trimmed,
                "{original:?}"
            );
        }
    }
}
