use unravel_nl::{
    Dimension, Kind, Locale, ParseCtx, ParsePurpose, parse, parse_dimensions_for_editor,
};

#[test]
fn extracts_editor_dimensions_without_general_values() {
    let input =
        "幅3m×奥行4m、予算1234、予算¥999、next friday、6帖、寸法3640、壁厚105mm、幅１．５ｍ";
    let matches = parse_dimensions_for_editor(
        input,
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(
        texts(&matches),
        vec!["3m", "4m", "6帖", "3640", "105mm", "１．５ｍ"]
    );
    assert_quantity(&matches[0], 3.0, "m", Dimension::Length);
    assert_quantity(&matches[1], 4.0, "m", Dimension::Length);
    assert_quantity(&matches[2], 9.72, "m2", Dimension::Area);
    assert_quantity(&matches[4], 0.105, "m", Dimension::Length);
    assert_quantity(&matches[5], 1.5, "m", Dimension::Length);

    let plain = matches[3].parsed.best.as_ref().expect("plain number");
    assert_eq!(plain.kind, Kind::Number);
    assert_eq!(plain.value, Some(3640.0));
    assert_eq!(
        matches[3].parsed.alternatives[0].unit.as_deref(),
        Some("mm")
    );

    // A space-grouped full-width number under the same label reads as one
    // value, not as the two runs of digits the window walks over.
    let grouped = parse_dimensions_for_editor(
        "寸法１ ２００",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );
    assert_eq!(texts(&grouped), vec!["１ ２００"]);
    let best = grouped[0].parsed.best.as_ref().expect("grouped number");
    assert_eq!(best.kind, Kind::Number);
    assert_eq!(best.value, Some(1200.0));
    assert_eq!(
        grouped[0].parsed.alternatives[0].unit.as_deref(),
        Some("mm")
    );
}

#[test]
fn editor_dimension_scanner_does_not_guess_unknown_unitless_labels() {
    let matches = parse_dimensions_for_editor(
        "部材3640、備考1234、north 800、somewidth 700、grossarea 800、subfloor area 1100、firewall thickness 104、room width 900、room w 910、floor area 12㎡、first floor area 13㎡、wall thickness 105mm、exterior wall thickness 106mm",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(
        texts(&matches),
        vec!["900", "910", "12㎡", "13㎡", "105mm", "106mm"]
    );
    assert_eq!(
        matches[0].parsed.alternatives[0].unit.as_deref(),
        Some("mm")
    );
    assert_eq!(
        matches[1].parsed.alternatives[0].unit.as_deref(),
        Some("mm")
    );
    assert_quantity(&matches[2], 12.0, "m2", Dimension::Area);
    assert_quantity(&matches[3], 13.0, "m2", Dimension::Area);
    assert_quantity(&matches[4], 0.105, "m", Dimension::Length);
    assert_quantity(&matches[5], 0.106, "m", Dimension::Length);
}

#[test]
fn editor_dimension_scanner_rejects_label_dimension_mismatches() {
    let matches = parse_dimensions_for_editor(
        "floor area 105mm、wall thickness 12㎡、floor area 1200、floor area 12㎡、wall thickness 105mm",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(texts(&matches), vec!["12㎡", "105mm"]);
    assert_quantity(&matches[0], 12.0, "m2", Dimension::Area);
    assert_quantity(&matches[1], 0.105, "m", Dimension::Length);
}

#[test]
fn editor_dimension_scanner_rejects_embedded_identifier_quantities() {
    let matches = parse_dimensions_for_editor(
        "beamA105mm、part_1200mm、room width900mm、wall thickness105mm、壁厚105mm",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(texts(&matches), vec!["900mm", "105mm", "105mm"]);
    assert_quantity(&matches[0], 0.9, "m", Dimension::Length);
    assert_quantity(&matches[1], 0.105, "m", Dimension::Length);
    assert_quantity(&matches[2], 0.105, "m", Dimension::Length);
}

#[test]
fn parse_purpose_limits_broad_parser_work() {
    let dimension = parse(
        "3640",
        Some(ParseCtx {
            purpose: ParsePurpose::DimensionEditor,
            ..ParseCtx::default()
        }),
    );
    assert_eq!(dimension.best.as_ref().unwrap().kind, Kind::Number);
    assert_eq!(dimension.alternatives[0].unit.as_deref(), Some("mm"));

    let non_dimension = parse(
        "¥1,234",
        Some(ParseCtx {
            purpose: ParsePurpose::DimensionEditor,
            ..ParseCtx::default()
        }),
    );
    assert!(non_dimension.best.is_none(), "{non_dimension:#?}");
}

fn texts(matches: &[unravel_nl::ParsedMatch]) -> Vec<&str> {
    matches
        .iter()
        .map(|parsed_match| parsed_match.text.as_str())
        .collect()
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
