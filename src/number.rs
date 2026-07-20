use crate::*;

pub(crate) fn parse_plain_number(text: &str) -> Option<Reading> {
    parse_number(text).map(|value| Reading::number(value, 0.99))
}

pub(crate) fn parse_plain_number_ctx(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    parse_number_ctx(text, ctx).map(|value| Reading::number(value, 0.99))
}

pub(crate) fn parse_number(text: &str) -> Option<f64> {
    parse_number_with_format(text, NumberFormat::Auto)
}

pub(crate) fn parse_number_ctx(text: &str, ctx: &ParseCtx) -> Option<f64> {
    parse_number_with_format(text, ctx.number_format)
}

pub(crate) fn parse_number_with_format(text: &str, number_format: NumberFormat) -> Option<f64> {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(value) = parse_japanese_large_number(trimmed) {
        return Some(value);
    }

    if let Some(value) = parse_unicode_fraction_number(trimmed) {
        return Some(value);
    }

    if let Some(value) = parse_english_number_words(trimmed) {
        return Some(value as f64);
    }

    if let Some(value) = parse_cjk_number(trimmed) {
        return Some(value as f64);
    }

    let normalized = normalize_locale_number(trimmed, number_format)?;

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+'))
    {
        normalized.parse::<f64>().ok()
    } else {
        None
    }
}

pub(crate) fn normalize_locale_number(text: &str, number_format: NumberFormat) -> Option<String> {
    let compact = text
        .chars()
        .filter(|ch| !matches!(ch, ' ' | '_' | '\'' | '\u{00A0}' | '\u{202F}' | '\u{2009}'))
        .collect::<String>();
    if compact.is_empty() {
        return None;
    }

    if compact.contains(',') && compact.contains('.') {
        let (decimal, grouping) = match number_format {
            NumberFormat::CommaDecimal => (',', '.'),
            NumberFormat::DotDecimal => ('.', ','),
            NumberFormat::Auto => {
                let comma = compact.rfind(',')?;
                let dot = compact.rfind('.')?;
                if comma > dot { (',', '.') } else { ('.', ',') }
            }
        };
        return normalize_decimal_grouped_number(&compact, decimal, grouping);
    }

    if compact.contains(',') {
        if number_format == NumberFormat::CommaDecimal {
            return Some(compact.replace(',', "."));
        }
        if number_format == NumberFormat::DotDecimal {
            // The comma can only group digits in this format, so it has to be
            // grouped the way a group separator is: 3-digit groups, or the
            // Indian 2-2-3 shape. Anything else is not a number in the format
            // the caller declared, and is refused rather than regrouped.
            if valid_grouped_number(&compact) || valid_indian_grouped_number(&compact) {
                return normalize_grouped_decimal_free_number(&compact, ',');
            }
            return None;
        }
        if valid_grouped_number(&compact) || valid_indian_grouped_number(&compact) {
            return Some(compact.replace(',', ""));
        }
        if compact.matches(',').count() == 1 {
            return Some(compact.replace(',', "."));
        }
        return None;
    }

    if compact.contains('.') && number_format == NumberFormat::CommaDecimal {
        // This format declares ',' as the decimal separator, so a dot can only
        // group digits — including when it appears exactly once. It must then
        // be grouped like a group separator; `1.5` and `1.2.3` are not numbers
        // in this format and are refused rather than silently regrouped into
        // 15 and 123. Mirrors the `DotDecimal` handling of a comma above.
        if valid_dot_grouped_number(&compact) {
            return normalize_grouped_decimal_free_number(&compact, '.');
        }
        return None;
    }

    if compact.matches('.').count() > 1 {
        if valid_dot_grouped_number(&compact) {
            return Some(compact.replace('.', ""));
        }
        return None;
    }

    Some(compact)
}

pub(crate) fn normalize_grouped_decimal_free_number(text: &str, grouping: char) -> Option<String> {
    let ungrouped = text.replace(grouping, "");
    if ungrouped
        .trim_start_matches(['-', '+'])
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.')
    {
        Some(ungrouped)
    } else {
        None
    }
}

pub(crate) fn normalize_decimal_grouped_number(
    text: &str,
    decimal: char,
    grouping: char,
) -> Option<String> {
    let (whole, fraction) = text.rsplit_once(decimal)?;
    if fraction.is_empty() || !fraction.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    // Grouping in the whole part must actually be grouping. Without this,
    // `1.2.3,45` was read as `123.45`: separators were stripped without ever
    // checking that they sat on group boundaries, unlike the decimal-free
    // paths, which reject `1.23.4` outright.
    if whole.contains(grouping) && !valid_grouping_widths(whole, grouping) {
        return None;
    }
    let whole_without_groups = whole.replace(grouping, "");
    if whole_without_groups.is_empty()
        || !whole_without_groups
            .trim_start_matches(['-', '+'])
            .chars()
            .all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    Some(format!("{whole_without_groups}.{fraction}"))
}

