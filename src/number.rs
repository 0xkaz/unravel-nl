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
    let compact = strip_space_style_grouping(text.trim())?;
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
        if number_format == NumberFormat::DotDecimal {
            // This format declares '.' as the decimal separator, so a number in
            // it carries at most one. `1.234.567` is not a number in the format
            // the caller declared and is refused rather than regrouped into
            // 1234567 by the `Auto` rule below. Mirrors the way `CommaDecimal`
            // refuses `1,234,567` above.
            return None;
        }
        if valid_dot_grouped_number(&compact) {
            return Some(compact.replace('.', ""));
        }
        return None;
    }

    Some(compact)
}

/// The group separators that are not whitespace: the Swiss apostrophe (`1'234`)
/// and the underscore.
///
/// Split out because a scanner has to treat the whitespace ones as spacing
/// while it walks a candidate, and cannot simply accept them as body
/// characters. Everything that needs "is this character grouping digits" reads
/// this or [`SPACE_STYLE_SEPARATORS`] — nothing spells the set out again. The
/// two lists disagreeing is what made `is_numeric_body_char` stop a candidate
/// at the apostrophe while `is_editor_plain_number_candidate` accepted it, so
/// the editor scanned `幅1'234` down to `1` and dropped `'234` in silence.
pub(crate) const NON_SPACE_GROUP_SEPARATORS: [char; 2] = ['_', '\''];

/// Separators that group digits without being a decimal separator anywhere: the
/// space family (`1 200`), the Swiss apostrophe (`1'234`), and the underscore.
pub(crate) const SPACE_STYLE_SEPARATORS: [char; 6] = [
    ' ',
    NON_SPACE_GROUP_SEPARATORS[0],
    NON_SPACE_GROUP_SEPARATORS[1],
    '\u{00A0}',
    '\u{202F}',
    '\u{2009}',
];

/// Whether `ch` groups digits. The one answer, for every caller that asks.
pub(crate) fn is_group_separator(ch: char) -> bool {
    SPACE_STYLE_SEPARATORS.contains(&ch)
}

// An apostrophe is a foot mark (`5'11"`) and a Swiss digit group separator
// (`1'234`). Both readings used to be built and offered as a choice, because
// each entry point had been settling it privately and saying nothing —
// `parse("1'234")` gave 6.2484 m while `parse_number_fast("1'234")` gave 1234,
// neither with a finding.
//
// The choice turned out not to exist. A Swiss group is three digits, so the
// follower is 100 or more, and inches have to stay under the foot above them,
// so it is under 12. No text satisfies both. The apparatus that reported the
// ambiguity is gone with it, rather than left in place describing a fork the
// parser can no longer reach.

