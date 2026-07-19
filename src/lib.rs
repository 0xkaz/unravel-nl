//! Deterministic parsing for informal natural-language quantities and values.
//!
//! This crate is an independent Rust implementation inspired by the public API
//! shape of `pascalorg/lingo` (MIT). It does not copy source code from that
//! project.

const SHAKU_M: f64 = 10.0 / 33.0;
const SUN_M: f64 = 1.0 / 33.0;
const KEN_M: f64 = 60.0 / 33.0;
const TATAMI_M2: f64 = 1.62;
const TSUBO_M2: f64 = 400.0 / 121.0;
const CM_M: f64 = 0.01;
const FOOT_M: f64 = 0.3048;
const INCH_M: f64 = 0.0254;
const YARD_M: f64 = 0.9144;
const MILE_M: f64 = 1609.344;
const NAUTICAL_MILE_M: f64 = 1852.0;
const LIGHT_YEAR_M: f64 = 9_460_730_472_580_800.0;
const LB_KG: f64 = 0.453_592_37;
const OZ_KG: f64 = 0.028_349_523_125;
const STONE_KG: f64 = 6.350_293_18;
const GRAIN_KG: f64 = 0.000_064_798_91;
const TROY_OZ_KG: f64 = 0.031_103_476_8;
const CARAT_KG: f64 = 0.0002;
const US_GALLON_M3: f64 = 0.003_785_411_784;
const CUBIC_FOOT_M3: f64 = 0.028_316_846_592;
const US_CUP_L: f64 = 0.236_588_236_5;
const UK_CUP_L: f64 = 0.284_130_625;
const METRIC_CUP_L: f64 = 0.25;

pub const CONTRACT_VERSION: &str = "unravel-nl.parse.v1";

pub fn contract_version() -> &'static str {
    CONTRACT_VERSION
}

pub fn parse_input_schema_json() -> &'static str {
    PARSE_INPUT_SCHEMA_JSON
}

pub fn parsed_output_schema_json() -> &'static str {
    PARSED_OUTPUT_SCHEMA_JSON
}

pub fn mcp_tool_schema_json() -> &'static str {
    MCP_TOOL_SCHEMA_JSON
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UnitDef {
    pub id: &'static str,
    pub canonical_unit: &'static str,
    pub aliases: &'static [&'static str],
    pub dimension: Dimension,
    pub factor: f64,
    pub provenance: Provenance,
    pub approximate: bool,
}

const UNIT_DEFS: &[UnitDef] = &[
    UnitDef {
        id: "m",
        canonical_unit: "m",
        aliases: &["m", "meter", "meters", "metre", "metres"],
        dimension: Dimension::Length,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "km",
        canonical_unit: "m",
        aliases: &[
            "km",
            "kilometer",
            "kilometers",
            "kilometre",
            "kilometres",
            "klick",
            "klicks",
        ],
        dimension: Dimension::Length,
        factor: 1000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "cm",
        canonical_unit: "m",
        aliases: &[
            "cm",
            "centimeter",
            "centimeters",
            "centimetre",
            "centimetres",
        ],
        dimension: Dimension::Length,
        factor: CM_M,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mm",
        canonical_unit: "m",
        aliases: &[
            "mm",
            "millimeter",
            "millimeters",
            "millimetre",
            "millimetres",
        ],
        dimension: Dimension::Length,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "μm",
        canonical_unit: "m",
        aliases: &[
            "μm",
            "µm",
            "um",
            "micron",
            "microns",
            "micrometer",
            "micrometers",
        ],
        dimension: Dimension::Length,
        factor: 0.000_001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "in",
        canonical_unit: "m",
        aliases: &["in", "inch", "inches", "\"", "″", "''"],
        dimension: Dimension::Length,
        factor: INCH_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "ft",
        canonical_unit: "m",
        aliases: &["ft", "ft.", "foot", "feet", "'", "′"],
        dimension: Dimension::Length,
        factor: FOOT_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "yd",
        canonical_unit: "m",
        aliases: &["yd", "yard", "yards"],
        dimension: Dimension::Length,
        factor: YARD_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "mi",
        canonical_unit: "m",
        aliases: &["mi", "mile", "miles"],
        dimension: Dimension::Length,
        factor: MILE_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "nmi",
        canonical_unit: "m",
        aliases: &["nmi", "nautical mile", "nautical miles"],
        dimension: Dimension::Length,
        factor: NAUTICAL_MILE_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "hand",
        canonical_unit: "m",
        aliases: &["hand", "hands"],
        dimension: Dimension::Length,
        factor: 4.0 * INCH_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "thou",
        canonical_unit: "m",
        aliases: &["thou", "mil", "mils"],
        dimension: Dimension::Length,
        factor: INCH_M / 1000.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "ly",
        canonical_unit: "m",
        aliases: &["ly", "light year", "light years"],
        dimension: Dimension::Length,
        factor: LIGHT_YEAR_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "fathom",
        canonical_unit: "m",
        aliases: &["fathom", "fathoms"],
        dimension: Dimension::Length,
        factor: 6.0 * FOOT_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "kg",
        canonical_unit: "kg",
        aliases: &[
            "kg",
            "kilogram",
            "kilograms",
            "kilogramme",
            "kilogrammes",
            "kilo",
            "kilos",
            "公斤",
            "キログラム",
            "キロ",
        ],
        dimension: Dimension::Mass,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "g",
        canonical_unit: "kg",
        aliases: &["g", "gram", "grams"],
        dimension: Dimension::Mass,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "lb",
        canonical_unit: "kg",
        aliases: &["lb", "lbs", "pound", "pounds", "#"],
        dimension: Dimension::Mass,
        factor: LB_KG,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "oz",
        canonical_unit: "kg",
        aliases: &["oz", "ounce", "ounces"],
        dimension: Dimension::Mass,
        factor: OZ_KG,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "st",
        canonical_unit: "kg",
        aliases: &["st", "stone", "stones"],
        dimension: Dimension::Mass,
        factor: STONE_KG,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "gr",
        canonical_unit: "kg",
        aliases: &["gr", "grain", "grains"],
        dimension: Dimension::Mass,
        factor: GRAIN_KG,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "troy oz",
        canonical_unit: "kg",
        aliases: &["troy oz", "troy ounce", "troy ounces"],
        dimension: Dimension::Mass,
        factor: TROY_OZ_KG,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "ct",
        canonical_unit: "kg",
        aliases: &["ct", "carat", "carats"],
        dimension: Dimension::Mass,
        factor: CARAT_KG,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "mcg",
        canonical_unit: "kg",
        aliases: &["mcg", "μg", "µg", "ug", "microgram", "micrograms"],
        dimension: Dimension::Mass,
        factor: 0.000_000_001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "t",
        canonical_unit: "kg",
        aliases: &["t", "tonne", "tonnes", "metric ton", "metric tons"],
        dimension: Dimension::Mass,
        factor: 1000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "s",
        canonical_unit: "s",
        aliases: &["s", "sec", "secs", "second", "seconds"],
        dimension: Dimension::Time,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "min",
        canonical_unit: "s",
        aliases: &["min", "mins", "minute", "minutes"],
        dimension: Dimension::Time,
        factor: 60.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "h",
        canonical_unit: "s",
        aliases: &["h", "hr", "hrs", "hour", "hours"],
        dimension: Dimension::Time,
        factor: 3600.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "day",
        canonical_unit: "s",
        aliases: &["d", "day", "days", "日"],
        dimension: Dimension::Time,
        factor: 86_400.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "week",
        canonical_unit: "s",
        aliases: &["week", "weeks"],
        dimension: Dimension::Time,
        factor: 604_800.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "fortnight",
        canonical_unit: "s",
        aliases: &["fortnight", "fortnights"],
        dimension: Dimension::Time,
        factor: 1_209_600.0,
        provenance: Provenance::TradeCustom,
        approximate: false,
    },
    UnitDef {
        id: "month",
        canonical_unit: "s",
        aliases: &["month", "months"],
        dimension: Dimension::Time,
        factor: 2_629_746.0,
        provenance: Provenance::TradeCustom,
        approximate: true,
    },
    UnitDef {
        id: "year",
        canonical_unit: "s",
        aliases: &["year", "years", "yr", "yrs"],
        dimension: Dimension::Time,
        factor: 31_556_952.0,
        provenance: Provenance::TradeCustom,
        approximate: true,
    },
    UnitDef {
        id: "m2",
        canonical_unit: "m2",
        aliases: &[
            "m2",
            "m^2",
            "m²",
            "㎡",
            "sqm",
            "square meter",
            "square meters",
            "square metre",
            "square metres",
            "平米",
        ],
        dimension: Dimension::Area,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "ft2",
        canonical_unit: "m2",
        aliases: &[
            "ft2",
            "ft^2",
            "ft²",
            "sq ft",
            "sqft",
            "square foot",
            "square feet",
        ],
        dimension: Dimension::Area,
        factor: FOOT_M * FOOT_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "acre",
        canonical_unit: "m2",
        aliases: &["acre", "acres"],
        dimension: Dimension::Area,
        factor: 4_046.856_422_4,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "ha",
        canonical_unit: "m2",
        aliases: &["ha", "hectare", "hectares"],
        dimension: Dimension::Area,
        factor: 10_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "L",
        canonical_unit: "L",
        aliases: &["L", "l", "liter", "liters", "litre", "litres"],
        dimension: Dimension::Volume,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mL",
        canonical_unit: "L",
        aliases: &[
            "mL",
            "ml",
            "milliliter",
            "milliliters",
            "millilitre",
            "millilitres",
        ],
        dimension: Dimension::Volume,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "cc",
        canonical_unit: "L",
        aliases: &["cc"],
        dimension: Dimension::Volume,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "m3",
        canonical_unit: "L",
        aliases: &[
            "m3",
            "m^3",
            "m³",
            "cbm",
            "cubic meter",
            "cubic meters",
            "cubic metre",
            "cubic metres",
        ],
        dimension: Dimension::Volume,
        factor: 1000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "tbsp",
        canonical_unit: "L",
        aliases: &["tbsp", "tablespoon", "tablespoons"],
        dimension: Dimension::Volume,
        factor: 0.014_786_764_781_25,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "tsp",
        canonical_unit: "L",
        aliases: &["tsp", "teaspoon", "teaspoons"],
        dimension: Dimension::Volume,
        factor: 0.004_928_921_593_75,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "fl oz",
        canonical_unit: "L",
        aliases: &["fl oz", "fl. oz.", "fluid ounce", "fluid ounces"],
        dimension: Dimension::Volume,
        factor: 0.029_573_529_562_5,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "pt",
        canonical_unit: "L",
        aliases: &["pt", "pint", "pints"],
        dimension: Dimension::Volume,
        factor: 0.473_176_473,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "qt",
        canonical_unit: "L",
        aliases: &["qt", "quart", "quarts"],
        dimension: Dimension::Volume,
        factor: 0.946_352_946,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "gal",
        canonical_unit: "L",
        aliases: &["gal", "gallon", "gallons"],
        dimension: Dimension::Volume,
        factor: 3.785_411_784,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "m/s",
        canonical_unit: "m/s",
        aliases: &[
            "m/s",
            "mps",
            "meter per second",
            "meters per second",
            "metre per second",
            "metres per second",
        ],
        dimension: Dimension::Speed,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "km/h",
        canonical_unit: "m/s",
        aliases: &[
            "km/h",
            "kph",
            "kmh",
            "kilometer per hour",
            "kilometers per hour",
        ],
        dimension: Dimension::Speed,
        factor: 1000.0 / 3600.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mph",
        canonical_unit: "m/s",
        aliases: &["mph", "mile per hour", "miles per hour"],
        dimension: Dimension::Speed,
        factor: MILE_M / 3600.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "kn",
        canonical_unit: "m/s",
        aliases: &["kn", "knot", "knots"],
        dimension: Dimension::Speed,
        factor: NAUTICAL_MILE_M / 3600.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "B",
        canonical_unit: "B",
        aliases: &["B", "byte", "bytes"],
        dimension: Dimension::Data,
        factor: 1.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "bit",
        canonical_unit: "B",
        aliases: &["bit", "bits", "b"],
        dimension: Dimension::Data,
        factor: 0.125,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "kB",
        canonical_unit: "B",
        aliases: &["kB", "KB", "kilobyte", "kilobytes"],
        dimension: Dimension::Data,
        factor: 1_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "MB",
        canonical_unit: "B",
        aliases: &["MB", "megabyte", "megabytes"],
        dimension: Dimension::Data,
        factor: 1_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "GB",
        canonical_unit: "B",
        aliases: &["GB", "gigabyte", "gigabytes", "gig", "gigs"],
        dimension: Dimension::Data,
        factor: 1_000_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "TB",
        canonical_unit: "B",
        aliases: &["TB", "terabyte", "terabytes"],
        dimension: Dimension::Data,
        factor: 1_000_000_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "Mbit",
        canonical_unit: "B",
        aliases: &["Mbit", "Mb", "megabit", "megabits"],
        dimension: Dimension::Data,
        factor: 125_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "TiB",
        canonical_unit: "B",
        aliases: &["TiB", "tebibyte", "tebibytes"],
        dimension: Dimension::Data,
        factor: 1_099_511_627_776.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "Mbit/s",
        canonical_unit: "bit/s",
        aliases: &[
            "Mbit/s",
            "Mbps",
            "mbps",
            "Mb/s",
            "megabit per second",
            "megabits per second",
        ],
        dimension: Dimension::DataRate,
        factor: 1_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "kbit/s",
        canonical_unit: "bit/s",
        aliases: &[
            "kbit/s",
            "kbps",
            "Kbps",
            "kb/s",
            "Kb/s",
            "kilobit/s",
            "kilobits/s",
        ],
        dimension: Dimension::DataRate,
        factor: 1_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "Gbit/s",
        canonical_unit: "bit/s",
        aliases: &[
            "Gbit/s",
            "Gbps",
            "gbps",
            "Gb/s",
            "gigabit per second",
            "gigabits per second",
        ],
        dimension: Dimension::DataRate,
        factor: 1_000_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "bit/s",
        canonical_unit: "bit/s",
        aliases: &["bit/s", "bits/s", "bit/sec", "bits/sec", "b/s"],
        dimension: Dimension::DataRate,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "MB/s",
        canonical_unit: "bit/s",
        aliases: &[
            "MB/s",
            "MBps",
            "megabyte per second",
            "megabytes per second",
        ],
        dimension: Dimension::DataRate,
        factor: 8_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "MiB/s",
        canonical_unit: "bit/s",
        aliases: &[
            "MiB/s",
            "MiBps",
            "mebibyte per second",
            "mebibytes per second",
        ],
        dimension: Dimension::DataRate,
        factor: 8_388_608.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "m3/s",
        canonical_unit: "m3/s",
        aliases: &[
            "m3/s",
            "m^3/s",
            "m³/s",
            "cubic meter per second",
            "cubic meters per second",
        ],
        dimension: Dimension::FlowRate,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "m3/h",
        canonical_unit: "m3/s",
        aliases: &["m3/h", "m^3/h", "m³/h", "m3 per hour", "m^3 per hour"],
        dimension: Dimension::FlowRate,
        factor: 1.0 / 3600.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "L/s",
        canonical_unit: "m3/s",
        aliases: &[
            "L/s",
            "l/s",
            "liter per second",
            "liters per second",
            "litre per second",
            "litres per second",
        ],
        dimension: Dimension::FlowRate,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "L/min",
        canonical_unit: "m3/s",
        aliases: &[
            "L/min",
            "l/min",
            "lpm",
            "liter per minute",
            "liters per minute",
            "litre per minute",
            "litres per minute",
        ],
        dimension: Dimension::FlowRate,
        factor: 0.001 / 60.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mL/min",
        canonical_unit: "m3/s",
        aliases: &[
            "mL/min",
            "ml/min",
            "mL per minute",
            "ml per minute",
            "cc/min",
            "cc per minute",
        ],
        dimension: Dimension::FlowRate,
        factor: 0.000_001 / 60.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "gpm",
        canonical_unit: "m3/s",
        aliases: &["gpm", "gal/min", "gallon per minute", "gallons per minute"],
        dimension: Dimension::FlowRate,
        factor: US_GALLON_M3 / 60.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "cfm",
        canonical_unit: "m3/s",
        aliases: &["cfm", "ft3/min", "ft^3/min", "cubic feet per minute"],
        dimension: Dimension::FlowRate,
        factor: CUBIC_FOOT_M3 / 60.0,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "mol/m3",
        canonical_unit: "mol/m3",
        aliases: &[
            "mol/m3",
            "mol/m^3",
            "mol/m³",
            "mole per cubic meter",
            "moles per cubic meter",
        ],
        dimension: Dimension::Concentration,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "μM",
        canonical_unit: "mol/m3",
        aliases: &["μM", "µM", "uM", "micromolar"],
        dimension: Dimension::Concentration,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mM",
        canonical_unit: "mol/m3",
        aliases: &["mM", "millimolar"],
        dimension: Dimension::Concentration,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "m/s2",
        canonical_unit: "m/s2",
        aliases: &[
            "m/s2",
            "m/s^2",
            "m/s²",
            "meter per second squared",
            "meters per second squared",
        ],
        dimension: Dimension::Acceleration,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "g0",
        canonical_unit: "m/s2",
        aliases: &["g0", "gee", "gees"],
        dimension: Dimension::Acceleration,
        factor: 9.806_65,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "N",
        canonical_unit: "N",
        aliases: &["N", "newton", "newtons"],
        dimension: Dimension::Force,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "kgf",
        canonical_unit: "N",
        aliases: &["kgf", "kilogram force", "kilogram-force"],
        dimension: Dimension::Force,
        factor: 9.806_65,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "lbf",
        canonical_unit: "N",
        aliases: &["lbf", "pound force", "pound-force"],
        dimension: Dimension::Force,
        factor: 4.448_221_615_260_5,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "N*m",
        canonical_unit: "N*m",
        aliases: &[
            "N*m",
            "N m",
            "N·m",
            "N⋅m",
            "Nm",
            "newton meter",
            "newton meters",
            "newton metre",
            "newton metres",
        ],
        dimension: Dimension::Torque,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "lbf*ft",
        canonical_unit: "N*m",
        aliases: &["lbf*ft", "lbf ft", "lbf·ft", "lb-ft", "pound force foot"],
        dimension: Dimension::Torque,
        factor: 4.448_221_615_260_5 * FOOT_M,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "Pa",
        canonical_unit: "Pa",
        aliases: &["Pa", "pascal", "pascals"],
        dimension: Dimension::Pressure,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "kPa",
        canonical_unit: "Pa",
        aliases: &["kPa", "kilopascal", "kilopascals"],
        dimension: Dimension::Pressure,
        factor: 1000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "psi",
        canonical_unit: "Pa",
        aliases: &["psi", "pounds per square inch", "pound per square inch"],
        dimension: Dimension::Pressure,
        factor: 6_894.757_293_168,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "inH2O",
        canonical_unit: "Pa",
        aliases: &["inH2O", "inH₂O", "in H2O", "in water", "iwc"],
        dimension: Dimension::Pressure,
        factor: 249.088_91,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "kgf/cm2",
        canonical_unit: "Pa",
        aliases: &[
            "kgf/cm2",
            "kgf/cm^2",
            "kgf/cm²",
            "kgf per square centimeter",
        ],
        dimension: Dimension::Pressure,
        factor: 98_066.5,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "W",
        canonical_unit: "W",
        aliases: &["W", "watt", "watts"],
        dimension: Dimension::Power,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "hp",
        canonical_unit: "W",
        aliases: &["hp", "horsepower", "mechanical horsepower"],
        dimension: Dimension::Power,
        factor: 745.699_872,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "V",
        canonical_unit: "V",
        aliases: &["V", "volt", "volts"],
        dimension: Dimension::Voltage,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "A",
        canonical_unit: "A",
        aliases: &["A", "amp", "amps", "ampere", "amperes"],
        dimension: Dimension::Current,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mA",
        canonical_unit: "A",
        aliases: &["mA", "milliamp", "milliamps", "milliampere", "milliamperes"],
        dimension: Dimension::Current,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "Ω",
        canonical_unit: "Ω",
        aliases: &["Ω", "ohm", "ohms"],
        dimension: Dimension::Resistance,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "C",
        canonical_unit: "C",
        aliases: &["coulomb", "coulombs"],
        dimension: Dimension::Charge,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mAh",
        canonical_unit: "C",
        aliases: &[
            "mAh",
            "mah",
            "milliamp hour",
            "milliamp hours",
            "milliampere hour",
            "milliampere hours",
        ],
        dimension: Dimension::Charge,
        factor: 3.6,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "lx",
        canonical_unit: "lx",
        aliases: &["lx", "lux"],
        dimension: Dimension::Illuminance,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "fc",
        canonical_unit: "lx",
        aliases: &[
            "fc",
            "foot-candle",
            "foot-candles",
            "foot candle",
            "foot candles",
        ],
        dimension: Dimension::Illuminance,
        factor: 10.763_910_416_709_722,
        provenance: Provenance::InternationalExact,
        approximate: false,
    },
    UnitDef {
        id: "Sv",
        canonical_unit: "Sv",
        aliases: &["Sv", "sievert", "sieverts"],
        dimension: Dimension::RadiationEquivalentDose,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "mSv",
        canonical_unit: "Sv",
        aliases: &["mSv", "millisievert", "millisieverts"],
        dimension: Dimension::RadiationEquivalentDose,
        factor: 0.001,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "Bq",
        canonical_unit: "Bq",
        aliases: &["Bq", "becquerel", "becquerels"],
        dimension: Dimension::Radioactivity,
        factor: 1.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "MBq",
        canonical_unit: "Bq",
        aliases: &["MBq", "megabecquerel", "megabecquerels"],
        dimension: Dimension::Radioactivity,
        factor: 1_000_000.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
];

const PARSE_INPUT_SCHEMA_JSON: &str = r#"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://0xkaz.github.io/unravel-nl/schema/parse-input.json",
  "title": "unravel-nl parse input",
  "type": "object",
  "additionalProperties": false,
  "required": ["text"],
  "properties": {
    "text": {
      "type": "string",
      "minLength": 1,
      "description": "Informal or ambiguous natural-language value to parse."
    },
    "locale": {
      "type": "string",
      "description": "Optional BCP-47 style locale hint such as ja, en, en-US, or en-GB."
    },
    "expect": {
      "type": "string",
      "enum": ["quantity", "date", "range", "number"],
      "description": "Optional expected top-level reading kind."
    },
    "expected_dimension": {
      "type": "string",
      "enum": ["length", "area", "mass", "time", "volume", "currency", "temperature", "speed", "data", "data_rate", "flow_rate", "concentration", "acceleration", "force", "torque", "pressure", "power", "charge", "voltage", "current", "resistance", "illuminance", "radiation_equivalent_dose", "radioactivity"],
      "description": "Optional expected quantity dimension."
    },
    "reference_date": {
      "type": "string",
      "format": "date",
      "description": "Civil reference date for relative dates."
    },
    "strictness": {
      "type": "string",
      "enum": ["forgiving", "confirm", "strict"],
      "default": "forgiving"
    }
  }
}"#;

