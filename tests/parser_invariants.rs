//! The two properties the sentence scanner used to violate, stated for the
//! entry points that survive it.
//!
//! These were pinned only through the removed multi-value scanner, whose own
//! tests went with it. They are not properties of scanning: they are properties
//! of reading a value out of a string, so they hold for [`parse`], for every
//! narrow entry point, and for every match the editor extractor returns.
//!
//! - **A readable value is never silently discarded.** A result with no reading
//!   must say why. `parse_all("I bought 3 and 4 apples")` used to return `[]`,
//!   which said nothing at all.
//! - **Nothing is fabricated.** A reading has to come from the text: no range
//!   without a word or a mark that states one, no digits the input does not
//!   have, no unit invented for a string of bare digits. A dropped value is
//!   detectable downstream, an invented one is not.

use unravel_nl::{
    Findings, Kind, Locale, ParseCtx, Parsed, Reading, parse, parse_date_fast,
    parse_dimensions_for_editor, parse_number_fast, parse_quantity_fast, parse_recurrence_fast,
};

/// Inputs the removed scanner tests used, plus the shapes that broke it, plus a
/// generated sweep of label × separator × number × unit.
fn corpus() -> Vec<String> {
    let mut inputs: Vec<String> = [
        // The exact strings the deleted scanner tests read.
        "I bought 3 and 4 apples",
        "1 and 2",
        "1 and",
        "1 and2 kg",
        "10 and 20 and 30",
        "rooms 2 and 3",
        "1, 2 and 3",
        "4 apples",
        "1.234 apples",
        "2 apples 3 oranges",
        "2 apples3 oranges",
        "3 apples4",
        "I bought 3 and4 apples",
        "1 in 2",
        "add 1 to 2",
        "between 5 and 10 kg",
        "1--2",
        "10-20-30",
        "version 1-2-3",
        "tel 03-1234-5678",
        "10-20",
        "100-120㎡",
        "5-10kg",
        "3pm-4pm",
        "10 ± 0.5 mm",
        "2〜3日",
        "1,234",
        "area 100-120㎡ and height 3.5m",
        "工期2〜3日、面積100-120㎡",
        "延床100㎡、敷地面積120㎡、高さ3.5m",
        "延床100㎡、敷地面積120㎡、高さ3.5m、予算¥1,234",
        "3m 3m、約3m",
        "about 3m",
        "ABOUT 3m",
        "about 3m and 4m",
        "mebers 3m",
        "tsbo 6帖",
        "幅１．５ｍ；重量五キログラム；面積百二十平米",
        "6帖 / 4畳半",
        "約3m",
        "3m×4m のLDK",
        "壁厚105mm",
        "高さ2.9m",
        "寸法3640",
        "寸法１ ２００",
        "1.234 kg of flour",
        "the tank holds 5 m3",
        "ship 2 boxes at 5 kg each by friday",
        "convert 72 in to cm and keep pressure under 10 inH₂O",
        // Shapes with numbers next to text that is not a unit.
        "1.2 3-4 kg",
        "1.2 3",
        "3 smoots",
        "5 W",
        "3640",
        "0",
        "",
        "   ",
        "hello",
        // Positional CJK numerals: a place-value spelling, not a sum of the
        // characters. These used to be exempt from every digit check.
        "二〇二四",
        "一〇",
        "一〇〇",
        "二〇二四年",
        "幅二〇二四",
        "三〇",
        // The same unit repeated, and a compound written in ascending order.
        // Neither states a sum, so a sum is a guess.
        "3 m 5 m",
        "5 kg 3 kg",
        "3 cm 5 m",
        "3 mm 5 mm",
        // Descending compounds, which do state one. These must keep reading.
        "5 m 3 cm",
        "2 lb 3 oz",
        "5尺3寸",
        "1h30",
        "1 23 456",
        "1'234",
        "5 €",
    ]
    .iter()
    .map(|text| (*text).to_owned())
    .collect();

    let labels = [
        "",
        "幅",
        "壁厚",
        "面積",
        "寸法",
        "予算",
        "width",
        "wall thickness",
        "room w",
        "beamA",
    ];
    let separators = ["", " ", ":", "：", "-", "×", "、"];
    let numbers = [
        "3",
        "3640",
        "-5",
        "3.5",
        "1,234",
        "１．５",
        "百二十",
        "1 200",
    ];
    let units = ["", "m", "mm", "㎡", "帖", "kg", "W", "in", "and", "apples"];
    let tails = ["", "×4m", "-20", " and 4"];

    for label in labels {
        for separator in separators {
            for number in numbers {
                for unit in units {
                    for tail in tails {
                        inputs.push(format!("{label}{separator}{number}{unit}{tail}"));
                    }
                }
            }
        }
    }
    inputs
}