/// Checks the whole part of a decimal number for well-formed digit grouping.
///
/// Defers to the same validators the decimal-free paths use, so the two agree:
/// western three-digit groups either way, plus Indian two-digit grouping when
/// the separator is a comma (`12,34,567.89`).
pub(crate) fn valid_grouping_widths(whole: &str, grouping: char) -> bool {
    if grouping == ',' {
        valid_grouped_number(whole) || valid_indian_grouped_number(whole)
    } else {
        valid_dot_grouped_number(whole)
    }
}

pub(crate) fn parse_japanese_large_number(text: &str) -> Option<f64> {
    if !text.contains(['万', '億', '兆']) {
        return None;
    }
    let mut total = 0.0;
    let mut rest = text.trim();
    for (unit, factor) in [
        ('兆', 1_000_000_000_000.0),
        ('億', 100_000_000.0),
        ('万', 10_000.0),
    ] {
        if let Some((head, tail)) = rest.split_once(unit) {
            let value = if head.trim().is_empty() {
                1.0
            } else {
                parse_number_without_large_units(head.trim())?
            };
            total += value * factor;
            rest = tail;
        }
    }
    if !rest.trim().is_empty() {
        total += parse_number_without_large_units(rest.trim())?;
    }
    Some(total)
}

pub(crate) fn parse_number_without_large_units(text: &str) -> Option<f64> {
    if let Some(value) = parse_cjk_number(text) {
        return Some(value as f64);
    }
    normalize_locale_number(text, NumberFormat::Auto)?
        .parse::<f64>()
        .ok()
}

pub(crate) fn parse_unicode_fraction_number(text: &str) -> Option<f64> {
    let mut chars = text.chars();
    let fraction = chars.next_back().and_then(fraction_char_value)?;
    let whole_text = chars.as_str().trim();
    if whole_text.is_empty() {
        return Some(fraction);
    }
    if whole_text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(whole_text.parse::<f64>().ok()? + fraction);
    }
    None
}

pub(crate) fn fraction_char_value(ch: char) -> Option<f64> {
    match ch {
        '¼' => Some(0.25),
        '½' => Some(0.5),
        '¾' => Some(0.75),
        '⅓' => Some(1.0 / 3.0),
        '⅔' => Some(2.0 / 3.0),
        '⅛' => Some(0.125),
        '⅜' => Some(0.375),
        '⅝' => Some(0.625),
        '⅞' => Some(0.875),
        _ => None,
    }
}

pub(crate) fn parse_english_number_words(text: &str) -> Option<i64> {
    let normalized = text
        .to_ascii_lowercase()
        .replace('-', " ")
        .replace(" and ", " ");
    let mut total = 0_i64;
    let mut current = 0_i64;
    let mut saw_word = false;

    for word in normalized.split_whitespace() {
        if word == "a" || word == "an" {
            current = current.checked_add(1)?;
            saw_word = true;
            continue;
        }
        if let Some(value) = small_number_word(word) {
            current = current.checked_add(value)?;
            saw_word = true;
            continue;
        }
        if word == "hundred" {
            current = current.checked_mul(100)?;
            saw_word = true;
            continue;
        }
        if word == "thousand" {
            total = total.checked_add(current.checked_mul(1000)?)?;
            current = 0;
            saw_word = true;
            continue;
        }
        return None;
    }

    if !saw_word {
        return None;
    }
    total.checked_add(current)
}

pub(crate) fn small_number_word(word: &str) -> Option<i64> {
    match word {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "thirty" => Some(30),
        "forty" => Some(40),
        "fifty" => Some(50),
        "sixty" => Some(60),
        "seventy" => Some(70),
        "eighty" => Some(80),
        "ninety" => Some(90),
        _ => None,
    }
}