const PARSED_OUTPUT_SCHEMA_JSON: &str = r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://0xkaz.github.io/unravel-nl/schema/parsed-output.json",
  "title": "unravel-nl parsed output",
  "type": "object",
  "additionalProperties": false,
  "required": ["input", "best", "alternatives", "suggestions", "findings"],
  "properties": {
    "input": { "type": "string" },
    "locale": { "type": ["string", "null"] },
    "best": {
      "anyOf": [
        { "$ref": "#/$defs/reading" },
        { "type": "null" }
      ]
    },
    "alternatives": {
      "type": "array",
      "items": { "$ref": "#/$defs/reading" }
    },
    "suggestions": {
      "type": "array",
      "items": { "$ref": "#/$defs/suggestion" }
    },
    "findings": { "$ref": "#/$defs/findings" }
  },
  "$defs": {
    "kind": { "type": "string", "enum": ["quantity", "date", "range", "number"] },
    "dimension": {
      "type": "string",
      "enum": ["length", "area", "mass", "time", "volume", "currency", "temperature", "speed", "data", "data_rate", "flow_rate", "concentration", "acceleration", "force", "torque", "pressure", "power", "charge", "voltage", "current", "resistance", "illuminance", "radiation_equivalent_dose", "radioactivity"]
    },
    "provenance": {
      "type": "string",
      "enum": ["international_exact", "japanese_statute", "trade_custom", "si_multiple"]
    },
    "reading": {
      "type": "object",
      "additionalProperties": false,
      "required": ["kind"],
      "properties": {
        "kind": { "$ref": "#/$defs/kind" },
        "value": { "type": ["number", "null"] },
        "unit": { "type": ["string", "null"] },
        "dimension": {
          "anyOf": [
            { "$ref": "#/$defs/dimension" },
            { "type": "null" }
          ]
        },
        "date": { "type": ["string", "null"], "format": "date" },
        "range": {
          "anyOf": [
            { "$ref": "#/$defs/range" },
            { "type": "null" }
          ]
        },
        "provenance": {
          "anyOf": [
            { "$ref": "#/$defs/provenance" },
            { "type": "null" }
          ]
        },
        "approximate": { "type": ["boolean", "null"] },
        "confidence": { "type": ["number", "null"], "minimum": 0, "maximum": 1 }
      }
    },
    "range": {
      "type": "object",
      "additionalProperties": false,
      "required": ["from", "to"],
      "properties": {
        "from": { "$ref": "#/$defs/reading" },
        "to": { "$ref": "#/$defs/reading" }
      }
    },
    "suggestion": {
      "type": "object",
      "additionalProperties": false,
      "required": ["from", "to"],
      "properties": {
        "from": { "type": "string" },
        "to": { "type": "string" },
        "score": { "type": ["number", "null"], "minimum": 0, "maximum": 1 }
      }
    },
    "span": {
      "type": "object",
      "additionalProperties": false,
      "required": ["start", "end", "text"],
      "properties": {
        "start": { "type": "integer", "minimum": 0 },
        "end": { "type": "integer", "minimum": 0 },
        "text": { "type": "string" }
      }
    },
    "issue": {
      "type": "object",
      "additionalProperties": true,
      "required": ["code", "ref_text", "reason", "span"],
      "properties": {
        "code": { "type": "string" },
        "ref_text": { "type": "string" },
        "reason": { "type": "string" },
        "span": { "$ref": "#/$defs/span" }
      }
    },
    "findings": {
      "type": "object",
      "additionalProperties": false,
      "required": ["skipped", "ambiguities", "approximations"],
      "properties": {
        "skipped": { "type": "array", "items": { "$ref": "#/$defs/issue" } },
        "ambiguities": { "type": "array", "items": { "$ref": "#/$defs/issue" } },
        "approximations": { "type": "array", "items": { "$ref": "#/$defs/issue" } }
      }
    }
  }
}"##;

const MCP_TOOL_SCHEMA_JSON: &str = r#"{
  "name": "unravel_nl_parse",
  "description": "Parse informal or ambiguous natural-language quantities, dates, ranges, and values into deterministic canonical readings.",
  "inputSchema": {
    "$ref": "https://0xkaz.github.io/unravel-nl/schema/parse-input.json"
  },
  "outputSchema": {
    "$ref": "https://0xkaz.github.io/unravel-nl/schema/parsed-output.json"
  }
}"#;

pub fn unit_definitions() -> &'static [UnitDef] {
    UNIT_DEFS
}

pub fn units_of(dimension: Dimension) -> impl Iterator<Item = &'static UnitDef> {
    UNIT_DEFS
        .iter()
        .filter(move |unit| unit.dimension == dimension)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionKind {
    Unit,
    Date,
    Time,
    Currency,
}

impl CompletionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Date => "date",
            Self::Time => "time",
            Self::Currency => "currency",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Locale {
    Ja,
    En,
    EnUs,
    EnGb,
    Other(String),
}