fn contexts() -> Vec<Option<ParseCtx>> {
    vec![
        None,
        Some(ParseCtx::default()),
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
        Some(ParseCtx {
            locale: Some(Locale::En),
            ..ParseCtx::default()
        }),
    ]
}

fn silent(findings: &Findings) -> bool {
    findings.skipped.is_empty()
        && findings.ambiguities.is_empty()
        && findings.approximations.is_empty()
}

/// No entry point returns "nothing" without saying why.
///
/// This is the single-value form of the property the removed scanner broke
/// worst: it answered `[]` — no reading and no channel to carry a reason — for
/// input it could in fact read. A `Parsed` has a findings channel, so the same
/// silence is impossible only as long as something checks that it is used.
#[test]
fn a_result_with_no_reading_always_says_why() {
    for input in corpus() {
        for ctx in contexts() {
            let named: [(&str, Parsed); 5] = [
                ("parse", parse(&input, ctx.clone())),
                (
                    "parse_quantity_fast",
                    parse_quantity_fast(&input, ctx.clone()),
                ),
                ("parse_number_fast", parse_number_fast(&input, ctx.clone())),
                ("parse_date_fast", parse_date_fast(&input, ctx.clone())),
                (
                    "parse_recurrence_fast",
                    parse_recurrence_fast(&input, ctx.clone()),
                ),
            ];
            for (entry, parsed) in named {
                assert!(
                    parsed.best.is_some() || !silent(&parsed.findings),
                    "{entry}({input:?}) returned no reading and no finding"
                );
            }

            for found in parse_dimensions_for_editor(&input, ctx.clone()) {
                assert!(
                    found.parsed.best.is_some() || !silent(&found.parsed.findings),
                    "editor match {:?} of {input:?} has no reading and no finding",
                    found.text
                );
            }
        }
    }
}

/// Words and marks that state a range. A range reading needs one of these in
/// the input, because a range the text does not state is a range the parser
/// made up.
const RANGE_TOKENS: &[&str] = &[
    "-", "〜", "～", "±", "..", "+/-", " to ", "から", "and", "以下", "以上", "未満", "超",
    "a few", "under", "over", "below", "above", "up to", "at least", "between", "from", "約",
    "several", "couple", "many", "some",
];

/// Full-width forms fold to ASCII, so `ｆｒｏｍ １０ｋｇ ｔｏ ２ｋｇ` is checked
/// against the same token list its ASCII spelling is.
fn folded(text: &str) -> String {
    text.chars()
        .map(|ch| {
            let code = ch as u32;
            if (0xff01..=0xff5e).contains(&code) {
                char::from_u32(code - 0xfee0).unwrap_or(ch)
            } else {
                ch
            }
        })
        .collect::<String>()
        .to_lowercase()
}

fn ascii_digits(text: &str) -> String {
    folded(text).chars().filter(char::is_ascii_digit).collect()
}

/// Whether `fraction` is written as the digits following a decimal mark.
///
/// A fractional part is only ever spelled one way: right after the mark, with
/// nothing in between. `1.2 3` reading as 1.23 is exactly the shape this
/// rejects — the input writes `2` after the point, not `23`. Trailing input
/// digits the value does not carry are allowed, because `1.10` reads as 1.1.
fn fraction_is_written(fraction: &str, folded_input: &str) -> bool {
    let chars: Vec<char> = folded_input.chars().collect();
    chars.iter().enumerate().any(|(index, ch)| {
        matches!(ch, '.' | ',')
            && chars[index + 1..]
                .iter()
                .take_while(|next| next.is_ascii_digit())
                .collect::<String>()
                .starts_with(fraction)
    })
}

