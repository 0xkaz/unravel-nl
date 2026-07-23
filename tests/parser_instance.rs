use unravel_nl::{Dimension, DimensionSet, IssueCode, ParseCtx, Parser, UnitRegistry};

#[test]
fn default_public_parser_is_length_and_area_only() {
    let parser = Parser::default();

    assert_eq!(
        parser.context().unit_registry,
        UnitRegistry::only(DimensionSet::of(&[Dimension::Length, Dimension::Area,]))
    );
    assert!(parser.parse("5 m").best.is_some());
    assert!(parser.parse("6 m2").best.is_some());
    assert!(parser.parse("5 kg").best.is_none());
}

#[test]
fn empty_public_registry_still_reads_dimensionless_values() {
    let parser = Parser::new(DimensionSet::new());

    assert!(parser.parse("3640").best.is_some());
    assert!(parser.parse("5 m").best.is_none());
}

#[test]
fn registry_scope_does_not_invent_a_typo_from_a_known_disabled_unit() {
    let parsed = Parser::new(DimensionSet::from(Dimension::Length)).parse("5 kg");

    assert!(parsed.best.is_none());
    assert!(parsed.alternatives.is_empty());
    assert!(parsed.suggestions.is_empty());
    assert_eq!(parsed.findings.skipped[0].code, IssueCode::NoValue);
}

#[test]
fn unrestricted_policy_can_still_surface_a_refused_alternative() {
    let parser = Parser::unrestricted_with_context(ParseCtx {
        expected_dimensions: DimensionSet::from(Dimension::Length),
        ..ParseCtx::default()
    });
    let parsed = parser.parse("5 kg");

    assert!(parsed.best.is_none());
    assert_eq!(parsed.alternatives[0].dimension, Some(Dimension::Mass));
    assert_eq!(parsed.findings.skipped[0].code, IssueCode::RejectedByPolicy);
}
