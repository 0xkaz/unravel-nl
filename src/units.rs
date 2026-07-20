use crate::*;

pub(crate) const SHAKU_M: f64 = 10.0 / 33.0;
pub(crate) const SUN_M: f64 = 1.0 / 33.0;
pub(crate) const KEN_M: f64 = 60.0 / 33.0;
pub(crate) const TATAMI_M2: f64 = 1.62;
pub(crate) const TSUBO_M2: f64 = 400.0 / 121.0;
pub(crate) const CM_M: f64 = 0.01;
pub(crate) const FOOT_M: f64 = 0.3048;
pub(crate) const INCH_M: f64 = 0.0254;
pub(crate) const YARD_M: f64 = 0.9144;
pub(crate) const MILE_M: f64 = 1609.344;
pub(crate) const NAUTICAL_MILE_M: f64 = 1852.0;
pub(crate) const LIGHT_YEAR_M: f64 = 9_460_730_472_580_800.0;
pub(crate) const LB_KG: f64 = 0.453_592_37;
pub(crate) const OZ_KG: f64 = 0.028_349_523_125;
pub(crate) const STONE_KG: f64 = 6.350_293_18;
pub(crate) const GRAIN_KG: f64 = 0.000_064_798_91;
pub(crate) const TROY_OZ_KG: f64 = 0.031_103_476_8;
pub(crate) const CARAT_KG: f64 = 0.0002;
pub(crate) const US_GALLON_M3: f64 = 0.003_785_411_784;
pub(crate) const CUBIC_FOOT_M3: f64 = 0.028_316_846_592;
pub(crate) const US_CUP_L: f64 = 0.236_588_236_5;
pub(crate) const UK_CUP_L: f64 = 0.284_130_625;
pub(crate) const METRIC_CUP_L: f64 = 0.25;

pub fn unit_definitions() -> &'static [UnitDef] {
    UNIT_DEFS
}

pub fn units_of(dimension: Dimension) -> impl Iterator<Item = &'static UnitDef> {
    UNIT_DEFS
        .iter()
        .filter(move |unit| unit.dimension == dimension)
}

pub(crate) fn unit_by_alias(alias: &str) -> Option<&'static UnitDef> {
    let alias = alias.trim();
    if let Some(unit) = fast_unit_by_alias(alias, AliasMatchMode::Exact)
        .or_else(|| fallback_unit_by_alias(alias, AliasMatchMode::Exact))
    {
        return Some(unit);
    }

    let normalized;
    let lookup = {
        normalized = normalize_input(alias);
        normalized.trim()
    };
    if lookup != alias
        && let Some(unit) = fast_unit_by_alias(lookup, AliasMatchMode::Exact)
            .or_else(|| fallback_unit_by_alias(lookup, AliasMatchMode::Exact))
    {
        return Some(unit);
    }

    fast_unit_by_alias(alias, AliasMatchMode::AsciiCase)
        .or_else(|| fallback_unit_by_alias(alias, AliasMatchMode::AsciiCase))
        .or_else(|| {
            (lookup != alias)
                .then(|| {
                    fast_unit_by_alias(lookup, AliasMatchMode::AsciiCase)
                        .or_else(|| fallback_unit_by_alias(lookup, AliasMatchMode::AsciiCase))
                })
                .flatten()
        })
}

#[derive(Clone, Copy)]
pub(crate) enum AliasMatchMode {
    Exact,
    AsciiCase,
}

pub(crate) fn fast_unit_by_alias(alias: &str, mode: AliasMatchMode) -> Option<&'static UnitDef> {
    FAST_UNIT_ALIASES
        .iter()
        .find_map(|(candidate, unit_id)| alias_matches(candidate, alias, mode).then_some(*unit_id))
        .and_then(unit_by_id)
}

pub(crate) fn unit_by_id(id: &str) -> Option<&'static UnitDef> {
    UNIT_DEFS.iter().find(|unit| unit.id == id)
}

pub(crate) fn fallback_unit_by_alias(
    alias: &str,
    mode: AliasMatchMode,
) -> Option<&'static UnitDef> {
    UNIT_DEFS.iter().find(|unit| {
        unit_lookup_aliases(unit).any(|candidate| alias_matches(candidate.trim(), alias, mode))
    })
}