/// Reads a run of CJK numerals the two ways CJK numerals are written.
///
/// Positional (`二〇二四`) spells one digit per place, exactly as `2024` does.
/// Multiplicative (`百二十`) spells magnitudes. A run that mixes the two, or one
/// that is a bare string of digit characters with no zero to mark it positional,
/// has no single agreed reading, so this declines to state one rather than
/// asserting against a reading of its own invention.
fn cjk_numeral(run: &str) -> Option<f64> {
    fn digit(ch: char) -> Option<u32> {
        "〇一二三四五六七八九"
            .chars()
            .position(|c| c == ch)
            .map(|d| d as u32)
    }
    fn place(ch: char) -> Option<f64> {
        match ch {
            '十' => Some(10.0),
            '百' => Some(100.0),
            '千' => Some(1000.0),
            _ => None,
        }
    }
    fn big(ch: char) -> Option<f64> {
        match ch {
            '万' => Some(1e4),
            '億' => Some(1e8),
            '兆' => Some(1e12),
            _ => None,
        }
    }

    if run.is_empty() {
        return None;
    }
    let has_magnitude = run
        .chars()
        .any(|ch| place(ch).is_some() || big(ch).is_some());
    let has_zero = run.contains('〇');

    if has_zero && !has_magnitude {
        // Positional: the run is the number, digit for digit.
        let mut value = 0.0f64;
        for ch in run.chars() {
            value = value * 10.0 + f64::from(digit(ch)?);
        }
        return Some(value);
    }
    if has_zero {
        return None;
    }
    if !has_magnitude {
        // A single digit character is unambiguous; `二四` is not.
        return if run.chars().count() == 1 {
            digit(run.chars().next()?).map(f64::from)
        } else {
            None
        };
    }

    let (mut total, mut section, mut current) = (0.0f64, 0.0f64, 0.0f64);
    for ch in run.chars() {
        if let Some(d) = digit(ch) {
            current = f64::from(d);
        } else if let Some(p) = place(ch) {
            if current == 0.0 {
                current = 1.0;
            }
            section += current * p;
            current = 0.0;
        } else if let Some(b) = big(ch) {
            total += (section + current) * b;
            section = 0.0;
            current = 0.0;
        } else {
            return None;
        }
    }
    Some(total + section + current)
}

const CJK_NUMERALS: &str = "〇一二三四五六七八九十百千万億兆";

/// The maximal runs of CJK numeral characters in the input.
fn cjk_runs(input: &str) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for ch in input.chars() {
        if CJK_NUMERALS.contains(ch) {
            current.push(ch);
        } else if !current.is_empty() {
            runs.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        runs.push(current);
    }
    runs
}

/// Whether `unit` is written in the input as its own token.
///
/// `m` is written in `5 m 3 cm` and in `3m`, but not in `3mm` or `5 min`: a
/// canonical unit that only appears inside a longer word was not read from
/// there, it came out of a conversion, and a converted value cannot be checked
/// digit for digit.
fn unit_written_verbatim(unit: &str, input: &str) -> bool {
    let text = folded(input);
    let unit = unit.to_lowercase();
    if unit.is_empty() {
        return false;
    }
    let bytes: Vec<char> = text.chars().collect();
    let needle: Vec<char> = unit.chars().collect();
    (0..bytes.len().saturating_sub(needle.len() - 1)).any(|start| {
        bytes[start..start + needle.len()] == needle[..]
            && !bytes
                .get(start.wrapping_sub(1))
                .is_some_and(|ch| ch.is_alphabetic())
            && !bytes
                .get(start + needle.len())
                .is_some_and(|ch| ch.is_alphabetic())
    })
}

type Entry = fn(&str, Option<ParseCtx>) -> Parsed;