impl Locale {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Ja => "ja",
            Self::En => "en",
            Self::EnUs => "en-US",
            Self::EnGb => "en-GB",
            Self::Other(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Quantity,
    Date,
    Range,
    Number,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dimension {
    Length,
    Area,
    Mass,
    Time,
    Volume,
    Currency,
    Temperature,
    Speed,
    Data,
    DataRate,
    FlowRate,
    Concentration,
    Acceleration,
    Force,
    Torque,
    Pressure,
    Power,
    Charge,
    Voltage,
    Current,
    Resistance,
    Illuminance,
    RadiationEquivalentDose,
    Radioactivity,
}

impl Dimension {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Length => "length",
            Self::Area => "area",
            Self::Mass => "mass",
            Self::Time => "time",
            Self::Volume => "volume",
            Self::Currency => "currency",
            Self::Temperature => "temperature",
            Self::Speed => "speed",
            Self::Data => "data",
            Self::DataRate => "data_rate",
            Self::FlowRate => "flow_rate",
            Self::Concentration => "concentration",
            Self::Acceleration => "acceleration",
            Self::Force => "force",
            Self::Torque => "torque",
            Self::Pressure => "pressure",
            Self::Power => "power",
            Self::Charge => "charge",
            Self::Voltage => "voltage",
            Self::Current => "current",
            Self::Resistance => "resistance",
            Self::Illuminance => "illuminance",
            Self::RadiationEquivalentDose => "radiation_equivalent_dose",
            Self::Radioactivity => "radioactivity",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Strictness {
    #[default]
    Forgiving,
    Confirm,
    Strict,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provenance {
    InternationalExact,
    JapaneseStatute,
    TradeCustom,
    SiMultiple,
}

impl Provenance {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InternationalExact => "international_exact",
            Self::JapaneseStatute => "japanese_statute",
            Self::TradeCustom => "trade_custom",
            Self::SiMultiple => "si_multiple",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if (1..=12).contains(&month) && (1..=31).contains(&day) {
            Some(Self { year, month, day })
        } else {
            None
        }
    }

    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseCtx {
    pub locale: Option<Locale>,
    pub expect: Option<Kind>,
    pub expected_dimension: Option<Dimension>,
    pub reference_date: Option<Date>,
    pub strictness: Strictness,
    pub currency_rates: Vec<CurrencyRate>,
    pub custom_units: Vec<CustomUnit>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CurrencyRate {
    pub from: String,
    pub to: String,
    pub factor: f64,
}

impl CurrencyRate {
    pub fn new(from: &str, to: &str, factor: f64) -> Self {
        Self {
            from: normalize_currency_code(from).to_owned(),
            to: normalize_currency_code(to).to_owned(),
            factor,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CustomUnit {
    pub id: String,
    pub canonical_unit: String,
    pub aliases: Vec<String>,
    pub dimension: Dimension,
    pub factor: f64,
    pub approximate: bool,
}

impl CustomUnit {
    pub fn new(
        id: &str,
        canonical_unit: &str,
        aliases: &[&str],
        dimension: Dimension,
        factor: f64,
    ) -> Self {
        Self {
            id: id.to_owned(),
            canonical_unit: canonical_unit.to_owned(),
            aliases: aliases.iter().map(|alias| (*alias).to_owned()).collect(),
            dimension,
            factor,
            approximate: false,
        }
    }

    pub fn approximate(mut self, approximate: bool) -> Self {
        self.approximate = approximate;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct HumanizeCtx {
    pub locale: Option<Locale>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub input: String,
    pub locale: Option<Locale>,
    pub best: Option<Reading>,
    pub alternatives: Vec<Reading>,
    pub suggestions: Vec<Suggestion>,
    pub findings: Findings,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub kind: Kind,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub dimension: Option<Dimension>,
    pub date: Option<String>,
    pub range: Option<Box<RangeReading>>,
    pub provenance: Option<Provenance>,
    pub approximate: Option<bool>,
    pub confidence: Option<f64>,
}

impl Reading {
    pub fn quantity(
        value: f64,
        unit: &str,
        dimension: Dimension,
        provenance: Provenance,
        approximate: bool,
        confidence: f64,
    ) -> Self {
        Self {
            kind: Kind::Quantity,
            value: Some(value),
            unit: Some(unit.to_owned()),
            dimension: Some(dimension),
            date: None,
            range: None,
            provenance: Some(provenance),
            approximate: Some(approximate),
            confidence: Some(confidence),
        }
    }

    pub fn number(value: f64, confidence: f64) -> Self {
        Self {
            kind: Kind::Number,
            value: Some(value),
            unit: None,
            dimension: None,
            date: None,
            range: None,
            provenance: None,
            approximate: Some(false),
            confidence: Some(confidence),
        }
    }

    pub fn date(date: Date, confidence: f64) -> Self {
        Self {
            kind: Kind::Date,
            value: None,
            unit: None,
            dimension: None,
            date: Some(date.iso()),
            range: None,
            provenance: None,
            approximate: Some(false),
            confidence: Some(confidence),
        }
    }

    pub fn range(from: Reading, to: Reading, confidence: f64) -> Self {
        Self {
            kind: Kind::Range,
            value: None,
            unit: None,
            dimension: None,
            date: None,
            range: Some(Box::new(RangeReading { from, to })),
            provenance: None,
            approximate: Some(false),
            confidence: Some(confidence),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RangeReading {
    pub from: Reading,
    pub to: Reading,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    pub from: String,
    pub to: String,
    pub score: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    pub value: String,
    pub canonical: Option<String>,
    pub kind: CompletionKind,
    pub dimension: Option<Dimension>,
    pub score: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueCode {
    Empty,
    NoValue,
    UnknownUnit,
    TypoCorrected,
    UnitAssumed,
    AmbiguousNumber,
    AmbiguousDate,
    AmbiguousUnit,
    AmbiguousCurrency,
    Approximation,
}

impl IssueCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "EMPTY",
            Self::NoValue => "NO_VALUE",
            Self::UnknownUnit => "UNKNOWN_UNIT",
            Self::TypoCorrected => "TYPO_CORRECTED",
            Self::UnitAssumed => "UNIT_ASSUMED",
            Self::AmbiguousNumber => "AMBIGUOUS_NUMBER",
            Self::AmbiguousDate => "AMBIGUOUS_DATE",
            Self::AmbiguousUnit => "AMBIGUOUS_UNIT",
            Self::AmbiguousCurrency => "AMBIGUOUS_CURRENCY",
            Self::Approximation => "APPROXIMATION",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Findings {
    pub skipped: Vec<Skipped>,
    pub ambiguities: Vec<Ambiguity>,
    pub approximations: Vec<Approximation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Skipped {
    pub code: IssueCode,
    pub ref_text: String,
    pub reason: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ambiguity {
    pub code: IssueCode,
    pub ref_text: String,
    pub reason: String,
    pub candidate_count: Option<usize>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Approximation {
    pub code: IssueCode,
    pub ref_text: String,
    pub reason: String,
    pub relative_error: Option<f64>,
    pub span: Span,
}

struct AmbiguousParse {
    best: Option<Reading>,
    alternatives: Vec<Reading>,
    ambiguity: Ambiguity,
}

pub fn parse(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let input = text.to_owned();
    let trimmed = text.trim();
    let mut parsed = Parsed {
        input,
        locale: ctx.locale.clone(),
        best: None,
        alternatives: Vec::new(),
        suggestions: Vec::new(),
        findings: Findings::default(),
    };

    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }

    if let Some(ambiguous) = parse_ambiguous_slash_date_or_fraction(trimmed, &ctx) {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
        return parsed;
    }

    if let Some(reading) = parse_relative_date(trimmed, &ctx) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_range(trimmed, &ctx) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_conversion_request(trimmed, &ctx) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_japanese_length(trimmed) {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Japanese customary length converted to SI meters.",
        ));
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_tatami_area(trimmed) {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Tatami area uses a trade-custom regional approximation of 1.62 m2.",
        ));
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_tsubo_area(trimmed) {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Tsubo area converted through Japanese customary area.",
        ));
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_square_meter(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_temperature(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_registered_quantity(trimmed, &ctx) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_metric_length(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_mass(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_clock_time(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_duration(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some(reading) = parse_feet_inches(trimmed) {
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some((best, alternatives, ambiguity)) = parse_cups(trimmed, &ctx) {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        parsed.findings.ambiguities.push(ambiguity);
        return parsed;
    }

    if let Some((best, alternatives, ambiguity)) = parse_currency(trimmed) {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        if let Some(ambiguity) = ambiguity {
            parsed.findings.ambiguities.push(ambiguity);
        }
        return parsed;
    }

    if let Some(ambiguous) = parse_ambiguous_number(trimmed) {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
        return parsed;
    }

    if let Some(reading) = parse_plain_number(trimmed) {
        if ctx.expect == Some(Kind::Quantity) || ctx.expected_dimension == Some(Dimension::Length) {
            parsed.alternatives.push(Reading::quantity(
                reading.value.unwrap_or_default(),
                "mm",
                Dimension::Length,
                Provenance::SiMultiple,
                false,
                0.41,
            ));
            parsed.findings.ambiguities.push(ambiguity(
                trimmed,
                "Plain number could be unitless or a context-implied millimeter length.",
                Some(2),
                IssueCode::UnitAssumed,
            ));
        }
        parsed.best = Some(reading);
        return parsed;
    }

    if let Some((reading, suggestion, unit_text)) = parse_typo_corrected_quantity(trimmed) {
        parsed.suggestions.push(suggestion);
        match ctx.strictness {
            Strictness::Forgiving => {
                parsed.findings.ambiguities.push(ambiguity_with_span(
                    &unit_text,
                    "Unit spelling was corrected by did-you-mean matching.",
                    Some(1),
                    IssueCode::TypoCorrected,
                    span_token_in(trimmed, &unit_text),
                ));
                parsed.best = Some(reading);
            }
            Strictness::Confirm | Strictness::Strict => {
                parsed.findings.skipped.push(skipped_with_span(
                    &unit_text,
                    "unit spelling correction requires confirmation",
                    IssueCode::TypoCorrected,
                    span_token_in(trimmed, &unit_text),
                ));
            }
        }
        return parsed;
    }

    parsed.suggestions = suggestions_for(trimmed);
    parsed
        .findings
        .skipped
        .push(skipped(trimmed, "no supported reading matched"));
    parsed
}

pub fn complete(prefix: &str, ctx: Option<ParseCtx>) -> Vec<Completion> {
    let ctx = ctx.unwrap_or_default();
    let Some(raw_prefix) = completion_prefix(prefix) else {
        return Vec::new();
    };
    let normalized_prefix = normalize_alias(raw_prefix);
    if normalized_prefix.is_empty() {
        return Vec::new();
    }

    let mut completions = Vec::new();
    for unit in UNIT_DEFS {
        for alias in unit.aliases {
            push_completion(
                &mut completions,
                CompletionCandidate {
                    value: alias,
                    canonical: Some(unit.id),
                    kind: CompletionKind::Unit,
                    dimension: Some(unit.dimension),
                },
                &normalized_prefix,
                &ctx,
            );
        }
    }

    for custom in &ctx.custom_units {
        for alias in &custom.aliases {
            push_owned_completion(
                &mut completions,
                alias,
                Some(&custom.id),
                CompletionKind::Unit,
                Some(custom.dimension),
                &normalized_prefix,
                &ctx,
            );
        }
    }

    for (value, canonical, dimension) in LEGACY_UNIT_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: Some(canonical),
                kind: CompletionKind::Unit,
                dimension: Some(*dimension),
            },
            &normalized_prefix,
            &ctx,
        );
    }

    for value in DATE_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: None,
                kind: CompletionKind::Date,
                dimension: None,
            },
            &normalized_prefix,
            &ctx,
        );
    }

    for value in TIME_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: None,
                kind: CompletionKind::Time,
                dimension: Some(Dimension::Time),
            },
            &normalized_prefix,
            &ctx,
        );
    }

    for (value, canonical) in CURRENCY_COMPLETIONS {
        push_completion(
            &mut completions,
            CompletionCandidate {
                value,
                canonical: Some(canonical),
                kind: CompletionKind::Currency,
                dimension: Some(Dimension::Currency),
            },
            &normalized_prefix,
            &ctx,
        );
    }

    completions.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.value.cmp(&right.value))
            .then_with(|| left.kind.as_str().cmp(right.kind.as_str()))
    });
    completions.truncate(24);
    completions
}

pub fn humanize(value: &Reading, ctx: Option<HumanizeCtx>) -> String {
    let locale = ctx.and_then(|ctx| ctx.locale);
    match (locale, value.kind, value.value, value.unit.as_deref()) {
        (Some(Locale::Ja), Kind::Quantity, Some(meters), Some("m")) => {
            humanize_japanese_length(meters)
        }
        (Some(Locale::Ja), Kind::Quantity, Some(area), Some("m2")) => humanize_japanese_area(area),
        (_, Kind::Quantity, Some(number), Some(unit))
            if value.dimension == Some(Dimension::Currency) =>
        {
            format!("{unit} {}", format_number(number))
        }
        (Some(Locale::Ja), Kind::Quantity, Some(number), Some("C"))
            if value.dimension == Some(Dimension::Temperature) =>
        {
            format!("摂氏{}度", format_number(number))
        }
        (_, Kind::Quantity, Some(number), Some("C"))
            if value.dimension == Some(Dimension::Temperature) =>
        {
            format!("{} °C", format_number(number))
        }
        (_, Kind::Quantity, Some(number), Some(unit)) => {
            format!("{} {}", format_number(number), unit)
        }
        (_, Kind::Number, Some(number), _) => format_number(number),
        (_, Kind::Date, _, _) => value
            .date
            .clone()
            .unwrap_or_else(|| "unknown date".to_owned()),
        (_, Kind::Range, _, _) => value.range.as_ref().map_or_else(
            || "unresolved range".to_owned(),
            |range| {
                format!(
                    "{} to {}",
                    humanize(&range.from, None),
                    humanize(&range.to, None)
                )
            },
        ),
        _ => "unresolved".to_owned(),
    }
}

fn parse_japanese_length(text: &str) -> Option<Reading> {
    let mut number = String::new();
    let mut meters = 0.0;
    let mut saw_unit = false;

    for ch in text.chars().filter(|ch| !ch.is_whitespace()) {
        if ch.is_ascii_digit() || ch == '.' || ch == ',' {
            number.push(ch);
            continue;
        }

        let value = parse_number(&number)?;
        number.clear();
        match ch {
            '尺' => {
                meters += value * SHAKU_M;
                saw_unit = true;
            }
            '寸' => {
                meters += value * SUN_M;
                saw_unit = true;
            }
            '間' => {
                meters += value * KEN_M;
                saw_unit = true;
            }
            _ => return None,
        }
    }

    if saw_unit && number.is_empty() {
        Some(Reading::quantity(
            meters,
            "m",
            Dimension::Length,
            Provenance::JapaneseStatute,
            true,
            0.98,
        ))
    } else {
        None
    }
}

fn parse_tatami_area(text: &str) -> Option<Reading> {
    let suffix = if text.ends_with('帖') {
        "帖"
    } else if text.ends_with('畳') {
        "畳"
    } else {
        return None;
    };
    let number_text = text.trim_end_matches(suffix);
    let value = parse_number(number_text.trim())?;
    Some(Reading::quantity(
        value * TATAMI_M2,
        "m2",
        Dimension::Area,
        Provenance::TradeCustom,
        true,
        0.94,
    ))
}

fn parse_tsubo_area(text: &str) -> Option<Reading> {
    let number_text = text.strip_suffix("坪")?;
    let value = parse_number(number_text.trim())?;
    Some(Reading::quantity(
        value * TSUBO_M2,
        "m2",
        Dimension::Area,
        Provenance::JapaneseStatute,
        true,
        0.94,
    ))
}

fn parse_square_meter(text: &str) -> Option<Reading> {
    let stripped = text
        .strip_prefix("延床")
        .or_else(|| text.strip_prefix("延べ床"))
        .unwrap_or(text)
        .trim();

    for suffix in ["㎡", "m2", "m^2", "平米"] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value,
                "m2",
                Dimension::Area,
                Provenance::SiMultiple,
                false,
                0.99,
            ));
        }
    }
    None
}

fn parse_temperature(text: &str) -> Option<Reading> {
    let stripped = text.trim();
    if let Some(value) = stripped
        .strip_prefix("摂氏")
        .and_then(|rest| rest.strip_suffix('度'))
        .and_then(parse_number)
    {
        return Some(temperature_celsius(value, 0.95));
    }
    if let Some(value) = stripped
        .strip_prefix("華氏")
        .and_then(|rest| rest.strip_suffix('度'))
        .and_then(parse_number)
    {
        return Some(temperature_celsius(fahrenheit_to_celsius(value), 0.95));
    }

    for suffix in [
        "degrees celsius",
        "degree celsius",
        "celsius",
        "°c",
        "℃",
        "c",
    ] {
        if let Some(value) = strip_suffix_ascii_case(stripped, suffix).and_then(parse_number) {
            return Some(temperature_celsius(value, 0.95));
        }
    }
    for suffix in [
        "degrees fahrenheit",
        "degree fahrenheit",
        "fahrenheit",
        "°f",
        "℉",
        "f",
    ] {
        if let Some(value) = strip_suffix_ascii_case(stripped, suffix).and_then(parse_number) {
            return Some(temperature_celsius(fahrenheit_to_celsius(value), 0.93));
        }
    }
    for suffix in ["kelvin", "kelvins", "k"] {
        if let Some(value) = strip_suffix_ascii_case(stripped, suffix).and_then(parse_number) {
            return Some(temperature_celsius(value - 273.15, 0.93));
        }
    }

    None
}

fn temperature_celsius(value: f64, confidence: f64) -> Reading {
    Reading::quantity(
        value,
        "C",
        Dimension::Temperature,
        Provenance::InternationalExact,
        false,
        confidence,
    )
}

fn fahrenheit_to_celsius(value: f64) -> f64 {
    (value - 32.0) * 5.0 / 9.0
}

fn strip_suffix_ascii_case<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    let prefix_len = text.len().checked_sub(suffix.len())?;
    let prefix = text.get(..prefix_len)?;
    let actual_suffix = text.get(prefix_len..)?;
    actual_suffix
        .eq_ignore_ascii_case(suffix)
        .then_some(prefix.trim())
}

fn parse_registered_quantity(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (number_text, unit_text) = split_number_unit(text)?;
    let value = parse_number(number_text)?;
    if let Some(unit) = unit_by_alias(unit_text) {
        return Some(Reading::quantity(
            value * unit.factor,
            unit.canonical_unit,
            unit.dimension,
            unit.provenance,
            unit.approximate,
            0.98,
        ));
    }
    let unit = custom_unit_by_alias(unit_text, ctx)?;
    Some(Reading::quantity(
        value * unit.factor,
        &unit.canonical_unit,
        unit.dimension,
        Provenance::TradeCustom,
        unit.approximate,
        0.93,
    ))
}

fn parse_typo_corrected_quantity(text: &str) -> Option<(Reading, Suggestion, String)> {
    let (number_text, unit_text) = split_number_unit(text)?;
    let value = parse_number(number_text)?;
    let suggestion = suggest_unit(unit_text)?;
    let corrected = unit_by_alias(&suggestion.to)?;
    let reading = Reading::quantity(
        value * corrected.factor,
        corrected.canonical_unit,
        corrected.dimension,
        corrected.provenance,
        corrected.approximate,
        0.82,
    );
    Some((reading, suggestion, unit_text.to_owned()))
}

fn split_number_unit(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    let mut seen_digit = false;
    for (idx, ch) in trimmed.char_indices() {
        if ch.is_ascii_digit() {
            seen_digit = true;
            continue;
        }
        if matches!(ch, '+' | '-') && idx == 0 {
            continue;
        }
        if seen_digit && matches!(ch, '.' | ',' | '_' | '/' | '½' | '¼' | '¾') {
            continue;
        }
        if seen_digit {
            let (number_text, unit_text) = trimmed.split_at(idx);
            let unit_text = unit_text.trim();
            if !unit_text.is_empty() && parse_number(number_text.trim()).is_some() {
                return Some((number_text.trim(), unit_text));
            }
        }
        return None;
    }
    None
}

fn unit_by_alias(alias: &str) -> Option<&'static UnitDef> {
    let alias = alias.trim();
    UNIT_DEFS
        .iter()
        .find(|unit| unit_lookup_aliases(unit).any(|candidate| candidate == alias))
        .or_else(|| {
            UNIT_DEFS.iter().find(|unit| {
                unit_lookup_aliases(unit).any(|candidate| candidate.eq_ignore_ascii_case(alias))
            })
        })
}

fn split_once_ascii_case<'a>(text: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let idx = find_ascii_case(text, needle)?;
    let after = idx + needle.len();
    Some((text.get(..idx)?, text.get(after..)?))
}

fn find_ascii_case(text: &str, needle: &str) -> Option<usize> {
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

fn target_unit_for(dimension: Dimension, alias: &str) -> Option<&'static UnitDef> {
    let unit = unit_by_alias(alias)?;
    (unit.dimension == dimension).then_some(unit)
}

fn convert_registered_reading(source: &Reading, target_unit: &str) -> Option<Reading> {
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

fn unit_lookup_aliases(unit: &UnitDef) -> impl Iterator<Item = &'static str> + '_ {
    unit.aliases
        .iter()
        .copied()
        .chain(core::iter::once(unit.id))
}

fn custom_unit_lookup_aliases(unit: &CustomUnit) -> impl Iterator<Item = &str> {
    core::iter::once(unit.id.as_str()).chain(unit.aliases.iter().map(String::as_str))
}

fn exact_custom_alias(unit: &CustomUnit, alias: &str) -> bool {
    custom_unit_lookup_aliases(unit).any(|candidate| candidate == alias)
}

fn custom_unit_by_alias<'a>(alias: &str, ctx: &'a ParseCtx) -> Option<&'a CustomUnit> {
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

fn normalize_alias(alias: &str) -> String {
    alias.trim().to_ascii_lowercase()
}

fn parse_metric_length(text: &str) -> Option<Reading> {
    let stripped = text.trim().to_ascii_lowercase();
    if let Some((meters, centimeters)) = stripped.split_once('m')
        && !meters.is_empty()
        && !centimeters.is_empty()
        && !centimeters.contains(char::is_whitespace)
    {
        let meters = parse_number(meters.trim())?;
        let centimeters = parse_number(centimeters.trim())?;
        return Some(Reading::quantity(
            meters + centimeters * CM_M,
            "m",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.97,
        ));
    }

    for (suffix, factor) in [
        ("cm", CM_M),
        ("mm", 0.001),
        ("in", INCH_M),
        ("inch", INCH_M),
        ("inches", INCH_M),
        ("ft", FOOT_M),
        ("feet", FOOT_M),
        ("m", 1.0),
    ] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value * factor,
                "m",
                Dimension::Length,
                Provenance::SiMultiple,
                false,
                0.99,
            ));
        }
    }
    None
}

