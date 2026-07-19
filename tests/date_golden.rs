#![cfg(feature = "dates-jiff")]

use unravel_nl::{Date, Kind, Locale, ParseCtx, parse};

const DATE_GOLDEN: &str = include_str!("corpus/date_golden.tsv");

#[test]
fn relative_date_golden_corpus_uses_explicit_reference_date() {
    let reference_date = Date::new(2026, 7, 19);

    for (line_no, line) in DATE_GOLDEN.lines().enumerate() {
        let line_no = line_no + 1;
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 3, "line {line_no}: {line}");

        let parsed = parse(
            columns[0],
            Some(ParseCtx {
                locale: parse_locale(columns[1]),
                reference_date,
                timezone: Some("Asia/Tokyo".to_owned()),
                ..ParseCtx::default()
            }),
        );
        let best = parsed
            .best
            .unwrap_or_else(|| panic!("line {line_no}: no best for {:?}", columns[0]));
        assert_eq!(best.kind, Kind::Date, "line {line_no}: {}", columns[0]);
        assert_eq!(
            best.date.as_deref(),
            Some(columns[2]),
            "line {line_no}: {}",
            columns[0]
        );
    }
}

fn parse_locale(value: &str) -> Option<Locale> {
    match value {
        "ja" => Some(Locale::Ja),
        "en" => Some(Locale::En),
        "en-US" => Some(Locale::EnUs),
        "en-GB" => Some(Locale::EnGb),
        "" => None,
        other => Some(Locale::Other(other.to_owned())),
    }
}