/// Splits the input into the `<number><unit>` pieces a compound is written as.
fn compound_pieces(input: &str) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let is_digit = |ch: char| ch.is_ascii_digit() || ('０'..='９').contains(&ch);
    let mut pieces = Vec::new();
    let mut index = 0;
    while index < chars.len() {
        if !is_digit(chars[index]) {
            index += 1;
            continue;
        }
        let start = index;
        while index < chars.len()
            && (is_digit(chars[index]) || matches!(chars[index], '.' | '，' | '．'))
        {
            index += 1;
        }
        let mut cursor = index;
        while cursor < chars.len() && chars[cursor] == ' ' {
            cursor += 1;
        }
        let unit_start = cursor;
        while cursor < chars.len() && !is_digit(chars[cursor]) && chars[cursor] != ' ' {
            cursor += 1;
        }
        if cursor > unit_start {
            index = cursor;
        }
        pieces.push(chars[start..index].iter().collect());
    }
    pieces
}

/// Whether the reading is the sum of a compound the input actually writes.
///
/// A compound quantity — `5 m 3 cm`, `2 lb 3 oz`, `5尺3寸`, `1h30` — states a
/// sum because its units descend in magnitude and never repeat: each piece is a
/// smaller place of the same measurement. `3 m 5 m`, `5 kg 3 kg` and
/// `3 cm 5 m` write no such thing, so a sum read out of them is the parser's
/// guess and not the text's statement.
fn is_written_compound(
    reading: &Reading,
    input: &str,
    entry: Entry,
    ctx: &Option<ParseCtx>,
) -> bool {
    let Some(value) = reading.value else {
        return false;
    };
    let pieces = compound_pieces(input);
    if pieces.len() < 2 {
        return false;
    }
    let mut parts = Vec::new();
    for piece in &pieces {
        let parsed = entry(piece, ctx.clone());
        let Some(best) = parsed.best else {
            return false;
        };
        match (best.value, best.unit.clone()) {
            (Some(part), Some(unit)) if best.unit == reading.unit => parts.push((part, unit)),
            _ => return false,
        }
    }
    let sum: f64 = parts.iter().map(|(part, _)| part).sum();
    if (sum - value).abs() > 1e-9 * value.abs().max(1.0) {
        return false;
    }
    // Strictly descending magnitude, no unit written twice.
    parts.windows(2).all(|pair| pair[0].0 > pair[1].0)
        && pieces
            .iter()
            .map(|piece| {
                piece.trim_start_matches(|ch: char| {
                    ch.is_ascii_digit() || ('０'..='９').contains(&ch) || matches!(ch, '.' | ' ')
                })
            })
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            == pieces.len()
}