pub(crate) fn parse_cjk_number(text: &str) -> Option<i64> {
    let mut total = 0_i64;
    let mut section = 0_i64;
    let mut number = 0_i64;
    let mut saw = false;

    for ch in text.chars() {
        if let Some(value) = cjk_digit(ch) {
            number = value;
            saw = true;
            continue;
        }
        let unit = match ch {
            '十' => 10,
            '百' => 100,
            '千' => 1000,
            '万' => {
                section = section.checked_add(number)?;
                total = total.checked_add(section.checked_mul(10_000)?)?;
                section = 0;
                number = 0;
                saw = true;
                continue;
            }
            '億' => {
                section = section.checked_add(number)?;
                total = total.checked_add(section.checked_mul(100_000_000)?)?;
                section = 0;
                number = 0;
                saw = true;
                continue;
            }
            '兆' => {
                section = section.checked_add(number)?;
                total = total.checked_add(section.checked_mul(1_000_000_000_000)?)?;
                section = 0;
                number = 0;
                saw = true;
                continue;
            }
            _ => return None,
        };
        let addend = if number == 0 {
            unit
        } else {
            number.checked_mul(unit)?
        };
        section = section.checked_add(addend)?;
        number = 0;
        saw = true;
    }

    if !saw {
        return None;
    }
    total.checked_add(section)?.checked_add(number)
}

