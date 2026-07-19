use unravel_nl::{
    Dimension, IssueCode, Kind, Locale, ParseCtx, ParsePurpose, Strictness, parse, parse_all,
    parse_dimensions_for_editor,
};

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
fn scanner_keeps_sorted_non_overlapping_matches() {
    let matches = parse_all(
        "3m 3m、約3m",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );

    assert_eq!(texts(&matches), vec!["3m 3m", "約3m"]);
    assert!(matches.windows(2).all(|pair| pair[0].end <= pair[1].start));

    let single = parse_all("3m", None);
    assert_eq!(texts(&single), vec!["3m"]);

    let range = parse_all("between 5 and 10 kg", None);
    assert_eq!(texts(&range), vec!["between 5 and 10 kg"]);

    let approximate = parse_all("about 3m", None);
    assert_eq!(texts(&approximate), vec!["about 3m"]);

    let uppercase_approximate = parse_all("ABOUT 3m", None);
    assert_eq!(texts(&uppercase_approximate), vec!["ABOUT 3m"]);

    let approximate_many = parse_all("about 3m and 4m", None);
    assert_eq!(texts(&approximate_many), vec!["3m", "4m"]);

    let typo_then_dimension = parse_all("mebers 3m", None);
    assert_eq!(texts(&typo_then_dimension), vec!["3m"]);

    let typo_then_area = parse_all(
        "tsbo 6帖",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );
    assert_eq!(texts(&typo_then_area), vec!["6帖"]);
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

#[test]
fn extracts_editor_dimension_windows() {
    let ctx = Some(ParseCtx {
        locale: Some(Locale::Ja),
        expected_dimension: Some(Dimension::Length),
        ..ParseCtx::default()
    });

    let room = parse_all("3m×4m のLDK", ctx.clone());
    assert_eq!(texts(&room), vec!["3m", "4m"]);
    assert_quantity(&room[0], 3.0, "m", Dimension::Length);
    assert_quantity(&room[1], 4.0, "m", Dimension::Length);

    let wall = parse_all("壁厚105mm", ctx.clone());
    assert_eq!(texts(&wall), vec!["105mm"]);
    assert_quantity(&wall[0], 0.105, "m", Dimension::Length);

    let height = parse_all("高さ2.9m", ctx.clone());
    assert_eq!(texts(&height), vec!["2.9m"]);
    assert_quantity(&height[0], 2.9, "m", Dimension::Length);

    let plain = parse_all("寸法3640", ctx);
    assert_eq!(texts(&plain), vec!["3640"]);
    let best = plain[0].parsed.best.as_ref().expect("plain number");
    assert_eq!(best.kind, Kind::Number);
    assert_eq!(best.value, Some(3640.0));
    assert_eq!(plain[0].parsed.alternatives[0].unit.as_deref(), Some("mm"));
}

#[test]
fn extracts_area_and_strict_approximation_policy() {
    let areas = parse_all(
        "6帖 / 4畳半",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );
    assert_eq!(texts(&areas), vec!["6帖", "4畳半"]);
    assert_quantity(&areas[0], 9.72, "m2", Dimension::Area);
    assert_quantity(&areas[1], 7.29, "m2", Dimension::Area);
    assert!(!areas[0].parsed.findings.approximations.is_empty());
    assert!(!areas[1].parsed.findings.approximations.is_empty());

    let strict = parse_all(
        "約3m",
        Some(ParseCtx {
            strictness: Strictness::Strict,
            ..ParseCtx::default()
        }),
    );
    assert_eq!(texts(&strict), vec!["約3m"]);
    assert!(strict[0].parsed.best.is_none());
    assert_eq!(
        strict[0].parsed.findings.skipped[0].code,
        IssueCode::Approximation
    );
}

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