pub(crate) fn alias_matches(candidate: &str, alias: &str, mode: AliasMatchMode) -> bool {
    if candidate.len() != alias.len() || candidate.is_empty() {
        return false;
    }
    if candidate == alias {
        return true;
    }
    if matches!(mode, AliasMatchMode::Exact) {
        return false;
    }
    if !candidate.is_ascii() || !alias.is_ascii() {
        return false;
    }
    if candidate.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return false;
    }
    let candidate_first = candidate.as_bytes()[0];
    let alias_first = alias.as_bytes()[0];
    candidate_first.eq_ignore_ascii_case(&alias_first) && candidate.eq_ignore_ascii_case(alias)
}

pub(crate) fn split_once_ascii_case<'a>(text: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let idx = find_ascii_case(text, needle)?;
    let after = idx + needle.len();
    Some((text.get(..idx)?, text.get(after..)?))
}

pub(crate) fn find_ascii_case(text: &str, needle: &str) -> Option<usize> {
    let text = text.as_bytes();
    let needle = needle.as_bytes();
    if needle.is_empty() || needle.len() > text.len() {
        return None;
    }
    text.windows(needle.len()).position(|window| {
        window
            .iter()
            .zip(needle)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
    })
}

pub(crate) fn target_unit_for(dimension: Dimension, alias: &str) -> Option<&'static UnitDef> {
    let unit = unit_by_alias(alias)?;
    (unit.dimension == dimension).then_some(unit)
}

pub(crate) fn convert_registered_reading(source: &Reading, target_unit: &str) -> Option<Reading> {
    let dimension = source.dimension?;
    let target = target_unit_for(dimension, target_unit)?;
    Some(Reading::quantity(
        source.value? / target.factor,
        target.id,
        dimension,
        target.provenance,
        target.approximate,
        0.95,
    ))
}

pub(crate) fn unit_lookup_aliases(unit: &UnitDef) -> impl Iterator<Item = &'static str> + '_ {
    unit.aliases
        .iter()
        .copied()
        .chain(core::iter::once(unit.id))
}

pub(crate) fn custom_unit_lookup_aliases(unit: &CustomUnit) -> impl Iterator<Item = &str> {
    core::iter::once(unit.id.as_str()).chain(unit.aliases.iter().map(String::as_str))
}

pub(crate) fn exact_custom_alias(unit: &CustomUnit, alias: &str) -> bool {
    custom_unit_lookup_aliases(unit).any(|candidate| candidate == alias)
}

pub(crate) fn custom_unit_by_alias<'a>(alias: &str, ctx: &'a ParseCtx) -> Option<&'a CustomUnit> {
    let alias = alias.trim();
    ctx.custom_units
        .iter()
        .find(|unit| exact_custom_alias(unit, alias))
        .or_else(|| {
            ctx.custom_units.iter().find(|unit| {
                custom_unit_lookup_aliases(unit)
                    .any(|candidate| candidate.eq_ignore_ascii_case(alias))
            })
        })
}

