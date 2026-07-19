use unravel_nl::{Dimension, Kind, parse};

#[test]
fn parses_same_dimension_mixed_compound_units() {
    let yard_feet = parse("3 yd 2 ft", None).best.expect("yard feet");
    assert_eq!(yard_feet.kind, Kind::Quantity);
    assert_eq!(yard_feet.dimension, Some(Dimension::Length));
    assert_eq!(yard_feet.unit.as_deref(), Some("m"));
    assert_close(yard_feet.value.unwrap(), 3.3528);

    let stone_pounds = parse("4 stone 6 lb", None).best.expect("stone pounds");
    assert_eq!(stone_pounds.dimension, Some(Dimension::Mass));
    assert_eq!(stone_pounds.unit.as_deref(), Some("kg"));
    assert_close(stone_pounds.value.unwrap(), 28.122_726_94);
}

#[test]
fn rejects_cross_dimension_compound_units() {
    let parsed = parse("3 yd 2 kg", None);
    assert!(parsed.best.is_none());
    assert!(!parsed.findings.skipped.is_empty());
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-9,
        "expected {expected}, got {actual}"
    );
}
