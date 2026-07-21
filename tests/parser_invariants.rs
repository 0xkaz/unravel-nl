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

fn assert_not_fabricated(reading: &Reading, findings: &Findings, input: &str, entry: &str) {
    let folded_input = folded(input);

    if reading.kind == Kind::Range {
        assert!(
            RANGE_TOKENS
                .iter()
                .any(|token| folded_input.contains(token)),
            "{entry}({input:?}) invented a range: nothing in the input states one"
        );
    }

    // Only bare numbers are checked digit for digit. A quantity is converted to
    // its canonical unit, so `5cm` legitimately reads as `0.05`, and CJK
    // numerals and exponents legitimately carry digits the text does not spell.
    let digits = ascii_digits(input);
    let cjk = input
        .chars()
        .any(|ch| "〇一二三四五六七八九十百千万億兆".contains(ch));
    let exponent = folded_input.contains('e');
    if reading.kind == Kind::Number
        && reading.unit.is_none()
        && !cjk
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
        assert!(
            !written.contains('e'),
            "{entry}({input:?}) read {value}, which is not written in any form"
        );
        let (whole, fraction) = written.split_once('.').unwrap_or((written.as_str(), ""));
        // `,12` reads as `0.12`; the leading zero is notation, not a digit
        // taken from the input.
        let whole_digits = whole.trim_start_matches('0');
        assert!(
            whole_digits.is_empty() || digits.contains(whole_digits),
            "{entry}({input:?}) read {value}, whose whole part is not written in the input"
        );
        assert!(
            fraction.is_empty() || fraction_is_written(fraction, &folded_input),
            "{entry}({input:?}) read {value}, whose fraction is not written in the input"
        );
    }

    // An endpoint is a reading like any other, and is held to the same rule:
    // `parse("1.2 3-4 kg")` reporting an interval from 1.23 is the fabrication
    // this whole file is about.
    if let Some(range) = reading.range.as_ref() {
        assert_not_fabricated(&range.from, findings, input, entry);
        assert_not_fabricated(&range.to, findings, input, entry);
    }

    // A unit reported for a string of nothing but digits was not read from the
    // text. Assuming one is allowed — saying nothing about having assumed it is
    // not.
    if let Some(unit) = reading.unit.as_deref() {
        let only_digits = input.chars().all(|ch| {
            ch.is_whitespace() || folded(&ch.to_string()).chars().all(|c| c.is_ascii_digit())
        });
        assert!(
            !only_digits || !silent(findings),
            "{entry}({input:?}) reported unit {unit} with nothing in the input to read it from"
        );
    }
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
    for input in corpus() {
        for ctx in contexts() {
            let named: [(&str, Parsed); 3] = [
                ("parse", parse(&input, ctx.clone())),
                (
                    "parse_quantity_fast",
                    parse_quantity_fast(&input, ctx.clone()),
                ),
                ("parse_number_fast", parse_number_fast(&input, ctx.clone())),
            ];
            for (entry, parsed) in named {
                if let Some(best) = parsed.best.as_ref() {
                    assert_not_fabricated(best, &parsed.findings, &input, entry);
                }
            }

            for found in parse_dimensions_for_editor(&input, ctx.clone()) {
                if let Some(best) = found.parsed.best.as_ref() {
                    assert_not_fabricated(best, &found.parsed.findings, &found.text, "editor");
                }
            }
        }
    }
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
