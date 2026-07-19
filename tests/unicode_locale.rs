use unravel_nl::{Dimension, Kind, Locale, ParseCtx, parse};

#[test]
fn normalizes_full_width_and_compatibility_units() {
    for (input, expected_value, expected_unit, expected_dimension) in [
        ("１．５ｍ", 1.5, "m", Dimension::Length),
        ("１㍍", 1.0, "m", Dimension::Length),
        ("２㎞", 2000.0, "m", Dimension::Length),
        ("５㎏", 5.0, "kg", Dimension::Mass),
        ("百二十平方メートル", 120.0, "m2", Dimension::Area),
        ("五キログラム", 5.0, "kg", Dimension::Mass),
    ] {
        let parsed = parse(
            input,
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect(input);
        assert_eq!(best.kind, Kind::Quantity, "{input}");
        assert_eq!(best.unit.as_deref(), Some(expected_unit), "{input}");
        assert_eq!(best.dimension, Some(expected_dimension), "{input}");
        assert_close(best.value.expect("value"), expected_value, input);
    }
}

#[test]
fn parses_locale_number_formats() {
    for (input, expected_value, expected_unit, expected_dimension) in [
        ("1.234,56 kg", 1234.56, "kg", Dimension::Mass),
        ("1 234,56 m", 1234.56, "m", Dimension::Length),
        ("1\u{202f}234,56 EUR", 1234.56, "EUR", Dimension::Currency),
        ("1,23,456 kg", 123456.0, "kg", Dimension::Mass),
        ("3.5万円", 35000.0, "JPY", Dimension::Currency),
    ] {
        let parsed = parse(input, None);
        let best = parsed.best.expect(input);
        assert_eq!(best.kind, Kind::Quantity, "{input}");
        assert_eq!(best.unit.as_deref(), Some(expected_unit), "{input}");
        assert_eq!(best.dimension, Some(expected_dimension), "{input}");
        assert_close(best.value.expect("value"), expected_value, input);
    }
}

#[test]
fn parses_large_japanese_numbers() {
    for (input, expected) in [
        ("2億", 200_000_000.0),
        ("三万五千", 35_000.0),
        ("1万2345", 12_345.0),
    ] {
        let parsed = parse(input, None);
        let best = parsed.best.expect(input);
        assert_eq!(best.kind, Kind::Number, "{input}");
        assert_close(best.value.expect("value"), expected, input);
    }
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{label}: expected {expected}, got {actual}"
    );
}
