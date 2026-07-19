use unravel_nl::{Dimension, Kind, Locale, ParseCtx, parse_all};

#[test]
fn extracts_multiple_values_with_spans() {
    let input = "延床100㎡、敷地面積120㎡、高さ3.5m、予算¥1,234";
    let matches = parse_all(
        input,
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(matches.len(), 4, "{matches:#?}");
    assert_eq!(matches[0].text, "延床100㎡");
    assert_eq!(matches[1].text, "120㎡");
    assert_eq!(matches[2].text, "3.5m");
    assert_eq!(matches[3].text, "¥1,234");

    assert_quantity(&matches[0], 100.0, "m2", Dimension::Area);
    assert_quantity(&matches[1], 120.0, "m2", Dimension::Area);
    assert_quantity(&matches[2], 3.5, "m", Dimension::Length);
    assert_quantity(&matches[3], 1234.0, "JPY", Dimension::Currency);

    for parsed_match in &matches {
        assert_eq!(
            input.get(parsed_match.start..parsed_match.end),
            Some(parsed_match.text.as_str())
        );
    }
}

#[test]
fn extracts_full_width_and_cjk_number_values() {
    let input = "幅１．５ｍ；重量五キログラム；面積百二十平米";
    let matches = parse_all(
        input,
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(matches.len(), 3, "{matches:#?}");
    assert_quantity(&matches[0], 1.5, "m", Dimension::Length);
    assert_quantity(&matches[1], 5.0, "kg", Dimension::Mass);
    assert_quantity(&matches[2], 120.0, "m2", Dimension::Area);
}

fn assert_quantity(
    parsed_match: &unravel_nl::ParsedMatch,
    expected_value: f64,
    expected_unit: &str,
    expected_dimension: Dimension,
) {
    let best = parsed_match.parsed.best.as_ref().expect("best");
    assert_eq!(best.kind, Kind::Quantity, "{}", parsed_match.text);
    assert_eq!(
        best.unit.as_deref(),
        Some(expected_unit),
        "{}",
        parsed_match.text
    );
    assert_eq!(
        best.dimension,
        Some(expected_dimension),
        "{}",
        parsed_match.text
    );
    assert!(
        (best.value.expect("value") - expected_value).abs() < 1e-9,
        "{}: expected {expected_value}, got {:?}",
        parsed_match.text,
        best.value
    );
}
