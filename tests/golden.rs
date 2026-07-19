use unravel_nl::{Dimension, Kind, Locale, ParseCtx, parse};

const GOLDEN: &str = include_str!("corpus/golden.tsv");

#[test]
fn golden_corpus_matches_canonical_readings() {
    for (line_no, line) in GOLDEN.lines().enumerate() {
        let line_no = line_no + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }

        let mut columns: Vec<&str> = line.split('\t').collect();
        assert!(columns.len() <= 9, "line {line_no}: {line}");
        columns.resize(9, "");

        let input = columns[0];
        let locale = parse_locale(columns[1]);
        let expected_kind = parse_kind(columns[2]);
        let expected_dimension = parse_dimension(columns[3]);
        let expected_unit = empty_as_none(columns[4]);
        let expected_value = parse_optional_f64(columns[5]);
        let expected_date = empty_as_none(columns[6]);
        let expected_range_from = parse_optional_f64(columns[7]);
        let expected_range_to = parse_optional_f64(columns[8]);

        let parsed = parse(
            input,
            Some(ParseCtx {
                locale,
                ..ParseCtx::default()
            }),
        );
        let best = parsed
            .best
            .unwrap_or_else(|| panic!("line {line_no}: no best for {input:?}"));

        assert_eq!(best.kind, expected_kind, "line {line_no}: {input}");
        if let Some(dimension) = expected_dimension {
            if best.kind == Kind::Range {
                let range = best.range.as_ref().expect("range");
                assert_eq!(
                    range.from.dimension,
                    Some(dimension),
                    "line {line_no}: {input}"
                );
                assert_eq!(
                    range.to.dimension,
                    Some(dimension),
                    "line {line_no}: {input}"
                );
            } else {
                assert_eq!(best.dimension, Some(dimension), "line {line_no}: {input}");
            }
        }
        if let Some(unit) = expected_unit {
            if best.kind == Kind::Range {
                let range = best.range.as_ref().expect("range");
                assert_eq!(
                    range.from.unit.as_deref(),
                    Some(unit),
                    "line {line_no}: {input}"
                );
                assert_eq!(
                    range.to.unit.as_deref(),
                    Some(unit),
                    "line {line_no}: {input}"
                );
            } else {
                assert_eq!(best.unit.as_deref(), Some(unit), "line {line_no}: {input}");
            }
        }
        if let Some(value) = expected_value {
            assert_close(best.value.expect("value"), value, input);
        }
        if let Some(date) = expected_date {
            assert_eq!(best.date.as_deref(), Some(date), "line {line_no}: {input}");
        }
        if let Some(range_from) = expected_range_from {
            let range = best.range.as_ref().expect("range");
            assert_close(range.from.value.expect("range from"), range_from, input);
        }
        if let Some(range_to) = expected_range_to {
            let range = best.range.as_ref().expect("range");
            assert_close(range.to.value.expect("range to"), range_to, input);
        }
    }
}

fn empty_as_none(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

fn parse_optional_f64(value: &str) -> Option<f64> {
    empty_as_none(value).map(|value| value.parse().expect("fixture number"))
}

fn parse_locale(value: &str) -> Option<Locale> {
    match empty_as_none(value)? {
        "ja" => Some(Locale::Ja),
        "en" => Some(Locale::En),
        "en-US" => Some(Locale::EnUs),
        "en-GB" => Some(Locale::EnGb),
        other => Some(Locale::Other(other.to_owned())),
    }
}

fn parse_kind(value: &str) -> Kind {
    match value {
        "quantity" => Kind::Quantity,
        "date" => Kind::Date,
        "range" => Kind::Range,
        "number" => Kind::Number,
        "recurrence" => Kind::Recurrence,
        other => panic!("unknown kind {other:?}"),
    }
}

fn parse_dimension(value: &str) -> Option<Dimension> {
    Some(match empty_as_none(value)? {
        "length" => Dimension::Length,
        "area" => Dimension::Area,
        "mass" => Dimension::Mass,
        "time" => Dimension::Time,
        "volume" => Dimension::Volume,
        "currency" => Dimension::Currency,
        "temperature" => Dimension::Temperature,
        "speed" => Dimension::Speed,
        "data" => Dimension::Data,
        "data_rate" => Dimension::DataRate,
        "flow_rate" => Dimension::FlowRate,
        "concentration" => Dimension::Concentration,
        "acceleration" => Dimension::Acceleration,
        "force" => Dimension::Force,
        "torque" => Dimension::Torque,
        "pressure" => Dimension::Pressure,
        "power" => Dimension::Power,
        "charge" => Dimension::Charge,
        "voltage" => Dimension::Voltage,
        "current" => Dimension::Current,
        "resistance" => Dimension::Resistance,
        "illuminance" => Dimension::Illuminance,
        "radiation_equivalent_dose" => Dimension::RadiationEquivalentDose,
        "radioactivity" => Dimension::Radioactivity,
        other => panic!("unknown dimension {other:?}"),
    })
}

fn assert_close(actual: f64, expected: f64, label: &str) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "{label}: expected {expected}, got {actual}"
    );
}