fn parse_mass(text: &str) -> Option<Reading> {
    let stripped = text.trim().to_ascii_lowercase();
    if let Some((pounds_text, ounces_tail)) = stripped.split_once(" lb ") {
        let ounces_text = ounces_tail
            .strip_suffix(" oz")
            .or_else(|| ounces_tail.strip_suffix(" ounce"))
            .or_else(|| ounces_tail.strip_suffix(" ounces"))?;
        let pounds = parse_number(pounds_text.trim())?;
        let ounces = parse_number(ounces_text.trim())?;
        return Some(Reading::quantity(
            pounds * LB_KG + ounces * OZ_KG,
            "kg",
            Dimension::Mass,
            Provenance::InternationalExact,
            false,
            0.96,
        ));
    }

    for (suffix, factor) in [
        ("kg", 1.0),
        ("kilograms", 1.0),
        ("kilogram", 1.0),
        ("公斤", 1.0),
        ("キログラム", 1.0),
        ("キロ", 1.0),
        ("lbs", LB_KG),
        ("lb", LB_KG),
        ("pounds", LB_KG),
        ("pound", LB_KG),
        ("ounces", OZ_KG),
        ("ounce", OZ_KG),
        ("oz", OZ_KG),
        ("g", 0.001),
    ] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value * factor,
                "kg",
                Dimension::Mass,
                Provenance::SiMultiple,
                false,
                0.98,
            ));
        }
    }
    None
}

fn parse_duration(text: &str) -> Option<Reading> {
    let stripped = text.trim().to_ascii_lowercase();
    if stripped == "an hour and a half" || stripped == "one hour and a half" {
        return Some(Reading::quantity(
            5400.0,
            "s",
            Dimension::Time,
            Provenance::SiMultiple,
            false,
            0.97,
        ));
    }

    if let Some(seconds) = parse_iso_duration(&stripped) {
        return Some(Reading::quantity(
            seconds,
            "s",
            Dimension::Time,
            Provenance::SiMultiple,
            false,
            0.94,
        ));
    }

    if let Some(seconds) = parse_compact_duration(&stripped) {
        return Some(Reading::quantity(
            seconds,
            "s",
            Dimension::Time,
            Provenance::SiMultiple,
            false,
            0.94,
        ));
    }

    for (suffix, factor) in [
        ("minutes", 60.0),
        ("minute", 60.0),
        ("mins", 60.0),
        ("min", 60.0),
        ("hours", 3600.0),
        ("hour", 3600.0),
        ("hrs", 3600.0),
        ("hr", 3600.0),
        ("h", 3600.0),
        ("日", 86_400.0),
        ("days", 86_400.0),
        ("day", 86_400.0),
        ("d", 86_400.0),
    ] {
        if let Some(number_text) = stripped.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                value * factor,
                "s",
                Dimension::Time,
                Provenance::SiMultiple,
                false,
                0.96,
            ));
        }
    }
    None
}

fn parse_iso_duration(text: &str) -> Option<f64> {
    let mut chars = text.trim().chars();
    if !chars.next()?.eq_ignore_ascii_case(&'P') {
        return None;
    }

    let mut seconds = 0.0;
    let mut number = String::new();
    let mut in_time = false;
    let mut saw_component = false;

    for ch in chars {
        if ch == 'T' || ch == 't' {
            if in_time || !number.is_empty() {
                return None;
            }
            in_time = true;
            continue;
        }

        if ch.is_ascii_digit() || ch == '.' {
            number.push(ch);
            continue;
        }

        if number.is_empty() {
            return None;
        }

        let value = number.parse::<f64>().ok()?;
        number.clear();
        match ch.to_ascii_uppercase() {
            'W' if !in_time => seconds += value * 7.0 * 86_400.0,
            'D' if !in_time => seconds += value * 86_400.0,
            'H' if in_time => seconds += value * 3600.0,
            'M' if in_time => seconds += value * 60.0,
            'S' if in_time => seconds += value,
            _ => return None,
        }
        saw_component = true;
    }

    if !number.is_empty() {
        return None;
    }
    saw_component.then_some(seconds)
}

fn parse_compact_duration(text: &str) -> Option<f64> {
    if !text.trim().starts_with(|ch: char| ch.is_ascii_digit()) {
        return None;
    }

    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if compact.is_empty() {
        return None;
    }

    let mut rest = compact.as_str();
    let mut seconds = 0.0;
    let mut saw_component = false;
    let mut last_unit: Option<&str> = None;

    while !rest.is_empty() {
        let number_end = rest
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_digit() || *ch == '.')
            .map(|(idx, ch)| idx + ch.len_utf8())
            .last()
            .unwrap_or(0);
        if number_end == 0 {
            return None;
        }

        let value = rest[..number_end].parse::<f64>().ok()?;
        rest = &rest[number_end..];
        if rest.is_empty() {
            if last_unit == Some("h") {
                seconds += value * 60.0;
                saw_component = true;
                break;
            }
            return None;
        }

        let unit_end = rest
            .char_indices()
            .take_while(|(_, ch)| ch.is_ascii_alphabetic())
            .map(|(idx, ch)| idx + ch.len_utf8())
            .last()
            .unwrap_or(0);
        if unit_end == 0 {
            return None;
        }

        let unit = &rest[..unit_end];
        let factor = match unit {
            "w" => 7.0 * 86_400.0,
            "d" => 86_400.0,
            "h" | "hr" | "hrs" => 3600.0,
            "m" | "min" | "mins" => 60.0,
            "s" | "sec" | "secs" => 1.0,
            _ => return None,
        };
        seconds += value * factor;
        saw_component = true;
        last_unit = Some(if matches!(unit, "hr" | "hrs") {
            "h"
        } else {
            unit
        });
        rest = &rest[unit_end..];
    }

    saw_component.then_some(seconds)
}

fn parse_clock_time(text: &str) -> Option<Reading> {
    let seconds = parse_clock_seconds(text)?;
    Some(Reading::quantity(
        seconds,
        "s",
        Dimension::Time,
        Provenance::TradeCustom,
        false,
        0.92,
    ))
}