pub(crate) fn cjk_digit(ch: char) -> Option<i64> {
    match ch {
        '零' | '〇' => Some(0),
        '一' | '壱' => Some(1),
        '二' | '弐' => Some(2),
        '三' | '参' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

pub(crate) fn parse_whole_i64(text: &str) -> Option<i64> {
    if text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

pub(crate) fn valid_grouped_number(text: &str) -> bool {
    let (whole, decimal) = text.split_once('.').unwrap_or((text, ""));
    if !decimal.is_empty() && !decimal.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    let signless = whole.trim_start_matches(['-', '+']);
    let groups: Vec<&str> = signless.split(',').collect();
    if groups.len() <= 1 || groups[0].is_empty() || groups[0].len() > 3 {
        return false;
    }
    groups.iter().enumerate().all(|(idx, group)| {
        group.chars().all(|ch| ch.is_ascii_digit()) && (idx == 0 || group.len() == 3)
    })
}

pub(crate) fn valid_dot_grouped_number(text: &str) -> bool {
    let signless = text.trim_start_matches(['-', '+']);
    let groups: Vec<&str> = signless.split('.').collect();
    groups.len() > 1
        && !groups[0].is_empty()
        && groups[0].len() <= 3
        && groups.iter().enumerate().all(|(idx, group)| {
            group.chars().all(|ch| ch.is_ascii_digit()) && (idx == 0 || group.len() == 3)
        })
}

pub(crate) fn valid_indian_grouped_number(text: &str) -> bool {
    let (whole, decimal) = text.split_once('.').unwrap_or((text, ""));
    if !decimal.is_empty() && !decimal.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    let signless = whole.trim_start_matches(['-', '+']);
    let groups: Vec<&str> = signless.split(',').collect();
    if groups.len() < 3 || groups[0].is_empty() || groups[0].len() > 2 {
        return false;
    }
    let last_is_three = groups
        .last()
        .is_some_and(|group| group.len() == 3 && group.chars().all(|ch| ch.is_ascii_digit()));
    let middle_are_two = groups[1..groups.len() - 1]
        .iter()
        .all(|group| group.len() == 2 && group.chars().all(|ch| ch.is_ascii_digit()));
    groups[0].chars().all(|ch| ch.is_ascii_digit()) && middle_are_two && last_is_three
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn surfaces_ambiguous_grouped_decimal_number() {
        let parsed = parse("1,234", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Number);
        assert_close(best.value.unwrap(), 1234.0);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_close(parsed.alternatives[0].value.unwrap(), 1.234);
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "1,234");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousNumber
        );
        assert_eq!(parsed.findings.ambiguities[0].span.start, 0);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 5);
    }

    #[test]
    fn surfaces_ambiguous_dot_number_symmetrically_with_comma() {
        let parsed = parse("1.234", None);
        let best = parsed.best.expect("best reading");
        assert_close(best.value.unwrap(), 1.234);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_close(parsed.alternatives[0].value.unwrap(), 1234.0);
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousNumber
        );
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "1.234");
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(2));

        // An explicit format is the caller resolving the ambiguity, so no
        // alternative and no finding — the comma path already behaves this way.
        for number_format in [NumberFormat::DotDecimal, NumberFormat::CommaDecimal] {
            for input in ["1.234", "1,234"] {
                let parsed = parse(
                    input,
                    Some(ParseCtx {
                        number_format,
                        ..ParseCtx::default()
                    }),
                );
                assert!(parsed.alternatives.is_empty(), "{input} {number_format:?}");
                assert!(
                    parsed.findings.ambiguities.is_empty(),
                    "{input} {number_format:?}"
                );
            }
        }

        // Only a genuine grouping shape is ambiguous.
        for unambiguous in ["1.23", "3.5", "0.5", "1234.567"] {
            let parsed = parse(unambiguous, None);
            assert!(
                parsed.findings.ambiguities.is_empty(),
                "{unambiguous}: {:?}",
                parsed.findings.ambiguities
            );
        }
    }

    #[test]
    fn rejects_malformed_grouping_in_decimal_numbers() {
        // `1.2.3` is not valid dot grouping, so `1.2.3,45` is not a number —
        // it used to be silently read as 123.45.
        for input in ["1.2.3,45", "1.23.4,5", "1,2,3.45"] {
            let parsed = parse(input, None);
            assert!(parsed.best.is_none(), "{input}: {:?}", parsed.best);
            assert!(!parsed.findings.skipped.is_empty(), "{input}");
        }

        assert_eq!(normalize_decimal_grouped_number("1.2.3,45", ',', '.'), None);
        // Well-formed grouping still normalizes, western and Indian alike.
        assert_eq!(
            normalize_decimal_grouped_number("1.234.567,89", ',', '.').as_deref(),
            Some("1234567.89")
        );
        assert_eq!(
            normalize_decimal_grouped_number("12,34,567.89", '.', ',').as_deref(),
            Some("1234567.89")
        );
        assert_eq!(
            normalize_decimal_grouped_number("1234,56", ',', '.').as_deref(),
            Some("1234.56")
        );
    }

    #[test]
    fn parses_number_words_and_unicode_fractions() {
        let words = parse("twenty-five kg", None).best.expect("words");
        assert_eq!(words.unit.as_deref(), Some("kg"));
        assert_close(words.value.unwrap(), 25.0);

        let fraction = parse("1½ cups", None).best.expect("fraction");
        assert_eq!(fraction.unit.as_deref(), Some("L"));
        assert_close(fraction.value.unwrap(), 1.5 * US_CUP_L);
    }

    #[test]
    fn parses_cjk_number_mass() {
        let parsed = parse(
            "三十五公斤",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("kg"));
        assert_close(best.value.unwrap(), 35.0);
    }

    #[test]
    fn cjk_number_overflow_returns_none_instead_of_panicking() {
        let overflowing = "九千兆".repeat(1026);
        assert_eq!(parse_cjk_number(&overflowing), None);
        assert!(parse(&overflowing, None).best.is_none());
    }

    #[test]
    fn cjk_number_just_below_overflow_still_parses() {
        // Each `九千兆` adds 9_000 * 10^12; 1024 repeats is the largest count
        // that fits in i64 (1025 overflows), so the fix must not reject 1024.
        assert_eq!(
            parse_cjk_number(&"九千兆".repeat(1024)),
            Some(9_216_000_000_000_000_000)
        );
        assert_eq!(parse_cjk_number(&"九千兆".repeat(1025)), None);
    }

    #[test]
    fn cjk_number_ordinary_inputs_still_parse() {
        assert_eq!(parse_cjk_number("九千"), Some(9000));
        assert_eq!(parse_cjk_number("三十五"), Some(35));
        assert_eq!(parse_cjk_number("一億二千万"), Some(120_000_000));
    }

    #[test]
    fn english_number_words_overflow_returns_none_instead_of_panicking() {
        let text = "ten hundred hundred hundred hundred hundred hundred hundred hundred hundred";
        assert_eq!(parse_english_number_words(text), None);
        assert!(parse(text, None).best.is_none());
    }

    #[test]
    fn english_number_words_just_below_overflow_still_parses() {
        // 10 * 100^8 = 1e17, still inside i64.
        let text = "ten hundred hundred hundred hundred hundred hundred hundred hundred";
        assert_eq!(
            parse_english_number_words(text),
            Some(100_000_000_000_000_000)
        );
    }

    #[test]
    fn english_number_words_ordinary_inputs_still_parse() {
        assert_eq!(parse_english_number_words("ten hundred"), Some(1000));
        assert_eq!(parse_english_number_words("two hundred"), Some(200));
        assert_eq!(parse_english_number_words("twenty-five"), Some(25));
        assert_eq!(parse_english_number_words("three thousand"), Some(3000));
    }

    #[test]
    fn parses_grouped_number() {
        let parsed = parse("1,234", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Number);
        assert_close(best.value.unwrap(), 1234.0);
        assert_eq!(parsed.alternatives.len(), 1);
    }
}
