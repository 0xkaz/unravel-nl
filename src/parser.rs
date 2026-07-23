//! A configured parser instance.

use crate::*;

/// A reusable parser with one explicit context and unit-domain boundary.
///
/// Keeping the context on the instance makes the enabled dimensions a property
/// of the parser rather than an optional hint repeated at every call.
#[derive(Clone, Debug, PartialEq)]
pub struct Parser {
    ctx: ParseCtx,
}

pub(crate) const fn minimal_dimensions() -> DimensionSet {
    DimensionSet::of(&[Dimension::Length, Dimension::Area])
}

impl Default for Parser {
    /// Creates the minimal general preset: length and area units, with no
    /// locale assumption.
    fn default() -> Self {
        Self::new(minimal_dimensions())
    }
}

impl Parser {
    /// Creates a parser for explicit measurement domains.
    ///
    /// The same set controls the loaded built-in registry and accepted output,
    /// so an unrelated unit is absent rather than parsed and then refused.
    pub fn new(dimensions: DimensionSet) -> Self {
        Self {
            ctx: ParseCtx {
                expected_dimensions: dimensions,
                unit_registry: UnitRegistry::only(dimensions),
                ..ParseCtx::default()
            },
        }
    }

    /// Creates a parser registry for explicit domains with additional policy.
    ///
    /// This method preserves `ctx.expected_dimensions`, allowing the accepted
    /// output to be narrower than the vocabulary when a caller needs refused
    /// alternatives. Use [`Parser::new`] when registry and acceptance should
    /// be identical.
    pub fn with_context(dimensions: DimensionSet, mut ctx: ParseCtx) -> Self {
        ctx.unit_registry = UnitRegistry::only(dimensions);
        Self { ctx }
    }

    /// Creates the small preset for Japanese building dimensions.
    ///
    /// Only length and area measurement grammars and registry entries are
    /// enabled. Dimensionless numbers and dates remain readable because they
    /// are not unit-registry entries.
    pub fn japanese_building() -> Self {
        let dimensions = minimal_dimensions();
        Self::with_context(
            dimensions,
            ParseCtx {
                locale: Some(Locale::Ja),
                expected_dimensions: dimensions,
                ..ParseCtx::default()
            },
        )
    }

    /// Creates an explicitly unrestricted parser.
    ///
    /// This is available for exploration and compatibility, but a field that
    /// knows its dimensions should use [`Parser::new`] with a non-empty set.
    pub fn unrestricted() -> Self {
        Self {
            ctx: ParseCtx::default(),
        }
    }

    /// Creates an explicitly unrestricted parser with additional policy.
    ///
    /// The context's unit registry is replaced with [`UnitRegistry::All`].
    /// Use [`Parser::with_context`] when measurement vocabulary must be scoped.
    pub fn unrestricted_with_context(mut ctx: ParseCtx) -> Self {
        ctx.unit_registry = UnitRegistry::all();
        Self { ctx }
    }

    /// Returns the context shared by every operation on this parser.
    pub fn context(&self) -> &ParseCtx {
        &self.ctx
    }

    /// Parses one whole input value through this instance.
    pub fn parse(&self, text: &str) -> Parsed {
        crate::entry::parse(text, Some(self.ctx.clone()))
    }

    /// Extracts labelled editor dimensions through this instance.
    pub fn parse_dimensions_for_editor(&self, text: &str) -> Vec<ParsedMatch> {
        crate::entry::parse_dimensions_for_editor(text, Some(self.ctx.clone()))
    }

    /// Completes input prefixes through this instance.
    pub fn complete(&self, text: &str) -> Vec<Completion> {
        crate::completion::complete(text, Some(self.ctx.clone()))
    }

    /// Completes canonical readings through this instance.
    pub fn complete_readings(&self, text: &str) -> Vec<CompletionReading> {
        crate::completion::complete_readings(text, Some(self.ctx.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn japanese_building_preset_keeps_only_its_measurement_domains() {
        let parser = Parser::japanese_building();

        for input in ["5尺3寸", "6帖", "延床100㎡", "100-120㎡"] {
            assert!(parser.parse(input).best.is_some(), "{input}");
        }

        for input in ["5 kg", "5 W", "5 mM", "1.5 cups"] {
            let parsed = parser.parse(input);
            assert!(parsed.best.is_none(), "{input}");
            assert!(parsed.alternatives.is_empty(), "{input}");
            assert!(parsed.suggestions.is_empty(), "{input}");
            assert!(!parsed.findings.skipped.is_empty(), "{input}");
        }
    }

    #[test]
    fn configured_registry_removes_a_cross_domain_competitor_before_ranking() {
        let parser = Parser::new(DimensionSet::from(Dimension::Length));

        let parsed = parser.parse("5m3");
        let best = parsed.best.expect("length reading");
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.value, Some(5.03));
        assert!(parsed.alternatives.is_empty());
        assert!(parsed.findings.skipped.is_empty());
        assert!(parsed.findings.ambiguities.is_empty());
    }

    #[test]
    fn unrestricted_is_an_explicit_choice() {
        let parsed = Parser::unrestricted().parse("5 kg");
        assert_eq!(
            parsed.best.and_then(|reading| reading.dimension),
            Some(Dimension::Mass)
        );
    }

    #[test]
    fn default_registry_is_the_small_building_domain_set() {
        let parser = Parser::default();

        assert!(parser.parse("5 m").best.is_some());
        assert!(parser.parse("6 m2").best.is_some());
        assert!(parser.parse("5 kg").best.is_none());
    }

    #[test]
    fn empty_configured_registry_loads_no_measurement_units() {
        let parser = Parser::new(DimensionSet::new());

        assert!(parser.parse("5 kg").best.is_none());
        assert!(parser.parse("5 m").best.is_none());
        assert!(parser.parse("3640").best.is_some());
    }

    #[test]
    fn configured_dimensions_skip_fixed_grammars_before_dispatch() {
        let parser = Parser::japanese_building();
        let (_, traces) = traced(|| parser.parse("1.5 cups"));
        let trace = traces.first().expect("one dispatch");

        for excluded in [
            Grammar::Mass,
            Grammar::Temperature,
            Grammar::Duration,
            Grammar::Cups,
            Grammar::Currency,
        ] {
            assert!(!trace.attempted.contains(&excluded), "{excluded:?}");
        }
    }
}