fn check_not_fabricated(
    reading: &Reading,
    findings: &Findings,
    input: &str,
    entry: &str,
    reader: Entry,
    ctx: &Option<ParseCtx>,
    out: &mut Vec<String>,
) {
    let folded_input = folded(input);
    let mut fail = |message: String| out.push(message);

    if reading.kind == Kind::Range
        && !RANGE_TOKENS
            .iter()
            .any(|token| folded_input.contains(token))
    {
        fail(format!(
            "{entry}({input:?}) invented a range: nothing in the input states one"
        ));
    }

    // A run of CJK numerals is a written number like any other. Exempting it
    // from every check is what let `二〇二四` read as 4: the reading is not the
    // number the run spells, and nothing said so.
    if reading.kind == Kind::Number
        && reading.unit.is_none()
        && ascii_digits(input).is_empty()
        && silent(findings)
        && let Some(value) = reading.value
    {
        let runs = cjk_runs(input);
        if runs.len() == 1
            && let Some(expected) = cjk_numeral(&runs[0])
            && (expected - value).abs() > 1e-9
        {
            fail(format!(
                "{entry}({input:?}) read {value}, but {:?} spells {expected}",
                runs[0]
            ));
        }
    }

    // Only bare numbers are checked digit for digit. A quantity is converted to
    // its canonical unit, so `5cm` legitimately reads as `0.05`, and exponents
    // legitimately carry digits the text does not spell. A quantity whose
    // canonical unit is written verbatim was *not* converted, so it is held to
    // the same rule as a bare number — unless the input writes a compound, whose
    // sum is stated rather than guessed.
    let digits = ascii_digits(input);
    let exponent = folded_input.contains('e');
    let verbatim_unit = reading.kind == Kind::Quantity
        && reading
            .unit
            .as_deref()
            .is_some_and(|unit| unit_written_verbatim(unit, input))
        && !is_written_compound(reading, input, reader, ctx);
    if (reading.kind == Kind::Number && reading.unit.is_none() || verbatim_unit)
        && !exponent
        && !digits.is_empty()
        && let Some(value) = reading.value
    {
        // The strict property: the number is *written* in the input. Its whole
        // part is one unbroken run of digits there, group separators aside, and
        // its fractional digits sit immediately after a decimal mark. Checking
        // the digits as a subsequence instead used to be necessary because
        // `1.2 3` read as 1.23, which no reading of the input spells; that is
        // now refused, so the property can be stated as written.
        let written = format!("{}", value.abs());
        if written.contains('e') {
            fail(format!(
                "{entry}({input:?}) read {value}, which is not written in any form"
            ));
        }
        let (whole, fraction) = written.split_once('.').unwrap_or((written.as_str(), ""));
        // `,12` reads as `0.12`; the leading zero is notation, not a digit
        // taken from the input.
        let whole_digits = whole.trim_start_matches('0');
        if !whole_digits.is_empty() && !digits.contains(whole_digits) {
            fail(format!(
                "{entry}({input:?}) read {value}, whose whole part is not written in the input"
            ));
        }
        if !fraction.is_empty() && !fraction_is_written(fraction, &folded_input) {
            fail(format!(
                "{entry}({input:?}) read {value}, whose fraction is not written in the input"
            ));
        }
    }

    // An endpoint is a reading like any other, and is held to the same rule:
    // `parse("1.2 3-4 kg")` reporting an interval from 1.23 is the fabrication
    // this whole file is about.
    if let Some(range) = reading.range.as_ref() {
        check_not_fabricated(&range.from, findings, input, entry, reader, ctx, out);
        check_not_fabricated(&range.to, findings, input, entry, reader, ctx, out);
    }

    // A unit reported for a string of nothing but digits was not read from the
    // text. Assuming one is allowed — saying nothing about having assumed it is
    // not.
    if let Some(unit) = reading.unit.as_deref() {
        let only_digits = input.chars().all(|ch| {
            ch.is_whitespace() || folded(&ch.to_string()).chars().all(|c| c.is_ascii_digit())
        });
        if only_digits && silent(findings) {
            out.push(format!(
                "{entry}({input:?}) reported unit {unit} with nothing in the input to read it from"
            ));
        }
    }
}