fn parse_clock_seconds(text: &str) -> Option<f64> {
    let lowered = text.trim().to_ascii_lowercase();
    let compact: String = lowered.chars().filter(|ch| !ch.is_whitespace()).collect();

    if compact == "noon" {
        return Some(12.0 * 3600.0);
    }
    if compact == "midnight" {
        return Some(0.0);
    }

    let (body, meridiem) = if let Some(body) = compact.strip_suffix("am") {
        (body, Some("am"))
    } else if let Some(body) = compact.strip_suffix("pm") {
        (body, Some("pm"))
    } else {
        (compact.as_str(), None)
    };

    let (hour_text, minute_text) = body.split_once(':').unwrap_or((body, "0"));
    if hour_text.is_empty()
        || minute_text.is_empty()
        || !hour_text.chars().all(|ch| ch.is_ascii_digit())
        || !minute_text.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    let mut hour = hour_text.parse::<u8>().ok()?;
    let minute = minute_text.parse::<u8>().ok()?;
    if minute > 59 {
        return None;
    }

    match meridiem {
        Some("am") => {
            if hour == 0 || hour > 12 {
                return None;
            }
            if hour == 12 {
                hour = 0;
            }
        }
        Some("pm") => {
            if hour == 0 || hour > 12 {
                return None;
            }
            if hour != 12 {
                hour += 12;
            }
        }
        Some(_) => return None,
        None => {
            if !body.contains(':') || hour > 23 {
                return None;
            }
        }
    }

    Some(f64::from(hour) * 3600.0 + f64::from(minute) * 60.0)
}

fn parse_conversion_request(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (source, target_unit) = split_once_ascii_case(text.trim(), " to ")?;
    let source = parse_endpoint(source, ctx)?;
    let target_unit = target_unit.trim();

    if let Some(reading) = convert_registered_reading(&source, target_unit) {
        return Some(reading);
    }

    let value = source.value?;
    let source_unit = source.unit.as_deref()?;

    match (source.dimension?, source_unit, target_unit) {
        (Dimension::Length, "m", "cm") => Some(Reading::quantity(
            value / CM_M,
            "cm",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.95,
        )),
        (Dimension::Length, "m", "m") => Some(Reading::quantity(
            value,
            "m",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.95,
        )),
        (Dimension::Length, "m", "in") => Some(Reading::quantity(
            value / INCH_M,
            "in",
            Dimension::Length,
            Provenance::InternationalExact,
            false,
            0.95,
        )),
        (Dimension::Length, "m", "ft") => Some(Reading::quantity(
            value / FOOT_M,
            "ft",
            Dimension::Length,
            Provenance::InternationalExact,
            false,
            0.95,
        )),
        (Dimension::Mass, "kg", "lb" | "lbs") => Some(Reading::quantity(
            value / LB_KG,
            "lb",
            Dimension::Mass,
            Provenance::InternationalExact,
            false,
            0.95,
        )),
        (Dimension::Mass, "kg", "kg") => Some(Reading::quantity(
            value,
            "kg",
            Dimension::Mass,
            Provenance::SiMultiple,
            false,
            0.95,
        )),
        (Dimension::Currency, unit, target) => {
            let target = normalize_currency_code(target);
            if unit == target {
                return Some(currency_reading(value, unit, 0.96));
            }
            let rate = ctx
                .currency_rates
                .iter()
                .find(|rate| rate.from == unit && rate.to == target)?;
            Some(currency_reading(value * rate.factor, &target, 0.91))
        }
        _ => None,
    }
}

fn parse_feet_inches(text: &str) -> Option<Reading> {
    let lowered = text.trim().to_ascii_lowercase();
    let ft_pos = lowered
        .find("ft")
        .or_else(|| lowered.find("feet"))
        .or_else(|| lowered.find('\''))?;
    let feet = parse_number(lowered[..ft_pos].trim())?;
    let rest = lowered[ft_pos..]
        .trim_start_matches("feet")
        .trim_start_matches("ft")
        .trim_start_matches('\'')
        .trim();
    let inches = if rest.is_empty() {
        0.0
    } else {
        let cleaned = rest
            .trim_end_matches("inches")
            .trim_end_matches("inch")
            .trim_end_matches("in")
            .trim_end_matches('"')
            .trim();
        parse_number(cleaned)?
    };

    Some(Reading::quantity(
        feet * FOOT_M + inches * INCH_M,
        "m",
        Dimension::Length,
        Provenance::InternationalExact,
        false,
        0.97,
    ))
}

fn parse_cups(text: &str, ctx: &ParseCtx) -> Option<(Reading, Vec<Reading>, Ambiguity)> {
    let lowered = text.trim().to_ascii_lowercase();
    let unit_text = if lowered.ends_with("cups") {
        "cups"
    } else if lowered.ends_with("cup") {
        "cup"
    } else {
        return None;
    };
    let number_text = lowered.strip_suffix(unit_text)?.trim();
    let value = parse_number(number_text)?;

    let us = Reading::quantity(
        value * US_CUP_L,
        "L",
        Dimension::Volume,
        Provenance::TradeCustom,
        true,
        0.72,
    );
    let uk = Reading::quantity(
        value * UK_CUP_L,
        "L",
        Dimension::Volume,
        Provenance::TradeCustom,
        true,
        0.68,
    );
    let metric = Reading::quantity(
        value * METRIC_CUP_L,
        "L",
        Dimension::Volume,
        Provenance::TradeCustom,
        true,
        0.66,
    );

    let (best, alternatives) = match ctx.locale.as_ref() {
        Some(Locale::EnGb) => (uk, vec![us, metric]),
        Some(Locale::Ja) | Some(Locale::EnUs) => (us, vec![metric, uk]),
        _ => (us, vec![metric, uk]),
    };

    Some((
        best,
        alternatives,
        ambiguity_with_span(
            unit_text,
            "Cup volume depends on locale (US, imperial, or metric cup).",
            Some(3),
            IssueCode::AmbiguousUnit,
            span_token_in(text, unit_text),
        ),
    ))
}

fn parse_currency(text: &str) -> Option<(Reading, Vec<Reading>, Option<Ambiguity>)> {
    let trimmed = text.trim();

    for (prefix, code) in [
        ("US$", "USD"),
        ("USD", "USD"),
        ("usd", "USD"),
        ("EUR", "EUR"),
        ("eur", "EUR"),
        ("GBP", "GBP"),
        ("gbp", "GBP"),
        ("JPY", "JPY"),
        ("jpy", "JPY"),
        ("€", "EUR"),
        ("£", "GBP"),
        ("¥", "JPY"),
        ("￥", "JPY"),
    ] {
        if let Some(number_text) = strip_prefix_currency(trimmed, prefix) {
            let value = parse_number(number_text.trim())?;
            return Some((currency_reading(value, code, 0.95), Vec::new(), None));
        }
    }

    for (suffix, code) in [
        ("USD", "USD"),
        ("usd", "USD"),
        ("EUR", "EUR"),
        ("eur", "EUR"),
        ("GBP", "GBP"),
        ("gbp", "GBP"),
        ("JPY", "JPY"),
        ("jpy", "JPY"),
        ("dollars", "USD"),
        ("dollar", "USD"),
        ("bucks", "USD"),
        ("buck", "USD"),
        ("euros", "EUR"),
        ("euro", "EUR"),
        ("pounds", "GBP"),
        ("pound", "GBP"),
        ("quid", "GBP"),
        ("yen", "JPY"),
        ("円", "JPY"),
    ] {
        if let Some(number_text) = strip_suffix_currency(trimmed, suffix) {
            let value = parse_number(number_text.trim())?;
            return Some((currency_reading(value, code, 0.95), Vec::new(), None));
        }
    }

    for (suffix, code) in [
        ("pence", "GBP"),
        ("penny", "GBP"),
        ("cents usd", "USD"),
        ("cent usd", "USD"),
        ("euro cents", "EUR"),
        ("euro cent", "EUR"),
    ] {
        if let Some(number_text) = strip_suffix_currency(trimmed, suffix) {
            let value = parse_number(number_text.trim())?;
            return Some((
                currency_reading(value / 100.0, code, 0.93),
                Vec::new(),
                None,
            ));
        }
    }

    if let Some(number_text) =
        strip_suffix_currency(trimmed, "cents").or_else(|| strip_suffix_currency(trimmed, "cent"))
    {
        let value = parse_number(number_text.trim())?;
        let best = currency_reading(value / 100.0, "USD", 0.67);
        let alternatives = vec![currency_reading(value / 100.0, "EUR", 0.58)];
        let fragment = if trimmed.ends_with('s') {
            "cents"
        } else {
            "cent"
        };
        let ambiguity = ambiguity_with_span(
            fragment,
            "Cent minor unit needs currency context.",
            Some(2),
            IssueCode::AmbiguousCurrency,
            span_token_in(trimmed, fragment),
        );
        return Some((best, alternatives, Some(ambiguity)));
    }

    if let Some(number_text) = trimmed.strip_prefix('$') {
        let value = parse_number(number_text.trim())?;
        let best = currency_reading(value, "USD", 0.74);
        let alternatives = vec![
            currency_reading(value, "CAD", 0.61),
            currency_reading(value, "AUD", 0.59),
        ];
        let ambiguity = ambiguity_with_span(
            "$",
            "Dollar symbol can refer to multiple currencies without locale or market context.",
            Some(3),
            IssueCode::AmbiguousCurrency,
            span_token_in(trimmed, "$"),
        );
        return Some((best, alternatives, Some(ambiguity)));
    }

    None
}

fn currency_reading(value: f64, unit: &str, confidence: f64) -> Reading {
    Reading::quantity(
        value,
        unit,
        Dimension::Currency,
        Provenance::TradeCustom,
        false,
        confidence,
    )
}

fn strip_prefix_currency<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if prefix.is_ascii() {
        let candidate = text.get(..prefix.len())?;
        candidate
            .eq_ignore_ascii_case(prefix)
            .then(|| &text[prefix.len()..])
    } else {
        text.strip_prefix(prefix)
    }
}

fn strip_suffix_currency<'a>(text: &'a str, suffix: &str) -> Option<&'a str> {
    if suffix.is_ascii() {
        if text.len() < suffix.len() {
            return None;
        }
        let start = text.len() - suffix.len();
        let candidate = text.get(start..)?;
        candidate
            .eq_ignore_ascii_case(suffix)
            .then(|| &text[..start])
    } else {
        text.strip_suffix(suffix)
    }
}

fn normalize_currency_code(code: &str) -> String {
    code.trim().to_ascii_uppercase()
}

fn parse_plain_number(text: &str) -> Option<Reading> {
    parse_number(text).map(|value| Reading::number(value, 0.99))
}

fn parse_ambiguous_number(text: &str) -> Option<AmbiguousParse> {
    if text.matches(',').count() != 1 || text.contains('.') || !valid_grouped_number(text) {
        return None;
    }
    let best = parse_number(text)?;
    let decimal = text.replace(',', ".").parse::<f64>().ok()?;
    Some(AmbiguousParse {
        best: Some(Reading::number(best, 0.64)),
        alternatives: vec![Reading::number(decimal, 0.56)],
        ambiguity: ambiguity(
            text,
            "Comma can be read as a thousands separator or a decimal separator.",
            Some(2),
            IssueCode::AmbiguousNumber,
        ),
    })
}

fn parse_range(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (left, right) = split_range_text(text)?;
    let right_suffix = unit_suffix(right, ctx);
    let left_with_unit;
    let left_text = if right_suffix.is_some() && unit_suffix(left, ctx).is_none() {
        left_with_unit = format!("{}{}", left.trim(), right_suffix?);
        left_with_unit.as_str()
    } else {
        left.trim()
    };

    let from = parse_endpoint(left_text, ctx)?;
    let to = parse_endpoint(right.trim(), ctx)?;
    if from.kind != to.kind {
        return None;
    }
    Some(Reading::range(from, to, 0.94))
}

fn split_range_text(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    if let Some(inner) = trimmed.strip_prefix("between ") {
        return inner.split_once(" and ");
    }
    if let Some(inner) = trimmed.strip_prefix("from ") {
        return inner.split_once(" to ");
    }
    for separator in ["〜", "～", " to ", ".."] {
        if let Some((left, right)) = trimmed.split_once(separator) {
            return non_empty_pair(left, right);
        }
    }
    if let Some((left, right)) = split_clock_hyphen_range(trimmed) {
        return Some((left, right));
    }
    split_ascii_hyphen_range(trimmed)
}

fn split_clock_hyphen_range(text: &str) -> Option<(&str, &str)> {
    let (left, right) = text.split_once('-')?;
    if parse_clock_seconds(left).is_some() && parse_clock_seconds(right).is_some() {
        non_empty_pair(left, right)
    } else {
        None
    }
}

fn split_ascii_hyphen_range(text: &str) -> Option<(&str, &str)> {
    let mut previous = None;
    for (idx, ch) in text.char_indices() {
        if ch != '-' {
            previous = Some(ch);
            continue;
        }
        let next = text[idx + 1..].chars().next();
        if previous?.is_ascii_digit() && next?.is_ascii_digit() {
            return non_empty_pair(&text[..idx], &text[idx + 1..]);
        }
    }
    None
}

fn non_empty_pair<'a>(left: &'a str, right: &'a str) -> Option<(&'a str, &'a str)> {
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        None
    } else {
        Some((left, right))
    }
}

fn unit_suffix<'a>(text: &str, ctx: &'a ParseCtx) -> Option<&'a str> {
    let trimmed = text.trim();
    let mut best = [
        "㎡", "m^2", "m2", "平米", "帖", "畳", "坪", "cm", "mm", "m", "kg", "g", "minutes",
        "minute", "mins", "min", "hours", "hour", "hrs", "hr", "days", "day", "日",
    ]
    .into_iter()
    .chain(
        UNIT_DEFS
            .iter()
            .flat_map(|unit| unit.aliases.iter().copied()),
    )
    .filter(|suffix| ends_with_ascii_case(trimmed, suffix))
    .max_by_key(|suffix| suffix.len());

    for unit in &ctx.custom_units {
        for suffix in
            core::iter::once(unit.id.as_str()).chain(unit.aliases.iter().map(String::as_str))
        {
            if ends_with_ascii_case(trimmed, suffix)
                && best.is_none_or(|current| suffix.len() > current.len())
            {
                best = Some(suffix);
            }
        }
    }

    best
}