pub(crate) fn normalize_alias(alias: &str) -> String {
    normalize_input(alias).trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    #[test]
    fn exposes_unit_registry_by_dimension() {
        let length_ids: Vec<&str> = units_of(Dimension::Length).map(|unit| unit.id).collect();
        assert!(length_ids.contains(&"m"));
        assert!(length_ids.contains(&"cm"));
        assert!(length_ids.contains(&"ft"));

        let mass_ids: Vec<&str> = units_of(Dimension::Mass).map(|unit| unit.id).collect();
        assert!(mass_ids.contains(&"kg"));
        assert!(mass_ids.contains(&"lb"));
    }

    #[test]
    fn parses_registry_aliases() {
        let parsed = parse("5 meters", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_close(best.value.unwrap(), 5.0);
    }

    #[test]
    fn parses_expanded_lingo_catalog_examples() {
        for (input, dimension, unit, expected) in [
            ("12 km", Dimension::Length, "m", 12_000.0),
            ("25 microns", Dimension::Length, "m", 0.000_025),
            ("6 yd", Dimension::Length, "m", 5.4864),
            ("2 miles", Dimension::Length, "m", 3218.688),
            ("1 nautical mile", Dimension::Length, "m", 1852.0),
            ("3 hands", Dimension::Length, "m", 0.3048),
            ("15 thou", Dimension::Length, "m", 0.000_381),
            ("12 st", Dimension::Mass, "kg", 76.203_518_16),
            ("5 grains", Dimension::Mass, "kg", 0.000_323_994_55),
            ("2 troy oz", Dimension::Mass, "kg", 0.062_206_953_6),
            ("3 carats", Dimension::Mass, "kg", 0.0006),
            ("500 mcg", Dimension::Mass, "kg", 0.000_000_5),
            ("2 tonnes", Dimension::Mass, "kg", 2000.0),
            ("250 mL", Dimension::Volume, "L", 0.25),
            ("12 fl. oz.", Dimension::Volume, "L", 0.354_882_354_75),
            ("50 sq ft", Dimension::Area, "m2", 4.645_152),
            ("5 acres", Dimension::Area, "m2", 20_234.282_112),
            ("60 mph", Dimension::Speed, "m/s", 26.8224),
            ("100 km/h", Dimension::Speed, "m/s", 27.777_777_777_777_78),
            ("500 GB", Dimension::Data, "B", 500_000_000_000.0),
            ("5 Mbps", Dimension::DataRate, "bit/s", 5_000_000.0),
            ("20 MB/s", Dimension::DataRate, "bit/s", 160_000_000.0),
            ("5 gpm", Dimension::FlowRate, "m3/s", 0.000_315_450_982),
            ("500 mAh", Dimension::Charge, "C", 1800.0),
            ("5 uM", Dimension::Concentration, "mol/m3", 0.005),
            ("9.8 m/s²", Dimension::Acceleration, "m/s2", 9.8),
            ("10 Nm", Dimension::Torque, "N*m", 10.0),
            ("500 lux", Dimension::Illuminance, "lx", 500.0),
            ("20 mSv", Dimension::RadiationEquivalentDose, "Sv", 0.02),
            ("5 MBq", Dimension::Radioactivity, "Bq", 5_000_000.0),
            ("10 inH₂O", Dimension::Pressure, "Pa", 2490.8891),
            ("1 kgf/cm²", Dimension::Pressure, "Pa", 98_066.5),
        ] {
            let best = parse(input, None).best.expect(input);
            assert_eq!(best.dimension, Some(dimension), "{input}");
            assert_eq!(best.unit.as_deref(), Some(unit), "{input}");
            assert_close(best.value.unwrap(), expected);
        }
    }

    #[test]
    fn converts_same_dimension_registry_units() {
        for (input, dimension, unit, expected) in [
            ("20 MB/s to Mbit/s", Dimension::DataRate, "Mbit/s", 160.0),
            (
                "5 gpm to L/min",
                Dimension::FlowRate,
                "L/min",
                18.927_058_92,
            ),
            ("500 mAh to C", Dimension::Charge, "C", 1800.0),
            ("5 uM to mol/m3", Dimension::Concentration, "mol/m3", 0.005),
            ("1 hp to W", Dimension::Power, "W", 745.699_872),
            ("10 inH2O to Pa", Dimension::Pressure, "Pa", 2490.8891),
            ("1 kgf/cm² to kPa", Dimension::Pressure, "kPa", 98.0665),
            (
                "1 fc to lx",
                Dimension::Illuminance,
                "lx",
                10.763_910_416_709_722,
            ),
        ] {
            let best = parse(input, None).best.expect(input);
            assert_eq!(best.dimension, Some(dimension), "{input}");
            assert_eq!(best.unit.as_deref(), Some(unit), "{input}");
            assert_close(best.value.unwrap(), expected);
        }
    }

    #[test]
    fn parses_custom_unit_registry_entries() {
        let ctx = ParseCtx {
            custom_units: vec![CustomUnit::new(
                "smoot",
                "m",
                &["smoot", "smoots"],
                Dimension::Length,
                1.7018,
            )],
            ..ParseCtx::default()
        };
        let parsed = parse("3 smoots", Some(ctx));
        let best = parsed.best.expect("custom unit");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_eq!(best.provenance, Some(Provenance::TradeCustom));
        assert_close(best.value.unwrap(), 5.1054);
    }

    #[test]
    fn uses_custom_unit_suffix_for_ranges() {
        let ctx = ParseCtx {
            custom_units: vec![CustomUnit::new(
                "smoot",
                "m",
                &["smoot", "smoots"],
                Dimension::Length,
                1.7018,
            )],
            ..ParseCtx::default()
        };
        let parsed = parse("2-3 smoots", Some(ctx));
        let best = parsed.best.expect("custom unit range");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_close(range.from.value.unwrap(), 3.4036);
        assert_close(range.to.value.unwrap(), 5.1054);
    }
}
