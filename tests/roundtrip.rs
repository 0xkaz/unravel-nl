use unravel_nl::{HumanizeCtx, Kind, Locale, ParseCtx, Reading, humanize, parse};

const GOLDEN: &str = include_str!("corpus/golden.tsv");

#[test]
fn humanized_core_values_parse_back_to_same_canonical_value() {
    for case in [
        RoundTripCase::new("180cm", None, None),
        RoundTripCase::new("68 F", None, None),
        RoundTripCase::new("12 bucks", None, None),
        RoundTripCase::new("100-120㎡", Some(Locale::Ja), None),
        RoundTripCase::new("5尺3寸", Some(Locale::Ja), Some(Locale::Ja)),
        RoundTripCase::new("1坪", Some(Locale::Ja), Some(Locale::Ja)),
        RoundTripCase::new("午後3時", Some(Locale::Ja), None),
    ] {
        let parsed = parse(case.input, case.parse_ctx());
        let first = parsed.best.expect(case.input);
        let rendered = humanize(&first, case.humanize_ctx());
        let reparsed = parse(&rendered, case.parse_ctx());
        let second = reparsed
            .best
            .unwrap_or_else(|| panic!("{} humanized as {rendered:?} did not parse", case.input));
        assert_same_canonical(&first, &second, case.input, &rendered);
    }
}

/// Every canonical reading in the maintained parser corpus must survive the
/// public reverse direction. This is deliberately driven by the whole corpus,
/// not a second hand-picked list that can drift away from parser coverage.
#[test]
fn every_golden_reading_humanizes_back_to_the_same_canonical_value() {
    for (index, line) in GOLDEN.lines().enumerate() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let mut columns = line.split('\t');
        let input = columns.next().expect("golden input");
        let locale = parse_locale(columns.next().unwrap_or_default());
        let parse_ctx = Some(ParseCtx {
            locale: locale.clone(),
            ..ParseCtx::default()
        });
        let humanize_ctx = Some(HumanizeCtx { locale });
        let parsed = parse(input, parse_ctx.clone());

        for first in parsed.best.iter().chain(parsed.alternatives.iter()) {
            let rendered = humanize(first, humanize_ctx.clone());
            let reparsed = parse(&rendered, parse_ctx.clone());
            let second = reparsed.best.unwrap_or_else(|| {
                panic!(
                    "golden line {}: {input:?} humanized as {rendered:?} but did not parse: {:?}",
                    index + 1,
                    reparsed.findings
                )
            });
            assert_same_canonical(first, &second, input, &rendered);
        }
    }
}

#[test]
fn humanize_preserves_values_below_the_old_six_decimal_floor() {
    for input in ["0.0000001 m", "0.473176473 m", "-0.0000001 m"] {
        let first = parse(input, None).best.expect(input);
        let rendered = humanize(&first, None);
        let second = parse(&rendered, None).best.expect(&rendered);
        assert_same_canonical(&first, &second, input, &rendered);
    }
}

fn parse_locale(locale: &str) -> Option<Locale> {
    match locale {
        "" => None,
        "ja" => Some(Locale::Ja),
        "en" => Some(Locale::En),
        "en-US" => Some(Locale::EnUs),
        "en-GB" => Some(Locale::EnGb),
        other => Some(Locale::Other(other.to_owned())),
    }
}

#[cfg(feature = "dates-jiff")]
#[test]
fn humanized_dates_parse_back_to_same_date() {
    let parsed = parse(
        "next friday",
        Some(ParseCtx {
            locale: Some(Locale::En),
            reference_date: unravel_nl::Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        }),
    );
    let first = parsed.best.expect("date");
    let rendered = humanize(&first, None);
    let second = parse(
        &rendered,
        Some(ParseCtx {
            locale: Some(Locale::En),
            reference_date: unravel_nl::Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        }),
    )
    .best
    .expect("humanized date");
    assert_eq!(first.kind, Kind::Date);
    assert_eq!(second.kind, Kind::Date);
    assert_eq!(first.date, second.date);
}

struct RoundTripCase {
    input: &'static str,
    parse_locale: Option<Locale>,
    humanize_locale: Option<Locale>,
}

impl RoundTripCase {
    const fn new(
        input: &'static str,
        parse_locale: Option<Locale>,
        humanize_locale: Option<Locale>,
    ) -> Self {
        Self {
            input,
            parse_locale,
            humanize_locale,
        }
    }

    fn parse_ctx(&self) -> Option<ParseCtx> {
        Some(ParseCtx {
            locale: self.parse_locale.clone(),
            ..ParseCtx::default()
        })
    }

    fn humanize_ctx(&self) -> Option<HumanizeCtx> {
        Some(HumanizeCtx {
            locale: self.humanize_locale.clone(),
        })
    }
}

fn assert_same_canonical(first: &Reading, second: &Reading, input: &str, rendered: &str) {
    assert_eq!(first.kind, second.kind, "{input} -> {rendered}");
    match first.kind {
        Kind::Quantity | Kind::Number => {
            assert_eq!(first.unit, second.unit, "{input} -> {rendered}");
            assert_eq!(first.dimension, second.dimension, "{input} -> {rendered}");
            assert_close(
                first.value.expect("first value"),
                second.value.expect("second value"),
                input,
                rendered,
            );
        }
        Kind::Date => assert_eq!(first.date, second.date, "{input} -> {rendered}"),
        Kind::Range => {
            let first_range = first.range.as_ref().expect("first range");
            let second_range = second.range.as_ref().expect("second range");
            assert_same_canonical(&first_range.from, &second_range.from, input, rendered);
            assert_same_canonical(&first_range.to, &second_range.to, input, rendered);
        }
    }
}

fn assert_close(actual: f64, expected: f64, input: &str, rendered: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{input} -> {rendered}: expected {expected}, got {actual}"
    );
}