fn ends_with_ascii_case(text: &str, suffix: &str) -> bool {
    if text.ends_with(suffix) {
        return true;
    }
    if text.len() < suffix.len() || !suffix.is_ascii() {
        return false;
    }
    text.get(text.len() - suffix.len()..)
        .is_some_and(|tail| tail.eq_ignore_ascii_case(suffix))
}

fn parse_endpoint(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    parse_relative_date(text, ctx)
        .or_else(|| parse_japanese_length(text))
        .or_else(|| parse_tatami_area(text))
        .or_else(|| parse_tsubo_area(text))
        .or_else(|| parse_square_meter(text))
        .or_else(|| parse_temperature(text))
        .or_else(|| parse_registered_quantity(text, ctx))
        .or_else(|| parse_metric_length(text))
        .or_else(|| parse_mass(text))
        .or_else(|| parse_clock_time(text))
        .or_else(|| parse_duration(text))
        .or_else(|| parse_feet_inches(text))
        .or_else(|| parse_currency(text).map(|(best, _, _)| best))
        .or_else(|| parse_plain_number(text))
}

fn parse_ambiguous_slash_date_or_fraction(text: &str, ctx: &ParseCtx) -> Option<AmbiguousParse> {
    let (left, right) = text.split_once('/')?;
    if left.is_empty() || right.is_empty() {
        return None;
    }
    let numerator = parse_number(left.trim())?;
    let denominator = parse_number(right.trim())?;
    if denominator == 0.0 {
        return None;
    }

    let fraction = Reading::number(numerator / denominator, 0.55);
    let mut alternatives = Vec::new();
    if let Some(reference_date) = ctx.reference_date {
        let month = numerator as u8;
        let day = denominator as u8;
        if let Some(date) = Date::new(reference_date.year, month, day) {
            alternatives.push(Reading::date(date, 0.51));
        }
    }

    if alternatives.is_empty() {
        return None;
    }

    Some(AmbiguousParse {
        best: Some(fraction),
        alternatives,
        ambiguity: ambiguity(
            text,
            "Slash expression can be read as a fraction or a month/day date.",
            Some(2),
            IssueCode::AmbiguousDate,
        ),
    })
}