/// Fails once with every violation listed, so one broken shape does not hide
/// the rest.
fn report(name: &str, violations: Vec<String>) {
    if violations.is_empty() {
        return;
    }
    let mut seen = std::collections::BTreeSet::new();
    let unique: Vec<&String> = violations.iter().filter(|v| seen.insert(*v)).collect();
    panic!(
        "{name}: {} violations ({} distinct)\n{}",
        violations.len(),
        unique.len(),
        unique
            .iter()
            .map(|line| format!("  - {line}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A reading has to come from the text it was read out of.
///
/// The removed scanner's worst class was not the value it dropped but the value
/// it made up: `parse_all("1.2 3-4 kg")` reported an interval from 1.23 kg to
/// 4 kg with an empty findings list. A dropped value is detectable downstream —
/// the caller can see the span it never got a reading for — but an invented one
/// arrives indistinguishable from a real one.
#[test]
fn no_reading_is_invented_from_text_that_does_not_hold_it() {
    let mut violations = Vec::new();
    for input in corpus() {
        for ctx in contexts() {
            let named: [(&str, Entry, Parsed); 3] = [
                ("parse", parse as Entry, parse(&input, ctx.clone())),
                (
                    "parse_quantity_fast",
                    parse_quantity_fast as Entry,
                    parse_quantity_fast(&input, ctx.clone()),
                ),
                (
                    "parse_number_fast",
                    parse_number_fast as Entry,
                    parse_number_fast(&input, ctx.clone()),
                ),
            ];
            for (entry, reader, parsed) in named {
                if let Some(best) = parsed.best.as_ref() {
                    check_not_fabricated(
                        best,
                        &parsed.findings,
                        &input,
                        entry,
                        reader,
                        &ctx,
                        &mut violations,
                    );
                }
            }

            for found in parse_dimensions_for_editor(&input, ctx.clone()) {
                if let Some(best) = found.parsed.best.as_ref() {
                    check_not_fabricated(
                        best,
                        &found.parsed.findings,
                        &found.text,
                        "editor",
                        parse as Entry,
                        &ctx,
                        &mut violations,
                    );
                }
            }
        }
    }
    report("fabricated readings", violations);
}

/// Extending a readable input with more text never silently empties the result.
///
/// [`parse_dimensions_for_editor`] returns a bare `Vec`: an empty one carries no
/// reading *and no channel to say why*, which is the exact shape the removed
/// scanner was deleted for. So the empty vector is only honest when there was
/// nothing to read. If `幅3640` yields a match and `幅3640 and 2` yields none,
/// a value the extractor can demonstrably read was dropped without a word —
/// the parser guessed that the longer string held nothing, silently.
#[test]
fn a_longer_context_never_silently_empties_a_readable_input() {
    let bases = [
        "幅3640",
        "3640",
        "3 m",
        "寸法3640",
        "100㎡",
        "3.5m",
        "幅１．５ｍ",
        "3m×4m",
        "高さ2.9m",
        "壁厚105mm",
        "6帖",
        "width 3.5m",
    ];
    let tails = [
        " and 2", " and 4", "、2", " x 2", " 2", " ok", "です", " and 4m", "。", " のLDK",
    ];

    let mut violations = Vec::new();
    for base in bases {
        for ctx in contexts() {
            let before = parse_dimensions_for_editor(base, ctx.clone());
            if before.is_empty() {
                continue;
            }
            for tail in tails {
                let longer = format!("{base}{tail}");
                if parse_dimensions_for_editor(&longer, ctx.clone()).is_empty() {
                    violations.push(format!(
                        "parse_dimensions_for_editor({longer:?}) is empty, but {base:?} reads {:?} \
                         — the loss is reported nowhere",
                        before
                            .iter()
                            .map(|found| found.text.clone())
                            .collect::<Vec<_>>()
                    ));
                }
            }
        }
    }
    report("silently emptied results", violations);
}

/// The net above must not call a valid notation a fabrication.
///
/// A test that rejects everything proves nothing. These are the shapes the crate
/// is documented to read — Indian grouping, and compounds whose units descend
/// without repeating — and each must still read, silently, and pass the
/// fabrication check.
#[test]
fn valid_notation_is_not_mistaken_for_fabrication() {
    let mut violations = Vec::new();
    for input in [
        "1 23 456",
        "1 234 567",
        "5 m 3 cm",
        "2 lb 3 oz",
        "5尺3寸",
        "1h30",
        "1,234",
        "3.5m",
        "1 200",
        "5 W",
        "0",
    ] {
        for ctx in contexts() {
            let parsed = parse(input, ctx.clone());
            let Some(best) = parsed.best.as_ref() else {
                violations.push(format!("parse({input:?}) refused a valid notation"));
                continue;
            };
            check_not_fabricated(
                best,
                &parsed.findings,
                input,
                "parse",
                parse as Entry,
                &ctx,
                &mut violations,
            );
        }
    }
    report("valid notation rejected", violations);
}

/// A short description of what a reading says, for comparing two entry points.
fn shape(reading: &Reading) -> String {
    format!(
        "{:?} value={:?} unit={:?} date={:?} recurrence={:?}",
        reading.kind, reading.value, reading.unit, reading.date, reading.recurrence
    )
}

/// Two entry points that both read the same input agree, or one of them says so.
///
/// A narrow entry point declining to read is not a disagreement — that is what
/// narrow means. But when the broad [`parse`] and a narrow entry point both
/// return a reading of the *same string* and the readings differ, one of them is
/// wrong about what the text says, and the caller has no way to tell which. Two
/// silent, contradictory answers are two guesses; at most one of them can be the
/// deterministic best reading the crate promises.
///
/// `parse` and the numeric entry points are compared whatever kind they land on,
/// because `Number`, `Quantity` and `Range` are three answers to one question:
/// what number does this string hold. `parse_date_fast` and
/// `parse_recurrence_fast` are compared only where `parse` also read a date or a
/// recurrence, since reading a string as a date rather than a quantity is the
/// caller's declaration, not a contradiction.
#[test]
fn entry_points_that_both_read_an_input_do_not_contradict_each_other() {
    let mut violations = Vec::new();
    for input in corpus() {
        for ctx in contexts() {
            let broad = parse(&input, ctx.clone());
            let Some(best) = broad.best.as_ref() else {
                continue;
            };

            let numeric: [(&str, Parsed); 2] = [
                (
                    "parse_quantity_fast",
                    parse_quantity_fast(&input, ctx.clone()),
                ),
                ("parse_number_fast", parse_number_fast(&input, ctx.clone())),
            ];
            let kinded: [(&str, Kind, Parsed); 2] = [
                (
                    "parse_date_fast",
                    Kind::Date,
                    parse_date_fast(&input, ctx.clone()),
                ),
                (
                    "parse_recurrence_fast",
                    Kind::Recurrence,
                    parse_recurrence_fast(&input, ctx.clone()),
                ),
            ];

            let mut compare = |entry: &str, parsed: &Parsed| {
                let Some(other) = parsed.best.as_ref() else {
                    return;
                };
                if shape(best) != shape(other)
                    && silent(&broad.findings)
                    && silent(&parsed.findings)
                {
                    violations.push(format!(
                        "{input:?}: parse says [{}] but {entry} says [{}], both with no finding",
                        shape(best),
                        shape(other)
                    ));
                }
            };
            for (entry, parsed) in &numeric {
                compare(entry, parsed);
            }
            for (entry, kind, parsed) in &kinded {
                if best.kind == *kind {
                    compare(entry, parsed);
                }
            }
        }
    }
    report("contradicting entry points", violations);
}

/// The instance of the fabrication class that used to live in `parse`.
///
/// `parse("1.2 3-4 kg")` reported an interval from 1.23 kg to 4 kg with an
/// empty findings list. `1.23` was the space-grouping rule (`1 200` reads as
/// 1200) applied across a decimal point: a group separator cannot follow one, so
/// the endpoint was written nowhere in the input, and nothing on any channel
/// said a choice had been made. Space-style separators are now validated the
/// way the comma and dot paths validate theirs, so the malformed shapes are
/// refused with a finding while real grouping keeps reading.
#[test]
fn space_grouping_across_a_decimal_point_is_refused_not_invented() {
    for input in ["1.2 3-4 kg", "1.2 3", "1 2", "1 2020", "1_2", "1.234 567"] {
        let parsed = parse(input, None);
        assert!(
            parsed.best.is_none(),
            "{input:?} was read as {:?}",
            parsed.best
        );
        assert!(
            !silent(&parsed.findings),
            "{input:?} produced no reading and no finding"
        );
    }

    // Grouping that is grouping still reads, western and Indian alike, and
    // through every space-family separator the crate accepts.
    for (input, expected) in [
        ("1 200", 1200.0),
        ("1 234 567", 1_234_567.0),
        ("12 34 567", 1_234_567.0),
        ("1_234", 1234.0),
        ("- 1 234", -1234.0),
        ("1 234,56", 1234.56),
        ("1\u{00A0}234", 1234.0),
        ("1\u{202F}234", 1234.0),
        ("1\u{2009}234", 1234.0),
    ] {
        let parsed = parse(input, None);
        let best = parsed.best.as_ref().unwrap_or_else(|| panic!("{input:?}"));
        assert_eq!(best.value, Some(expected), "{input:?}");
    }
}