/// Removes space-style grouping, but only where it really is grouping.
///
/// These separators used to be stripped unconditionally, so `1.2 3` read as
/// 1.23 and `1 2` as 12: values written nowhere in the input, produced with an
/// empty findings channel. They are now held to the same standard the comma and
/// dot paths hold their separators to, and by the same validators, so the four
/// styles cannot disagree about what counts as a group boundary:
///
/// - a separator may only sit between digits of the whole part, in western
///   three-digit groups (`1 234 567`) or the Indian 2-2-3 shape (`12 34 567`);
/// - a separator may not appear at or after a decimal separator, which is what
///   `1.2 3` does — the comma and dot paths already refuse that, because they
///   only ever check grouping in the whole part and reject a fractional part
///   that is not all digits.
///
/// Anything else returns `None`, which the callers turn into a refusal with a
/// `NoValue` finding, exactly as a malformed comma- or dot-grouped number is
/// refused. No reading is emitted, and no reading is invented.
pub(crate) fn strip_space_style_grouping(text: &str) -> Option<String> {
    if !text.contains(SPACE_STYLE_SEPARATORS) {
        return Some(text.to_owned());
    }

    // Everything from the first `,` or `.` onwards is the decimal separator and
    // what follows it, whichever of the two the format turns out to use. Group
    // separators cannot live there.
    let whole_end = text.find([',', '.']).unwrap_or(text.len());
    let (whole, rest) = text.split_at(whole_end);
    if rest.contains(SPACE_STYLE_SEPARATORS) {
        return None;
    }

    // A sign is not a digit, so a space between it and the number is spacing
    // rather than grouping: `- 1 234` keeps reading as -1234.
    let signless = whole.trim_start_matches(['-', '+']);
    let sign = &whole[..whole.len() - signless.len()];
    let body = if sign.is_empty() {
        signless
    } else {
        signless.trim_start_matches(SPACE_STYLE_SEPARATORS)
    };
    if !body.contains(SPACE_STYLE_SEPARATORS) {
        return Some(format!("{sign}{body}{rest}"));
    }

    // Defer to the validators the comma path uses, on the same string with the
    // separator canonicalized, so "well-formed grouping" has one definition.
    let canonical = body.replace(SPACE_STYLE_SEPARATORS, ",");
    if !valid_grouped_number(&canonical) && !valid_indian_grouped_number(&canonical) {
        return None;
    }
    let grouped = body.replace(SPACE_STYLE_SEPARATORS, "");
    Some(format!("{sign}{grouped}{rest}"))
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

/// The magnitude a `十`/`百`/`千` character multiplies its digit by.
fn cjk_place(ch: char) -> Option<i64> {
    match ch {
        '十' => Some(10),
        '百' => Some(100),
        '千' => Some(1000),
        _ => None,
    }
}

/// The magnitude a `万`/`億`/`兆` character multiplies its whole section by.
fn cjk_myriad(ch: char) -> Option<i64> {
    match ch {
        '万' => Some(10_000),
        '億' => Some(100_000_000),
        '兆' => Some(1_000_000_000_000),
        _ => None,
    }
}

/// One myriad section of a multiplicative numeral, e.g. the `二千三百` of
/// `二千三百万`.
///
/// Strict on purpose. A place character may carry at most one digit, the places
/// must descend (`千` before `百` before `十`), none may repeat, and a zero digit
/// never multiplies a place. That is what refuses `十十`, `百百` and `二四十`,
/// none of which is a numeral, rather than summing them into a value the text
/// does not spell.
fn parse_cjk_section(chars: &[char]) -> Option<i64> {
    if chars.is_empty() {
        return None;
    }
    let mut total = 0_i64;
    let mut pending: Option<i64> = None;
    let mut last_place = i64::MAX;

    for &ch in chars {
        if let Some(digit) = cjk_digit(ch) {
            // Two digits in a row spell a positional number, which cannot be
            // mixed with places; a zero never multiplies a place.
            if pending.is_some() || digit == 0 {
                return None;
            }
            pending = Some(digit);
            continue;
        }
        let place = cjk_place(ch)?;
        if place >= last_place {
            return None;
        }
        last_place = place;
        let digit = pending.take().unwrap_or(1);
        total = total.checked_add(digit.checked_mul(place)?)?;
    }

    if let Some(digit) = pending {
        total = total.checked_add(digit)?;
    }
    Some(total)
}

/// Reads a run of CJK numerals, in either of the two ways they are written.
///
/// Positional (`二〇二四`) spells one digit per place, exactly as `2024` does.
/// Multiplicative (`百二十`) spells magnitudes. The two forms cannot be mixed,
/// and a bare run of digit characters with no `〇` to mark it positional —
/// `二四`, which is 24 to one reader and nothing at all to another — has no
/// single reading, so this declines to state one rather than returning the last
/// digit it happened to see.
pub(crate) fn parse_cjk_number(text: &str) -> Option<i64> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let has_magnitude = chars
        .iter()
        .any(|&ch| cjk_place(ch).is_some() || cjk_myriad(ch).is_some());

    if !has_magnitude {
        let digits: Vec<i64> = chars
            .iter()
            .copied()
            .map(cjk_digit)
            .collect::<Option<_>>()?;
        if digits.len() == 1 {
            return Some(digits[0]);
        }
        // Positional only when the run marks itself as positional.
        if !chars.iter().any(|&ch| matches!(ch, '〇' | '零')) {
            return None;
        }
        let mut value = 0_i64;
        for digit in digits {
            value = value.checked_mul(10)?.checked_add(digit)?;
        }
        return Some(value);
    }

    // Multiplicative form: a zero digit has no place in it.
    if chars.iter().any(|&ch| matches!(ch, '〇' | '零')) {
        return None;
    }

    let mut total = 0_i64;
    let mut last_myriad = i64::MAX;
    let mut start = 0_usize;
    for (index, &ch) in chars.iter().enumerate() {
        let Some(myriad) = cjk_myriad(ch) else {
            continue;
        };
        // `一万万` and `億万億` are not numerals: the myriads must descend.
        if myriad >= last_myriad {
            return None;
        }
        last_myriad = myriad;
        let section = parse_cjk_section(&chars[start..index])?;
        total = total.checked_add(section.checked_mul(myriad)?)?;
        start = index + 1;
    }
    if start < chars.len() {
        total = total.checked_add(parse_cjk_section(&chars[start..])?)?;
    }
    Some(total)
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

// Only the calendar grammars read a bare whole number as an `i64`; the
// recurrence grammar was the other caller, and it is gone. Without
// `dates-jiff` there is nothing left to call this, so the gate keeps a default
// build from carrying a function no configuration of it can reach.
#[cfg(feature = "dates-jiff")]
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

    /// Space-style separators used to be stripped before anything checked they
    /// sat on group boundaries, so `1.2 3` normalized to `1.23` — a value the
    /// input does not spell — while `1.2.3` and `1,2,3` were already refused.
    #[test]
    fn space_style_grouping_is_validated_like_comma_and_dot_grouping() {
        for refused in [
            "1.2 3",
            "1 2",
            "1 2020",
            "1_2",
            "1 2 3",
            "1  200",
            "1 .234",
            "1, 234",
            "1.234 567",
            "1,234 567",
            "1 234,5 6",
        ] {
            assert_eq!(
                normalize_locale_number(refused, NumberFormat::Auto),
                None,
                "{refused}"
            );
        }

        for (input, expected) in [
            ("1 200", "1200"),
            ("1 234 567", "1234567"),
            // The Indian 2-2-3 shape, from the same validator the comma path uses.
            ("12 34 567", "1234567"),
            ("1'234", "1234"),
            ("1_234", "1234"),
            ("1\u{00A0}234", "1234"),
            ("1\u{202F}234", "1234"),
            ("1\u{2009}234", "1234"),
            ("- 1 234", "-1234"),
            ("+1 234", "+1234"),
            ("1 234.56", "1234.56"),
            ("1 234,56", "1234.56"),
            ("3640", "3640"),
        ] {
            assert_eq!(
                normalize_locale_number(input, NumberFormat::Auto).as_deref(),
                Some(expected),
                "{input}"
            );
        }

        // A declared format still rules the decimal separator: the space is
        // grouping in both, and what follows the comma or dot is judged by the
        // format's own rule.
        assert_eq!(
            normalize_locale_number("1 234,56", NumberFormat::CommaDecimal).as_deref(),
            Some("1234.56")
        );
        assert_eq!(
            normalize_locale_number("1 234,56", NumberFormat::DotDecimal),
            None
        );
        assert_eq!(
            normalize_locale_number("1 234.56", NumberFormat::DotDecimal).as_deref(),
            Some("1234.56")
        );
        assert_eq!(
            normalize_locale_number("1 234.56", NumberFormat::CommaDecimal),
            None
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
    fn repeated_myriads_are_refused_rather_than_summed() {
        // A repeated `兆` is not a numeral, so no count of it reads. This used
        // to sum the repeats — which is where the overflow it had to be guarded
        // against came from; the grammar now caps a valid numeral below 10^16
        // and the checked arithmetic stays as a second line of defence.
        assert_eq!(parse_cjk_number("九千兆"), Some(9_000_000_000_000_000));
        assert_eq!(parse_cjk_number(&"九千兆".repeat(2)), None);
        assert_eq!(parse_cjk_number(&"九千兆".repeat(1024)), None);
    }

    #[test]
    fn malformed_cjk_numerals_are_refused_rather_than_composed() {
        for text in ["十十", "百百", "一万万", "二四十", "十百", "〇十"] {
            assert_eq!(parse_cjk_number(text), None, "{text}");
        }
    }

    #[test]
    fn positional_cjk_numerals_read_digit_for_digit() {
        assert_eq!(parse_cjk_number("二〇二四"), Some(2024));
        assert_eq!(parse_cjk_number("一〇"), Some(10));
        assert_eq!(parse_cjk_number("一〇〇"), Some(100));
        assert_eq!(parse_cjk_number("三〇"), Some(30));
        // No `〇` to mark it positional, so it has no single reading.
        assert_eq!(parse_cjk_number("二四"), None);
        assert_eq!(parse_cjk_number("五"), Some(5));
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

    /// `DotDecimal` declares the dot as the decimal separator, so a second dot
    /// is not a number in that format. It used to fall through to the `Auto`
    /// rule and regroup `1.234.567` into 1234567 with no finding, while the
    /// mirroring `CommaDecimal` refused `1,234,567`.
    #[test]
    fn dot_decimal_refuses_multiple_decimal_points_like_comma_decimal() {
        for (number_format, input) in [
            (NumberFormat::DotDecimal, "1.234.567"),
            (NumberFormat::DotDecimal, "1.2.3"),
            (NumberFormat::CommaDecimal, "1,234,567"),
            (NumberFormat::CommaDecimal, "1,2,3"),
        ] {
            let parsed = parse(
                input,
                Some(ParseCtx {
                    number_format,
                    ..ParseCtx::default()
                }),
            );
            assert!(
                parsed.best.is_none(),
                "{number_format:?} {input}: {:?}",
                parsed.best
            );
            assert_eq!(
                parsed.findings.skipped.first().map(|issue| issue.code),
                Some(IssueCode::NoValue),
                "{number_format:?} {input}"
            );
        }

        // The rest of the declared format is untouched: one dot is the decimal
        // separator, and a comma still groups.
        for (input, expected) in [
            ("1.5", 1.5),
            ("1.234", 1.234),
            ("1,234", 1234.0),
            ("1,234,567", 1_234_567.0),
            ("12,34,567", 1_234_567.0),
            ("1,234.56", 1234.56),
        ] {
            let parsed = parse(
                input,
                Some(ParseCtx {
                    number_format: NumberFormat::DotDecimal,
                    ..ParseCtx::default()
                }),
            );
            let best = parsed.best.unwrap_or_else(|| panic!("{input}"));
            assert_close(best.value.expect("value"), expected);
        }

        // `Auto` is unaffected: it still regroups a well-formed dot-grouped
        // number and still refuses a malformed one.
        assert_eq!(
            normalize_locale_number("1.234.567", NumberFormat::Auto).as_deref(),
            Some("1234567")
        );
        assert_eq!(
            normalize_locale_number("1.234.567", NumberFormat::DotDecimal),
            None
        );
        assert_eq!(normalize_locale_number("1.2.3", NumberFormat::Auto), None);
        assert_close(
            parse("1.234.567", None)
                .best
                .expect("auto")
                .value
                .expect("value"),
            1_234_567.0,
        );
    }

    /// The documented `CommaDecimal` asymmetry: the declared format reaches the
    /// quantity grammars through the parsing context, and only a fallback
    /// grammar that never consults it lets `1.5 kg` through.
    #[test]
    fn comma_decimal_applies_to_quantities_except_the_documented_fallback() {
        let comma = |input: &str| {
            parse(
                input,
                Some(ParseCtx {
                    number_format: NumberFormat::CommaDecimal,
                    ..ParseCtx::default()
                }),
            )
        };

        let grouped = comma("1.234 kg").best.expect("1.234 kg");
        assert_close(grouped.value.expect("value"), 1234.0);
        assert_eq!(grouped.unit.as_deref(), Some("kg"));

        for refused in ["$1.5", "1.5 m 2.5 cm"] {
            let parsed = comma(refused);
            assert!(parsed.best.is_none(), "{refused}: {:?}", parsed.best);
            assert_eq!(
                parsed.findings.skipped.first().map(|issue| issue.code),
                Some(IssueCode::NoValue),
                "{refused}"
            );
        }

        let fallback = comma("1.5 kg").best.expect("1.5 kg");
        assert_close(fallback.value.expect("value"), 1.5);
        assert_eq!(fallback.unit.as_deref(), Some("kg"));
    }

    #[test]
    fn parses_grouped_number() {
        let parsed = parse("1,234", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Number);
        assert_close(best.value.unwrap(), 1234.0);
        assert_eq!(parsed.alternatives.len(), 1);
    }

    /// What a digit group separator is has one definition, and everything that
    /// asks — the number parser, the free-text scanner, the editor candidate
    /// filter — asks it.
    ///
    /// They used to disagree. `is_numeric_body_char` stopped a candidate at the
    /// apostrophe while `is_editor_plain_number_candidate` accepted it, so
    /// `parse_dimensions_for_editor("幅1'234")` scanned `1` and dropped `'234`
    /// with nothing in the findings to say so.
    #[test]
    fn one_definition_says_what_groups_digits() {
        for separator in NON_SPACE_GROUP_SEPARATORS {
            assert!(
                SPACE_STYLE_SEPARATORS.contains(&separator),
                "{separator:?} groups digits but the number parser does not strip it"
            );
            assert!(
                is_numeric_body_char(separator),
                "{separator:?} groups digits but the scanner stops a candidate at it"
            );
        }
        for separator in SPACE_STYLE_SEPARATORS {
            assert!(
                is_group_separator(separator),
                "{separator:?} is stripped as grouping but is not a group separator"
            );
            assert!(
                is_editor_plain_number_candidate(&format!("1{separator}234")),
                "{separator:?} groups digits but the editor refuses the candidate"
            );
        }
    }

    /// An apostrophe is a foot mark and a Swiss group separator, and every
    /// entry point that reads `1'234` says so rather than picking one.
    ///
    /// `parse` used to answer 6.2484 m and `parse_number_fast` 1234, both with
    /// an empty findings channel — a 5000-fold disagreement no caller could
    /// detect. The two readings are now the same pair everywhere; only which
    /// one is ranked first differs, and that is reported.
    #[test]
    fn an_apostrophe_is_read_by_one_definition_at_every_entry_point() {
        let readings_of = |parsed: &Parsed| {
            let mut values: Vec<String> = parsed
                .best
                .iter()
                .chain(parsed.alternatives.iter())
                .map(|reading| format!("{:?}{:?}", reading.value, reading.unit))
                .collect();
            values.sort();
            values
        };

        // The two grammars cannot both accept the same text. A Swiss group is
        // three digits, and inches have to stay under the foot above them, so
        // the follower is either at least 100 or under 12 and never both. What
        // has to hold is that every entry point lands on the same side of that
        // line, and that a single-reading text is not dressed up as a choice.
        for input in ["1'234", "12'345", "1'11", "5'11\"", "1'2", "1'0", "1'13"] {
            let entry_points: Vec<(&str, Parsed)> = vec![
                ("parse", parse(input, None)),
                ("parse_quantity_fast", parse_quantity_fast(input, None)),
                ("parse_number_fast", parse_number_fast(input, None)),
            ];

            let expected = readings_of(&entry_points[0].1);
            for (name, parsed) in &entry_points {
                // A narrow entry declining the text is the caller's own
                // declaration — `parse_quantity_fast` has no business reading a
                // bare 1234. Reading it *differently* is the failure.
                let reading = readings_of(parsed);
                assert!(
                    reading.is_empty() || reading == expected,
                    "{name} reads {input:?} as {reading:?}, not {expected:?}"
                );
                // The two grammars are disjoint, so an apostrophe never leaves
                // a fork behind. If one ever does, the reading below stops
                // being alone and this says so rather than letting an entry
                // point pick quietly, which is how the two spellings drifted
                // apart in the first place.
                assert!(
                    reading.len() <= 1,
                    "{name} found {} readings for {input:?}: {reading:?}",
                    reading.len()
                );
            }
        }

        // The editor scanner reads the whole number rather than stopping at the
        // apostrophe, and lands on the same reading as every other entry point.
        let matches = parse_dimensions_for_editor("幅1'234", None);
        assert_eq!(matches.len(), 1);
        assert_close(
            matches[0]
                .parsed
                .best
                .as_ref()
                .expect("a reading")
                .value
                .unwrap(),
            1234.0,
        );
        // The editor offers `1234 mm` for a bare number, which is its own
        // suggestion and not a second reading of the apostrophe. What must not
        // be there is the foot reading, which canonicalises to metres.
        assert!(
            matches[0]
                .parsed
                .alternatives
                .iter()
                .all(|reading| reading.unit.as_deref() != Some("m")),
            "the editor kept a foot reading of an apostrophe it read as digits"
        );

        // A text only one of the two grammars accepts is not ambiguous, and is
        // still read: `5'11"` is feet and inches and nothing else.
        let feet = parse("5'11\"", None);
        assert_close(feet.best.expect("feet and inches").value.unwrap(), 1.8034);
        assert!(feet.findings.ambiguities.is_empty());
    }
}