#[cfg(feature = "dates-jiff")]
fn parse_relative_date(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    use jiff::{ToSpan, civil::Date as JiffDate};

    let reference = ctx.reference_date?;
    let base = to_jiff_date(reference)?;
    let lowered = text.trim().to_ascii_lowercase();

    if lowered == "today" {
        return Some(Reading::date(reference, 0.99));
    }

    if text == "今日" {
        return Some(Reading::date(reference, 0.99));
    }

    if lowered == "tomorrow" {
        return from_jiff_date(base.tomorrow().ok()?).map(|date| Reading::date(date, 0.98));
    }

    if text == "明日" {
        return from_jiff_date(base.tomorrow().ok()?).map(|date| Reading::date(date, 0.98));
    }

    if text == "明後日" {
        return from_jiff_date(base.checked_add(2.days()).ok()?)
            .map(|date| Reading::date(date, 0.97));
    }

    if let Some(days_text) = lowered.strip_prefix("in ").and_then(|tail| {
        tail.strip_suffix(" days")
            .or_else(|| tail.strip_suffix(" day"))
    }) {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_add(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(days_text) = text.strip_suffix("日後") {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_add(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_prefix("next ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_prefix("来週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Ok(date) = lowered.parse::<JiffDate>() {
        return from_jiff_date(date).map(|date| Reading::date(date, 0.99));
    }

    None
}

#[cfg(not(feature = "dates-jiff"))]
fn parse_relative_date(_text: &str, _ctx: &ParseCtx) -> Option<Reading> {
    None
}

#[cfg(feature = "dates-jiff")]
fn to_jiff_date(date: Date) -> Option<jiff::civil::Date> {
    jiff::civil::Date::new(
        i16::try_from(date.year).ok()?,
        i8::try_from(date.month).ok()?,
        i8::try_from(date.day).ok()?,
    )
    .ok()
}

#[cfg(feature = "dates-jiff")]
fn from_jiff_date(date: jiff::civil::Date) -> Option<Date> {
    Date::new(
        date.year().into(),
        date.month().try_into().ok()?,
        date.day().try_into().ok()?,
    )
}

#[cfg(feature = "dates-jiff")]
fn parse_weekday(text: &str) -> Option<jiff::civil::Weekday> {
    match text {
        "monday" | "mon" => Some(jiff::civil::Weekday::Monday),
        "tuesday" | "tue" | "tues" => Some(jiff::civil::Weekday::Tuesday),
        "wednesday" | "wed" => Some(jiff::civil::Weekday::Wednesday),
        "thursday" | "thu" | "thur" | "thurs" => Some(jiff::civil::Weekday::Thursday),
        "friday" | "fri" => Some(jiff::civil::Weekday::Friday),
        "saturday" | "sat" => Some(jiff::civil::Weekday::Saturday),
        "sunday" | "sun" => Some(jiff::civil::Weekday::Sunday),
        "月曜日" | "月曜" | "月" => Some(jiff::civil::Weekday::Monday),
        "火曜日" | "火曜" | "火" => Some(jiff::civil::Weekday::Tuesday),
        "水曜日" | "水曜" | "水" => Some(jiff::civil::Weekday::Wednesday),
        "木曜日" | "木曜" | "木" => Some(jiff::civil::Weekday::Thursday),
        "金曜日" | "金曜" | "金" => Some(jiff::civil::Weekday::Friday),
        "土曜日" | "土曜" | "土" => Some(jiff::civil::Weekday::Saturday),
        "日曜日" | "日曜" | "日" => Some(jiff::civil::Weekday::Sunday),
        _ => None,
    }
}

fn parse_number(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(value) = parse_unicode_fraction_number(trimmed) {
        return Some(value);
    }

    if let Some(value) = parse_english_number_words(trimmed) {
        return Some(value as f64);
    }

    if let Some(value) = parse_cjk_number(trimmed) {
        return Some(value as f64);
    }

    let normalized = if trimmed.contains(',') && valid_grouped_number(trimmed) {
        trimmed.replace(',', "")
    } else if trimmed.matches(',').count() == 1 && !trimmed.contains('.') {
        trimmed.replace(',', ".")
    } else if trimmed.contains(',') {
        return None;
    } else {
        trimmed.to_owned()
    };

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+')
    {
        normalized.parse::<f64>().ok()
    } else {
        None
    }
}

fn parse_unicode_fraction_number(text: &str) -> Option<f64> {
    let mut chars = text.chars();
    let fraction = chars.next_back().and_then(fraction_char_value)?;
    let whole_text = chars.as_str().trim();
    if whole_text.is_empty() {
        return Some(fraction);
    }
    if whole_text.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(whole_text.parse::<f64>().ok()? + fraction);
    }
    None
}

fn fraction_char_value(ch: char) -> Option<f64> {
    match ch {
        '¼' => Some(0.25),
        '½' => Some(0.5),
        '¾' => Some(0.75),
        '⅓' => Some(1.0 / 3.0),
        '⅔' => Some(2.0 / 3.0),
        '⅛' => Some(0.125),
        '⅜' => Some(0.375),
        '⅝' => Some(0.625),
        '⅞' => Some(0.875),
        _ => None,
    }
}

fn parse_english_number_words(text: &str) -> Option<i64> {
    let normalized = text
        .to_ascii_lowercase()
        .replace('-', " ")
        .replace(" and ", " ");
    let mut total = 0_i64;
    let mut current = 0_i64;
    let mut saw_word = false;

    for word in normalized.split_whitespace() {
        if word == "a" || word == "an" {
            current += 1;
            saw_word = true;
            continue;
        }
        if let Some(value) = small_number_word(word) {
            current += value;
            saw_word = true;
            continue;
        }
        if word == "hundred" {
            current *= 100;
            saw_word = true;
            continue;
        }
        if word == "thousand" {
            total += current * 1000;
            current = 0;
            saw_word = true;
            continue;
        }
        return None;
    }

    saw_word.then_some(total + current)
}

fn small_number_word(word: &str) -> Option<i64> {
    match word {
        "zero" => Some(0),
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        "four" => Some(4),
        "five" => Some(5),
        "six" => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine" => Some(9),
        "ten" => Some(10),
        "eleven" => Some(11),
        "twelve" => Some(12),
        "thirteen" => Some(13),
        "fourteen" => Some(14),
        "fifteen" => Some(15),
        "sixteen" => Some(16),
        "seventeen" => Some(17),
        "eighteen" => Some(18),
        "nineteen" => Some(19),
        "twenty" => Some(20),
        "thirty" => Some(30),
        "forty" => Some(40),
        "fifty" => Some(50),
        "sixty" => Some(60),
        "seventy" => Some(70),
        "eighty" => Some(80),
        "ninety" => Some(90),
        _ => None,
    }
}

fn parse_cjk_number(text: &str) -> Option<i64> {
    let mut total = 0_i64;
    let mut section = 0_i64;
    let mut number = 0_i64;
    let mut saw = false;

    for ch in text.chars() {
        if let Some(value) = cjk_digit(ch) {
            number = value;
            saw = true;
            continue;
        }
        let unit = match ch {
            '十' => 10,
            '百' => 100,
            '千' => 1000,
            '万' => {
                section += number;
                total += section * 10_000;
                section = 0;
                number = 0;
                saw = true;
                continue;
            }
            _ => return None,
        };
        section += if number == 0 { unit } else { number * unit };
        number = 0;
        saw = true;
    }

    saw.then_some(total + section + number)
}

fn cjk_digit(ch: char) -> Option<i64> {
    match ch {
        '零' | '〇' => Some(0),
        '一' | '壱' => Some(1),
        '二' | '弐' => Some(2),
        '三' | '参' => Some(3),
        '四' => Some(4),
        '五' => Some(5),
        '六' => Some(6),
        '七' => Some(7),
        '八' => Some(8),
        '九' => Some(9),
        _ => None,
    }
}

#[cfg(feature = "dates-jiff")]
fn parse_whole_i64(text: &str) -> Option<i64> {
    if text.is_empty() || !text.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn valid_grouped_number(text: &str) -> bool {
    let (whole, decimal) = text.split_once('.').unwrap_or((text, ""));
    if !decimal.is_empty() && !decimal.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    let signless = whole.trim_start_matches(['-', '+']);
    let groups: Vec<&str> = signless.split(',').collect();
    if groups.len() <= 1 || groups[0].is_empty() || groups[0].len() > 3 {
        return false;
    }
    groups.iter().enumerate().all(|(idx, group)| {
        group.chars().all(|ch| ch.is_ascii_digit()) && (idx == 0 || group.len() == 3)
    })
}

const LEGACY_UNIT_COMPLETIONS: &[(&str, &str, Dimension)] = &[
    ("shaku", "shaku", Dimension::Length),
    ("尺", "shaku", Dimension::Length),
    ("sun", "sun", Dimension::Length),
    ("寸", "sun", Dimension::Length),
    ("ken", "ken", Dimension::Length),
    ("間", "ken", Dimension::Length),
    ("tsubo", "tsubo", Dimension::Area),
    ("坪", "tsubo", Dimension::Area),
    ("tatami", "tatami", Dimension::Area),
    ("帖", "tatami", Dimension::Area),
    ("畳", "tatami", Dimension::Area),
    ("celsius", "C", Dimension::Temperature),
    ("°C", "C", Dimension::Temperature),
    ("℃", "C", Dimension::Temperature),
    ("fahrenheit", "F", Dimension::Temperature),
    ("°F", "F", Dimension::Temperature),
    ("℉", "F", Dimension::Temperature),
    ("kelvin", "K", Dimension::Temperature),
    ("摂氏", "C", Dimension::Temperature),
    ("華氏", "F", Dimension::Temperature),
];

const DATE_COMPLETIONS: &[&str] = &[
    "today",
    "tomorrow",
    "next monday",
    "next tuesday",
    "next wednesday",
    "next thursday",
    "next friday",
    "next saturday",
    "next sunday",
    "今日",
    "明日",
    "来週月曜日",
    "来週火曜日",
    "来週水曜日",
    "来週木曜日",
    "来週金曜日",
    "来週土曜日",
    "来週日曜日",
];

const TIME_COMPLETIONS: &[&str] = &["noon", "midnight"];

const CURRENCY_COMPLETIONS: &[(&str, &str)] = &[
    ("USD", "USD"),
    ("EUR", "EUR"),
    ("GBP", "GBP"),
    ("JPY", "JPY"),
    ("dollar", "USD"),
    ("dollars", "USD"),
    ("bucks", "USD"),
    ("euro", "EUR"),
    ("euros", "EUR"),
    ("pound", "GBP"),
    ("pounds", "GBP"),
    ("quid", "GBP"),
    ("yen", "JPY"),
    ("円", "JPY"),
    ("pence", "GBP"),
    ("cents", "USD"),
];

struct CompletionCandidate<'a> {
    value: &'a str,
    canonical: Option<&'a str>,
    kind: CompletionKind,
    dimension: Option<Dimension>,
}

fn completion_prefix(input: &str) -> Option<&str> {
    input.split_whitespace().last()
}

fn push_completion(
    completions: &mut Vec<Completion>,
    candidate: CompletionCandidate<'_>,
    normalized_prefix: &str,
    ctx: &ParseCtx,
) {
    push_owned_completion(
        completions,
        candidate.value,
        candidate.canonical,
        candidate.kind,
        candidate.dimension,
        normalized_prefix,
        ctx,
    );
}

fn push_owned_completion(
    completions: &mut Vec<Completion>,
    value: &str,
    canonical: Option<&str>,
    kind: CompletionKind,
    dimension: Option<Dimension>,
    normalized_prefix: &str,
    ctx: &ParseCtx,
) {
    if !completion_allowed(kind, dimension, ctx) {
        return;
    }
    let normalized_value = normalize_alias(value);
    if !normalized_value.starts_with(normalized_prefix) {
        return;
    }
    let score = completion_score(normalized_prefix, &normalized_value);
    if completions.iter().any(|existing| {
        existing.kind == kind
            && existing.value == value
            && existing.canonical.as_deref() == canonical
    }) {
        return;
    }
    completions.push(Completion {
        value: value.to_owned(),
        canonical: canonical.map(str::to_owned),
        kind,
        dimension,
        score,
    });
}

fn completion_allowed(kind: CompletionKind, dimension: Option<Dimension>, ctx: &ParseCtx) -> bool {
    if let Some(expected_dimension) = ctx.expected_dimension
        && dimension != Some(expected_dimension)
    {
        return false;
    }

    match ctx.expect {
        Some(Kind::Date) => kind == CompletionKind::Date,
        Some(Kind::Number) => false,
        Some(Kind::Quantity) => matches!(
            kind,
            CompletionKind::Unit | CompletionKind::Time | CompletionKind::Currency
        ),
        Some(Kind::Range) | None => true,
    }
}

fn completion_score(prefix: &str, value: &str) -> f64 {
    if prefix == value {
        1.0
    } else {
        0.6 + 0.4 * prefix.len() as f64 / value.len().max(prefix.len()) as f64
    }
}

fn suggestions_for(text: &str) -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    for token in ascii_tokens(text) {
        if unit_by_alias(&token).is_some() {
            continue;
        }
        if let Some(suggestion) = suggest_unit(&token).or_else(|| suggest_legacy_word(&token)) {
            suggestions.push(suggestion);
        }
    }
    suggestions
}

fn suggest_unit(token: &str) -> Option<Suggestion> {
    let normalized = normalize_alias(token);
    if normalized.len() > 32 {
        return None;
    }
    let mut best: Option<(&'static str, usize)> = None;
    for unit in UNIT_DEFS {
        for alias in unit_lookup_aliases(unit) {
            let alias = alias.trim();
            if alias.is_empty() || !alias.is_ascii() || alias.contains(char::is_whitespace) {
                continue;
            }
            let limit = if normalized.len() <= 5 { 1 } else { 2 };
            if normalized.len().abs_diff(alias.len()) > limit {
                continue;
            }
            let distance = levenshtein_ascii_case_insensitive(&normalized, alias);
            if distance > 0 && distance <= limit && best.is_none_or(|(_, best)| distance < best) {
                best = Some((unit.id, distance));
            }
        }
    }
    best.map(|(to, distance)| {
        let max_len = normalized.len().max(to.len()) as f64;
        Suggestion {
            from: token.to_owned(),
            to: to.to_owned(),
            score: Some(1.0 - distance as f64 / max_len),
        }
    })
}

fn levenshtein_ascii_case_insensitive(left: &str, right: &str) -> usize {
    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0; right.len() + 1];

    for (i, left_byte) in left.bytes().enumerate() {
        curr[0] = i + 1;
        for (j, right_byte) in right.bytes().enumerate() {
            let substitution = prev[j] + usize::from(!left_byte.eq_ignore_ascii_case(&right_byte));
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}

fn suggest_legacy_word(token: &str) -> Option<Suggestion> {
    if token.len() > 32 {
        return None;
    }
    let dictionary = ["tsubo", "shaku", "sun", "tatami"];
    for candidate in dictionary {
        let distance = levenshtein(token, candidate);
        let limit = if token.len() <= 5 { 1 } else { 2 };
        if distance > 0 && distance <= limit {
            let max_len = token.len().max(candidate.len()) as f64;
            return Some(Suggestion {
                from: token.to_owned(),
                to: candidate.to_owned(),
                score: Some(1.0 - distance as f64 / max_len),
            });
        }
    }
    None
}

fn ascii_tokens(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphabetic() {
            current.push(ch.to_ascii_lowercase());
        } else if !current.is_empty() {
            tokens.push(core::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn levenshtein(left: &str, right: &str) -> usize {
    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0; right.len() + 1];

    for (i, left_byte) in left.bytes().enumerate() {
        curr[0] = i + 1;
        for (j, right_byte) in right.bytes().enumerate() {
            let substitution = prev[j] + usize::from(left_byte != right_byte);
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}

fn humanize_japanese_length(meters: f64) -> String {
    let shaku_total = meters / SHAKU_M;
    let shaku = shaku_total.floor();
    let sun = ((shaku_total - shaku) * 10.0).round();
    if shaku > 0.0 && (shaku_total - (shaku + sun / 10.0)).abs() < 0.02 {
        if sun == 0.0 {
            format!("{}尺 (approx.)", format_number(shaku))
        } else {
            format!(
                "{}尺{}寸 (approx.)",
                format_number(shaku),
                format_number(sun)
            )
        }
    } else {
        format!("{} m", format_number(meters))
    }
}

fn humanize_japanese_area(area: f64) -> String {
    let tatami = area / TATAMI_M2;
    if (tatami - tatami.round()).abs() < 0.02 {
        return format!("{}帖 (approx.)", format_number(tatami.round()));
    }
    let tsubo = area / TSUBO_M2;
    if (tsubo - tsubo.round()).abs() < 0.02 {
        return format!("{}坪 (approx.)", format_number(tsubo.round()));
    }
    format!("{} m2", format_number(area))
}

fn format_number(value: f64) -> String {
    let rounded = (value * 1_000_000.0).round() / 1_000_000.0;
    if (rounded - rounded.trunc()).abs() < f64::EPSILON {
        format!("{}", rounded as i64)
    } else {
        let mut text = format!("{rounded:.6}");
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
        text
    }
}

fn skipped(ref_text: &str, reason: &str) -> Skipped {
    let code = if ref_text.is_empty() {
        IssueCode::Empty
    } else {
        IssueCode::NoValue
    };
    skipped_with_code(ref_text, reason, code)
}

fn skipped_with_code(ref_text: &str, reason: &str, code: IssueCode) -> Skipped {
    skipped_with_span(ref_text, reason, code, span(ref_text))
}

fn skipped_with_span(ref_text: &str, reason: &str, code: IssueCode, span: Span) -> Skipped {
    Skipped {
        code,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        span,
    }
}

fn ambiguity(
    ref_text: &str,
    reason: &str,
    candidate_count: Option<usize>,
    code: IssueCode,
) -> Ambiguity {
    ambiguity_with_span(ref_text, reason, candidate_count, code, span(ref_text))
}

fn ambiguity_with_span(
    ref_text: &str,
    reason: &str,
    candidate_count: Option<usize>,
    code: IssueCode,
    span: Span,
) -> Ambiguity {
    Ambiguity {
        code,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        candidate_count,
        span,
    }
}

fn approximation(ref_text: &str, reason: &str) -> Approximation {
    Approximation {
        code: IssueCode::Approximation,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        relative_error: None,
        span: span(ref_text),
    }
}

fn span(text: &str) -> Span {
    Span {
        start: 0,
        end: text.len(),
        text: text.to_owned(),
    }
}

fn span_in(source: &str, fragment: &str) -> Span {
    if let Some(start) = source.find(fragment) {
        Span {
            start,
            end: start + fragment.len(),
            text: fragment.to_owned(),
        }
    } else {
        span(fragment)
    }
}

fn span_token_in(source: &str, fragment: &str) -> Span {
    token_spans(source)
        .into_iter()
        .find(|token| token.text.eq_ignore_ascii_case(fragment))
        .unwrap_or_else(|| span_in(source, fragment))
}

fn token_spans(source: &str) -> Vec<Span> {
    let mut tokens = Vec::new();
    let mut current: Option<(usize, TokenKind)> = None;

    for (idx, ch) in source.char_indices() {
        let Some(kind) = TokenKind::of(ch) else {
            if let Some((start, _)) = current.take() {
                tokens.push(span_slice(source, start, idx));
            }
            continue;
        };

        match current {
            Some((_, current_kind)) if current_kind == kind && kind != TokenKind::Symbol => {}
            Some((start, _)) => {
                tokens.push(span_slice(source, start, idx));
                current = Some((idx, kind));
            }
            None => current = Some((idx, kind)),
        }
    }

    if let Some((start, _)) = current {
        tokens.push(span_slice(source, start, source.len()));
    }

    tokens
}

fn span_slice(source: &str, start: usize, end: usize) -> Span {
    Span {
        start,
        end,
        text: source[start..end].to_owned(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenKind {
    Number,
    Word,
    Symbol,
}

impl TokenKind {
    fn of(ch: char) -> Option<Self> {
        if ch.is_whitespace() {
            None
        } else if ch.is_ascii_digit() || matches!(ch, '.' | ',' | '+' | '-') {
            Some(Self::Number)
        } else if ch.is_alphabetic() || ch == '_' {
            Some(Self::Word)
        } else {
            Some(Self::Symbol)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64) {
        assert!(
            (actual - expected).abs() < 0.000_001,
            "actual={actual}, expected={expected}"
        );
    }

    #[test]
    fn exposes_schema_and_mcp_helpers() {
        assert_eq!(contract_version(), "unravel-nl.parse.v1");
        assert!(parse_input_schema_json().contains("\"text\""));
        assert!(parse_input_schema_json().contains("\"strictness\""));
        assert!(parsed_output_schema_json().contains("\"findings\""));
        assert!(parsed_output_schema_json().contains("\"currency\""));
        assert!(parsed_output_schema_json().contains("\"temperature\""));
        assert!(mcp_tool_schema_json().contains("unravel_nl_parse"));
        assert!(mcp_tool_schema_json().contains("inputSchema"));
        assert!(mcp_tool_schema_json().contains("outputSchema"));
    }

    #[test]
    fn completes_units_dates_and_custom_units() {
        let metric = complete("10 met", None);
        assert_eq!(metric[0].value, "meter");
        assert_eq!(metric[0].canonical.as_deref(), Some("m"));
        assert_eq!(metric[0].kind, CompletionKind::Unit);
        assert_eq!(metric[0].dimension, Some(Dimension::Length));

        let date = complete(
            "tom",
            Some(ParseCtx {
                expect: Some(Kind::Date),
                ..ParseCtx::default()
            }),
        );
        assert!(date.iter().any(|item| item.value == "tomorrow"));
        assert!(date.iter().all(|item| item.kind == CompletionKind::Date));

        let area = complete(
            "坪",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                expected_dimension: Some(Dimension::Area),
                ..ParseCtx::default()
            }),
        );
        assert!(
            area.iter()
                .any(|item| item.value == "坪" && item.canonical.as_deref() == Some("tsubo"))
        );

        let custom = complete(
            "smo",
            Some(ParseCtx {
                custom_units: vec![CustomUnit::new(
                    "smoot",
                    "m",
                    &["smoot", "smoots"],
                    Dimension::Length,
                    1.7018,
                )],
                ..ParseCtx::default()
            }),
        );
        assert!(
            custom
                .iter()
                .any(|item| item.value == "smoot" && item.canonical.as_deref() == Some("smoot"))
        );

        let temperature = complete(
            "cel",
            Some(ParseCtx {
                expected_dimension: Some(Dimension::Temperature),
                ..ParseCtx::default()
            }),
        );
        assert!(
            temperature
                .iter()
                .any(|item| item.value == "celsius" && item.canonical.as_deref() == Some("C"))
        );
    }

    #[test]
    fn rejects_hostile_no_match_corpus() {
        for input in [
            "meters meters meters",
            "1,,,,,,,,kg",
            "nextnextnextnextnext",
            "(((((((((((((((((((((((((((((((((",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "尺尺尺尺尺",
        ] {
            let parsed = parse(input, None);
            assert!(parsed.best.is_none(), "{input}");
            assert_eq!(parsed.findings.skipped.len(), 1, "{input}");
            assert_eq!(parsed.findings.skipped[0].code, IssueCode::NoValue);
            if input.starts_with('a') {
                assert!(parsed.suggestions.is_empty(), "{input}");
            }
        }
    }

    #[test]
    fn parses_temperature_forms() {
        let celsius = parse("20°C", None).best.expect("celsius");
        assert_eq!(celsius.unit.as_deref(), Some("C"));
        assert_eq!(celsius.dimension, Some(Dimension::Temperature));
        assert_close(celsius.value.unwrap(), 20.0);

        let fahrenheit = parse("68 F", None).best.expect("fahrenheit");
        assert_eq!(fahrenheit.unit.as_deref(), Some("C"));
        assert_close(fahrenheit.value.unwrap(), 20.0);

        let kelvin = parse("293.15 K", None).best.expect("kelvin");
        assert_eq!(kelvin.dimension, Some(Dimension::Temperature));
        assert_close(kelvin.value.unwrap(), 20.0);

        let japanese = parse("摂氏20度", None).best.expect("japanese celsius");
        assert_eq!(japanese.dimension, Some(Dimension::Temperature));
        assert_close(japanese.value.unwrap(), 20.0);

        let japanese_f = parse("華氏68度", None).best.expect("japanese fahrenheit");
        assert_close(japanese_f.value.unwrap(), 20.0);

        assert_eq!(humanize(&celsius, None), "20 °C");
        let round_trip_text = humanize(
            &japanese,
            Some(HumanizeCtx {
                locale: Some(Locale::Ja),
            }),
        );
        assert_eq!(round_trip_text, "摂氏20度");
        assert_close(
            parse(&round_trip_text, None)
                .best
                .expect("temperature round-trip")
                .value
                .unwrap(),
            20.0,
        );
    }

    #[test]
    fn parses_shaku_and_sun() {
        let parsed = parse(
            "5尺3寸",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Quantity);
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_eq!(best.provenance, Some(Provenance::JapaneseStatute));
        assert_eq!(best.approximate, Some(true));
        assert_close(best.value.unwrap(), 53.0 / 33.0);
        assert_eq!(parsed.findings.approximations.len(), 1);
        assert!(parsed.findings.skipped.is_empty());
    }

    #[test]
    fn parses_tatami_area() {
        let parsed = parse(
            "6帖",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m2"));
        assert_eq!(best.dimension, Some(Dimension::Area));
        assert_eq!(best.provenance, Some(Provenance::TradeCustom));
        assert_close(best.value.unwrap(), 9.72);
        assert_eq!(
            humanize(
                &best,
                Some(HumanizeCtx {
                    locale: Some(Locale::Ja)
                })
            ),
            "6帖 (approx.)"
        );
    }

    #[test]
    fn parses_gross_floor_square_meters() {
        let parsed = parse(
            "延床100㎡",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m2"));
        assert_eq!(best.dimension, Some(Dimension::Area));
        assert_eq!(best.provenance, Some(Provenance::SiMultiple));
        assert_eq!(best.approximate, Some(false));
        assert_close(best.value.unwrap(), 100.0);
    }

    #[test]
    fn parses_japanese_area_range() {
        let parsed = parse(
            "100-120㎡",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("m2"));
        assert_eq!(range.to.unit.as_deref(), Some("m2"));
        assert_close(range.from.value.unwrap(), 100.0);
        assert_close(range.to.value.unwrap(), 120.0);
    }

    #[test]
    fn parses_japanese_duration_range() {
        let parsed = parse(
            "2〜3日",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.dimension, Some(Dimension::Time));
        assert_eq!(range.to.dimension, Some(Dimension::Time));
        assert_close(range.from.value.unwrap(), 172_800.0);
        assert_close(range.to.value.unwrap(), 259_200.0);
    }

    #[test]
    fn parses_lingo_readme_metric_length_examples() {
        let cm = parse("180cm", None).best.expect("cm reading");
        assert_eq!(cm.unit.as_deref(), Some("m"));
        assert_close(cm.value.unwrap(), 1.8);

        let compound = parse("1m80", None).best.expect("compound reading");
        assert_eq!(compound.unit.as_deref(), Some("m"));
        assert_close(compound.value.unwrap(), 1.8);
    }

    #[test]
    fn parses_comma_decimal_mass() {
        let parsed = parse("1,5 kg", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("kg"));
        assert_eq!(best.dimension, Some(Dimension::Mass));
        assert_close(best.value.unwrap(), 1.5);
    }

    #[test]
    fn parses_between_mass_range() {
        let parsed = parse("between 5 and 10 kg", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("kg"));
        assert_eq!(range.to.unit.as_deref(), Some("kg"));
        assert_close(range.from.value.unwrap(), 5.0);
        assert_close(range.to.value.unwrap(), 10.0);
    }

    #[test]
    fn surfaces_ambiguous_grouped_decimal_number() {
        let parsed = parse("1,234", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Number);
        assert_close(best.value.unwrap(), 1234.0);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_close(parsed.alternatives[0].value.unwrap(), 1.234);
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "1,234");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousNumber
        );
        assert_eq!(parsed.findings.ambiguities[0].span.start, 0);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 5);
    }

    #[test]
    fn parses_number_words_and_unicode_fractions() {
        let words = parse("twenty-five kg", None).best.expect("words");
        assert_eq!(words.unit.as_deref(), Some("kg"));
        assert_close(words.value.unwrap(), 25.0);

        let fraction = parse("1½ cups", None).best.expect("fraction");
        assert_eq!(fraction.unit.as_deref(), Some("L"));
        assert_close(fraction.value.unwrap(), 1.5 * US_CUP_L);
    }

    #[test]
    fn parses_cjk_number_mass() {
        let parsed = parse(
            "三十五公斤",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("kg"));
        assert_close(best.value.unwrap(), 35.0);
    }

    #[test]
    fn parses_compound_imperial_mass() {
        let parsed = parse("2 lb 3 oz", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("kg"));
        assert_close(best.value.unwrap(), 0.992_233_375);
    }

    #[test]
    fn parses_simple_conversion_request() {
        let parsed = parse("72 in to cm", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("cm"));
        assert_eq!(best.dimension, Some(Dimension::Length));
        assert_close(best.value.unwrap(), 182.88);
    }

    #[test]
    fn parses_natural_duration_phrase() {
        let parsed = parse("an hour and a half", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("s"));
        assert_eq!(best.dimension, Some(Dimension::Time));
        assert_close(best.value.unwrap(), 5400.0);
    }

    #[test]
    fn parses_iso_duration_forms() {
        let hour_half = parse("PT1H30M", None).best.expect("iso duration");
        assert_eq!(hour_half.unit.as_deref(), Some("s"));
        assert_eq!(hour_half.dimension, Some(Dimension::Time));
        assert_close(hour_half.value.unwrap(), 5400.0);

        let days = parse("P2D", None).best.expect("day duration");
        assert_close(days.value.unwrap(), 172_800.0);
    }

    #[test]
    fn parses_compact_duration_forms() {
        let hour_half = parse("1h30", None).best.expect("compact duration");
        assert_eq!(hour_half.unit.as_deref(), Some("s"));
        assert_eq!(hour_half.dimension, Some(Dimension::Time));
        assert_close(hour_half.value.unwrap(), 5400.0);

        let days_hours = parse("2d4h", None).best.expect("compound duration");
        assert_close(days_hours.value.unwrap(), 187_200.0);
    }

    #[test]
    fn parses_clock_time_forms() {
        let afternoon = parse("3pm", None).best.expect("clock time");
        assert_eq!(afternoon.unit.as_deref(), Some("s"));
        assert_eq!(afternoon.dimension, Some(Dimension::Time));
        assert_close(afternoon.value.unwrap(), 15.0 * 3600.0);

        let twenty_four = parse("14:30", None).best.expect("24h clock");
        assert_close(twenty_four.value.unwrap(), 14.0 * 3600.0 + 30.0 * 60.0);

        let noon = parse("noon", None).best.expect("noon");
        assert_close(noon.value.unwrap(), 12.0 * 3600.0);
    }

    #[test]
    fn parses_clock_time_ranges() {
        let parsed = parse("3pm-4pm", None);
        let best = parsed.best.expect("time slot");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.dimension, Some(Dimension::Time));
        assert_eq!(range.to.dimension, Some(Dimension::Time));
        assert_close(range.from.value.unwrap(), 15.0 * 3600.0);
        assert_close(range.to.value.unwrap(), 16.0 * 3600.0);
    }

    #[test]
    fn parses_grouped_number() {
        let parsed = parse("1,234", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Number);
        assert_close(best.value.unwrap(), 1234.0);
        assert_eq!(parsed.alternatives.len(), 1);
    }

    #[test]
    fn suggests_context_implied_millimeters_for_plain_number() {
        let parsed = parse(
            "3640",
            Some(ParseCtx {
                expect: Some(Kind::Quantity),
                expected_dimension: Some(Dimension::Length),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(parsed.best.as_ref().unwrap().kind, Kind::Number);
        assert_eq!(parsed.alternatives.len(), 1);
        assert_eq!(parsed.alternatives[0].unit.as_deref(), Some("mm"));
        assert_eq!(parsed.findings.ambiguities.len(), 1);
    }

    #[test]
    fn parses_feet_inches() {
        let parsed = parse(
            "5ft 11",
            Some(ParseCtx {
                locale: Some(Locale::EnUs),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.provenance, Some(Provenance::InternationalExact));
        assert_close(best.value.unwrap(), 1.8034);
    }

    #[test]
    fn surfaces_cup_locale_ambiguity() {
        let parsed = parse(
            "1.5 cups",
            Some(ParseCtx {
                locale: Some(Locale::En),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("L"));
        assert_close(best.value.unwrap(), 1.5 * US_CUP_L);
        assert_eq!(parsed.alternatives.len(), 2);
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(3));
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "cups");
        assert_eq!(parsed.findings.ambiguities[0].span.start, 4);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 8);
    }

    #[test]
    fn parses_currency_codes_and_symbols() {
        let usd = parse("USD 12.34", None).best.expect("usd");
        assert_eq!(usd.unit.as_deref(), Some("USD"));
        assert_eq!(usd.dimension, Some(Dimension::Currency));
        assert_close(usd.value.unwrap(), 12.34);

        let yen = parse("¥1,234", None).best.expect("yen");
        assert_eq!(yen.unit.as_deref(), Some("JPY"));
        assert_close(yen.value.unwrap(), 1234.0);
    }

    #[test]
    fn surfaces_dollar_currency_ambiguity() {
        let parsed = parse("$12", None);
        let best = parsed.best.expect("dollar");
        assert_eq!(best.unit.as_deref(), Some("USD"));
        assert_eq!(best.dimension, Some(Dimension::Currency));
        assert_eq!(parsed.alternatives.len(), 2);
        assert_eq!(parsed.alternatives[0].unit.as_deref(), Some("CAD"));
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::AmbiguousCurrency
        );
        assert_eq!(parsed.findings.ambiguities[0].span.start, 0);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 1);
    }

    #[test]
    fn parses_currency_slang_and_minor_units() {
        let bucks = parse("12 bucks", None).best.expect("bucks");
        assert_eq!(bucks.unit.as_deref(), Some("USD"));
        assert_close(bucks.value.unwrap(), 12.0);
        assert_eq!(humanize(&bucks, None), "USD 12");

        let pence = parse("99 pence", None).best.expect("pence");
        assert_eq!(pence.unit.as_deref(), Some("GBP"));
        assert_close(pence.value.unwrap(), 0.99);

        let cents = parse("50 cents", None);
        let best = cents.best.expect("cents");
        assert_eq!(best.unit.as_deref(), Some("USD"));
        assert_close(best.value.unwrap(), 0.5);
        assert_eq!(cents.alternatives[0].unit.as_deref(), Some("EUR"));
        assert_eq!(
            cents.findings.ambiguities[0].code,
            IssueCode::AmbiguousCurrency
        );
        assert_eq!(cents.findings.ambiguities[0].ref_text, "cents");
        assert_eq!(cents.findings.ambiguities[0].span.start, 3);
        assert_eq!(cents.findings.ambiguities[0].span.end, 8);
    }

    #[test]
    fn converts_currency_with_supplied_rate() {
        let parsed = parse(
            "USD 10 to JPY",
            Some(ParseCtx {
                currency_rates: vec![CurrencyRate::new("USD", "JPY", 150.0)],
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("converted currency");
        assert_eq!(best.unit.as_deref(), Some("JPY"));
        assert_eq!(best.dimension, Some(Dimension::Currency));
        assert_close(best.value.unwrap(), 1500.0);

        let without_rate = parse("USD 10 to JPY", None);
        assert!(without_rate.best.is_none());
        assert_eq!(without_rate.findings.skipped[0].code, IssueCode::NoValue);
    }

    #[test]
    fn surfaces_slash_fraction_date_ambiguity() {
        let parsed = parse(
            "5/6",
            Some(ParseCtx {
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(parsed.best.as_ref().unwrap().kind, Kind::Number);
        assert_close(parsed.best.as_ref().unwrap().value.unwrap(), 5.0 / 6.0);
        assert_eq!(parsed.alternatives[0].kind, Kind::Date);
        assert_eq!(parsed.alternatives[0].date.as_deref(), Some("2026-05-06"));
        assert_eq!(parsed.findings.ambiguities[0].candidate_count, Some(2));
    }

    #[test]
    fn suggests_did_you_mean() {
        let parsed = parse("10 tsbo", None);
        assert!(parsed.best.is_none());
        assert_eq!(parsed.suggestions[0].from, "tsbo");
        assert_eq!(parsed.suggestions[0].to, "tsubo");
        assert_eq!(parsed.findings.skipped.len(), 1);
    }

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
    fn tokenizes_source_spans_for_findings() {
        let tokens = token_spans("USD 10 to JPY");
        assert_eq!(
            tokens
                .iter()
                .map(|token| token.text.as_str())
                .collect::<Vec<_>>(),
            vec!["USD", "10", "to", "JPY"]
        );

        let dollar = span_token_in("$12", "$");
        assert_eq!(dollar.start, 0);
        assert_eq!(dollar.end, 1);

        let cups = span_token_in("1.5 cups", "cups");
        assert_eq!(cups.start, 4);
        assert_eq!(cups.end, 8);
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

    #[test]
    fn typo_corrects_units_in_forgiving_mode() {
        let parsed = parse("5 meterz", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_close(best.value.unwrap(), 5.0);
        assert_eq!(parsed.suggestions[0].from, "meterz");
        assert_eq!(parsed.suggestions[0].to, "m");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::TypoCorrected
        );
        assert_eq!(parsed.findings.ambiguities[0].ref_text, "meterz");
        assert_eq!(parsed.findings.ambiguities[0].span.start, 2);
        assert_eq!(parsed.findings.ambiguities[0].span.end, 8);
    }

    #[test]
    fn confirm_mode_requires_typo_confirmation() {
        let parsed = parse(
            "5 meterz",
            Some(ParseCtx {
                strictness: Strictness::Confirm,
                ..ParseCtx::default()
            }),
        );
        assert!(parsed.best.is_none());
        assert_eq!(parsed.suggestions[0].to, "m");
        assert_eq!(parsed.findings.skipped[0].code, IssueCode::TypoCorrected);
        assert_eq!(parsed.findings.skipped[0].ref_text, "meterz");
        assert_eq!(parsed.findings.skipped[0].span.start, 2);
        assert_eq!(parsed.findings.skipped[0].span.end, 8);
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_next_friday_with_jiff() {
        let parsed = parse(
            "next friday",
            Some(ParseCtx {
                locale: Some(Locale::En),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Date);
        assert_eq!(best.date.as_deref(), Some("2026-07-24"));
        assert!(parsed.findings.skipped.is_empty());
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_in_days_with_jiff() {
        let parsed = parse(
            "in 3 days",
            Some(ParseCtx {
                locale: Some(Locale::En),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Date);
        assert_eq!(best.date.as_deref(), Some("2026-07-22"));
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_iso_date_with_jiff() {
        let parsed = parse(
            "2026-07-19",
            Some(ParseCtx {
                locale: Some(Locale::En),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Date);
        assert_eq!(best.date.as_deref(), Some("2026-07-19"));
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_japanese_relative_dates_with_jiff() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: Date::new(2026, 7, 19),
            ..ParseCtx::default()
        });

        assert_eq!(
            parse("明日", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-20")
        );
        assert_eq!(
            parse("3日後", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-22")
        );
        assert_eq!(
            parse("来週金曜日", ctx).best.unwrap().date.as_deref(),
            Some("2026-07-24")
        );
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_japanese_date_range_with_jiff() {
        let parsed = parse(
            "今日〜明日",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                reference_date: Date::new(2026, 7, 19),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("best reading");
        assert_eq!(best.kind, Kind::Range);
        let range = best.range.expect("range");
        assert_eq!(range.from.date.as_deref(), Some("2026-07-19"));
        assert_eq!(range.to.date.as_deref(), Some("2026-07-20"));
    }

    #[test]
    fn humanizes_japanese_length_round_trip() {
        let parsed = parse(
            "5尺3寸",
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.unwrap();
        assert_eq!(
            humanize(
                &best,
                Some(HumanizeCtx {
                    locale: Some(Locale::Ja)
                })
            ),
            "5尺3寸 (approx.)"
        );
    }
}
