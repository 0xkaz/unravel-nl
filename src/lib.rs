//! Deterministic parsing for informal natural-language quantities and values.
//!
//! This crate is an independent Rust implementation inspired by the public API
//! shape of `pascalorg/lingo` (MIT). It does not copy source code from that
//! project.

use std::borrow::Cow;

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

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json(text: &str) -> String {
    parsed_summary_json(&parse(text, None))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json_with_locale(text: &str, locale: &str) -> String {
    parsed_summary_json(&parse(
        text,
        Some(ParseCtx {
            locale: parse_locale_tag(locale),
            ..ParseCtx::default()
        }),
    ))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    parsed_summary_json(&parse(
        text,
        Some(parse_wasm_context(locale, expected_dimension, strictness)),
    ))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json(text: &str) -> String {
    parsed_matches_summary_json(text, &parse_all(text, None))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json_with_locale(text: &str, locale: &str) -> String {
    parsed_matches_summary_json(
        text,
        &parse_all(
            text,
            Some(ParseCtx {
                locale: parse_locale_tag(locale),
                ..ParseCtx::default()
            }),
        ),
    )
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_all_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    parsed_matches_summary_json(
        text,
        &parse_all(
            text,
            Some(parse_wasm_context(locale, expected_dimension, strictness)),
        ),
    )
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_dimensions_for_editor_json(text: &str) -> String {
    parsed_matches_summary_json(text, &parse_dimensions_for_editor(text, None))
}

#[cfg(feature = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn parse_dimensions_for_editor_json_with_context(
    text: &str,
    locale: &str,
    expected_dimension: &str,
    strictness: &str,
) -> String {
    let mut ctx = parse_wasm_context(locale, expected_dimension, strictness);
    ctx.purpose = ParsePurpose::DimensionEditor;
    parsed_matches_summary_json(text, &parse_dimensions_for_editor(text, Some(ctx)))
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
        aliases: &[
            "m", "meter", "meters", "metre", "metres", "metro", "metros", "mètre", "mètres", "米",
        ],
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
            "kilómetro",
            "kilómetros",
            "kilometro",
            "kilometros",
            "quilômetro",
            "quilômetros",
            "quilometro",
            "quilometros",
            "公里",
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
            "千克",
            "キログラム",
            "キロ",
            "kilogramo",
            "kilogramos",
            "quilograma",
            "quilogramas",
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
        aliases: &[
            "min", "mins", "minute", "minutes", "minuto", "minutos", "分钟",
        ],
        dimension: Dimension::Time,
        factor: 60.0,
        provenance: Provenance::SiMultiple,
        approximate: false,
    },
    UnitDef {
        id: "h",
        canonical_unit: "s",
        aliases: &[
            "h", "hr", "hrs", "hour", "hours", "hora", "horas", "heure", "heures", "時間", "小时",
        ],
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
            "metros cuadrados",
            "metro cuadrado",
            "mètres carrés",
            "mètre carré",
            "metres carres",
            "metre carre",
            "metros quadrados",
            "metro quadrado",
            "平方米",
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
        aliases: &[
            "L", "l", "liter", "liters", "litre", "litres", "litro", "litros", "升",
        ],
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

const FAST_UNIT_ALIASES: &[(&str, &str)] = &[
    ("m", "m"),
    ("meter", "m"),
    ("meters", "m"),
    ("metre", "m"),
    ("metres", "m"),
    ("米", "m"),
    ("km", "km"),
    ("kilometer", "km"),
    ("kilometers", "km"),
    ("kilometre", "km"),
    ("kilometres", "km"),
    ("公里", "km"),
    ("cm", "cm"),
    ("centimeter", "cm"),
    ("centimeters", "cm"),
    ("centimetre", "cm"),
    ("centimetres", "cm"),
    ("mm", "mm"),
    ("millimeter", "mm"),
    ("millimeters", "mm"),
    ("millimetre", "mm"),
    ("millimetres", "mm"),
    ("kg", "kg"),
    ("kilogram", "kg"),
    ("kilograms", "kg"),
    ("公斤", "kg"),
    ("千克", "kg"),
    ("キログラム", "kg"),
    ("キロ", "kg"),
    ("g", "g"),
    ("gram", "g"),
    ("grams", "g"),
    ("lb", "lb"),
    ("lbs", "lb"),
    ("pound", "lb"),
    ("pounds", "lb"),
    ("oz", "oz"),
    ("ounce", "oz"),
    ("ounces", "oz"),
    ("s", "s"),
    ("sec", "s"),
    ("second", "s"),
    ("seconds", "s"),
    ("min", "min"),
    ("mins", "min"),
    ("minute", "min"),
    ("minutes", "min"),
    ("分钟", "min"),
    ("h", "h"),
    ("hr", "h"),
    ("hour", "h"),
    ("hours", "h"),
    ("時間", "h"),
    ("小时", "h"),
    ("m2", "m2"),
    ("m^2", "m2"),
    ("m²", "m2"),
    ("㎡", "m2"),
    ("sqm", "m2"),
    ("square meter", "m2"),
    ("square meters", "m2"),
    ("square metre", "m2"),
    ("square metres", "m2"),
    ("平米", "m2"),
    ("平方米", "m2"),
    ("平方メートル", "m2"),
    ("L", "L"),
    ("l", "L"),
    ("liter", "L"),
    ("liters", "L"),
    ("litre", "L"),
    ("litres", "L"),
    ("litro", "L"),
    ("litros", "L"),
    ("升", "L"),
    ("ml", "mL"),
    ("mL", "mL"),
    ("milliliter", "mL"),
    ("milliliters", "mL"),
    ("millilitre", "mL"),
    ("millilitres", "mL"),
    ("GB", "GB"),
    ("MB", "MB"),
    ("KB", "kB"),
    ("Mbit/s", "Mbit/s"),
    ("Mb/s", "Mbit/s"),
    ("Mbps", "Mbit/s"),
    ("MB/s", "MB/s"),
    ("MBps", "MB/s"),
    ("bit/s", "bit/s"),
    ("b/s", "bit/s"),
    ("gpm", "gpm"),
    ("L/min", "L/min"),
    ("l/min", "L/min"),
    ("mAh", "mAh"),
    ("mah", "mAh"),
    ("uM", "μM"),
    ("μM", "μM"),
    ("µM", "μM"),
    ("Nm", "N*m"),
    ("N*m", "N*m"),
    ("N·m", "N*m"),
    ("lux", "lx"),
    ("lx", "lx"),
    ("mSv", "mSv"),
    ("MBq", "MBq"),
    ("inH2O", "inH2O"),
    ("inH₂O", "inH2O"),
    ("kgf/cm2", "kgf/cm2"),
    ("kgf/cm²", "kgf/cm2"),
];

const EDITOR_DIMENSION_LABELS: &[(&str, Dimension)] = &[
    ("延床", Dimension::Area),
    ("延べ床", Dimension::Area),
    ("床面積", Dimension::Area),
    ("敷地面積", Dimension::Area),
    ("面積", Dimension::Area),
    ("area", Dimension::Area),
    ("floorarea", Dimension::Area),
    ("sitearea", Dimension::Area),
    ("寸法", Dimension::Length),
    ("幅", Dimension::Length),
    ("高さ", Dimension::Length),
    ("壁厚", Dimension::Length),
    ("厚さ", Dimension::Length),
    ("長さ", Dimension::Length),
    ("奥行", Dimension::Length),
    ("奥行き", Dimension::Length),
    ("w", Dimension::Length),
    ("h", Dimension::Length),
    ("d", Dimension::Length),
    ("width", Dimension::Length),
    ("height", Dimension::Length),
    ("depth", Dimension::Length),
    ("length", Dimension::Length),
    ("wallthickness", Dimension::Length),
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
      "enum": ["quantity", "date", "range", "number", "recurrence"],
      "description": "Optional expected top-level reading kind."
    },
    "expected_dimension": {
      "type": "string",
      "enum": ["length", "area", "mass", "time", "volume", "currency", "temperature", "speed", "data", "data_rate", "flow_rate", "concentration", "acceleration", "force", "torque", "pressure", "power", "charge", "voltage", "current", "resistance", "illuminance", "radiation_equivalent_dose", "radioactivity"],
      "description": "Optional expected quantity dimension."
    },
    "number_format": {
      "type": "string",
      "enum": ["auto", "comma_decimal", "dot_decimal"],
      "default": "auto",
      "description": "Explicit numeric punctuation policy. Use comma_decimal for 1,5 and dot_decimal for 1,234 grouping."
    },
    "purpose": {
      "type": "string",
      "enum": ["general", "quantity", "number", "date", "recurrence", "dimension_editor"],
      "default": "general",
      "description": "Optional grammar dispatch hint. Use dimension_editor for UI fields that only accept building dimensions."
    },
    "accept": {
      "type": "object",
      "additionalProperties": false,
      "description": "Optional acceptance controls for broad parser shapes.",
      "properties": {
        "ranges": { "type": "boolean", "default": true },
        "conversions": { "type": "boolean", "default": true },
        "compounds": { "type": "boolean", "default": true },
        "fuzzy": { "type": "boolean", "default": true }
      }
    },
    "reference_date": {
      "type": "string",
      "format": "date",
      "description": "Civil reference date for relative dates. The parser never reads the host system clock or timezone."
    },
    "timezone": {
      "type": "string",
      "description": "Optional caller-supplied IANA timezone hint for adapter layers. The core parser does not infer timezone from the Rust host environment."
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
    "kind": { "type": "string", "enum": ["quantity", "date", "range", "number", "recurrence"] },
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
        "customKind": { "type": ["string", "null"] },
        "value": { "type": ["number", "null"] },
        "unit": { "type": ["string", "null"] },
        "dimension": {
          "anyOf": [
            { "$ref": "#/$defs/dimension" },
            { "type": "null" }
          ]
        },
        "date": { "type": ["string", "null"], "format": "date" },
        "recurrence": { "type": ["string", "null"], "description": "RRULE-style recurrence string for recurring expressions." },
        "timezone": { "type": ["string", "null"], "description": "Canonical timezone for timezone-normalized readings." },
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

#[cfg(feature = "wasm")]
fn parse_locale_tag(text: &str) -> Option<Locale> {
    match text {
        "" => None,
        "ja" | "ja-JP" => Some(Locale::Ja),
        "en" => Some(Locale::En),
        "en-US" => Some(Locale::EnUs),
        "en-GB" | "en-UK" => Some(Locale::EnGb),
        other => Some(Locale::Other(other.to_owned())),
    }
}

#[cfg(feature = "wasm")]
fn parse_wasm_context(locale: &str, expected_dimension: &str, strictness: &str) -> ParseCtx {
    ParseCtx {
        locale: parse_locale_tag(locale),
        expected_dimension: parse_dimension_tag(expected_dimension),
        strictness: parse_strictness_tag(strictness),
        ..ParseCtx::default()
    }
}

#[cfg(feature = "wasm")]
fn parse_dimension_tag(text: &str) -> Option<Dimension> {
    match text {
        "" => None,
        "length" => Some(Dimension::Length),
        "area" => Some(Dimension::Area),
        "mass" => Some(Dimension::Mass),
        "time" => Some(Dimension::Time),
        "volume" => Some(Dimension::Volume),
        "currency" => Some(Dimension::Currency),
        "temperature" => Some(Dimension::Temperature),
        "speed" => Some(Dimension::Speed),
        "data" => Some(Dimension::Data),
        "data_rate" => Some(Dimension::DataRate),
        "flow_rate" => Some(Dimension::FlowRate),
        "concentration" => Some(Dimension::Concentration),
        "acceleration" => Some(Dimension::Acceleration),
        "force" => Some(Dimension::Force),
        "torque" => Some(Dimension::Torque),
        "pressure" => Some(Dimension::Pressure),
        "power" => Some(Dimension::Power),
        "charge" => Some(Dimension::Charge),
        "voltage" => Some(Dimension::Voltage),
        "current" => Some(Dimension::Current),
        "resistance" => Some(Dimension::Resistance),
        "illuminance" => Some(Dimension::Illuminance),
        "radiation_equivalent_dose" => Some(Dimension::RadiationEquivalentDose),
        "radioactivity" => Some(Dimension::Radioactivity),
        _ => None,
    }
}

#[cfg(feature = "wasm")]
fn parse_strictness_tag(text: &str) -> Strictness {
    match text {
        "confirm" => Strictness::Confirm,
        "strict" => Strictness::Strict,
        _ => Strictness::Forgiving,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    Quantity,
    Date,
    Range,
    Number,
    Recurrence,
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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NumberFormat {
    #[default]
    Auto,
    CommaDecimal,
    DotDecimal,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParsePurpose {
    #[default]
    General,
    Quantity,
    Number,
    Date,
    Recurrence,
    DimensionEditor,
}

impl ParsePurpose {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Quantity => "quantity",
            Self::Number => "number",
            Self::Date => "date",
            Self::Recurrence => "recurrence",
            Self::DimensionEditor => "dimension_editor",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptOptions {
    pub ranges: bool,
    pub conversions: bool,
    pub compounds: bool,
    pub fuzzy: bool,
}

impl Default for AcceptOptions {
    fn default() -> Self {
        Self {
            ranges: true,
            conversions: true,
            compounds: true,
            fuzzy: true,
        }
    }
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
    pub number_format: NumberFormat,
    pub purpose: ParsePurpose,
    pub accept: AcceptOptions,
    pub reference_date: Option<Date>,
    pub timezone: Option<String>,
    pub strictness: Strictness,
    pub currency_rates: Vec<CurrencyRate>,
    pub custom_units: Vec<CustomUnit>,
    pub fuzzy_profiles: Vec<FuzzyProfile>,
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
    pub kind_id: Option<String>,
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
            kind_id: None,
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

    pub fn kind(mut self, kind_id: &str) -> Self {
        self.kind_id = Some(kind_id.to_owned());
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuzzyProfile {
    pub id: String,
    pub dimension: Dimension,
    pub unit: String,
    pub terms: Vec<FuzzyTerm>,
}

impl FuzzyProfile {
    pub fn new(id: &str, dimension: Dimension, unit: &str, terms: &[FuzzyTerm]) -> Self {
        Self {
            id: id.to_owned(),
            dimension,
            unit: unit.to_owned(),
            terms: terms.to_vec(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FuzzyTerm {
    pub term: String,
    pub low: f64,
    pub high: f64,
}

impl FuzzyTerm {
    pub fn new(term: &str, low: f64, high: f64) -> Self {
        Self {
            term: term.to_owned(),
            low,
            high,
        }
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
pub struct ParsedMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub parsed: Parsed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    pub kind: Kind,
    pub custom_kind: Option<String>,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub dimension: Option<Dimension>,
    pub date: Option<String>,
    pub recurrence: Option<String>,
    pub timezone: Option<String>,
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
            custom_kind: None,
            value: Some(value),
            unit: Some(unit.to_owned()),
            dimension: Some(dimension),
            date: None,
            recurrence: None,
            timezone: None,
            range: None,
            provenance: Some(provenance),
            approximate: Some(approximate),
            confidence: Some(confidence),
        }
    }

    pub fn number(value: f64, confidence: f64) -> Self {
        Self {
            kind: Kind::Number,
            custom_kind: None,
            value: Some(value),
            unit: None,
            dimension: None,
            date: None,
            recurrence: None,
            timezone: None,
            range: None,
            provenance: None,
            approximate: Some(false),
            confidence: Some(confidence),
        }
    }

    pub fn date(date: Date, confidence: f64) -> Self {
        Self {
            kind: Kind::Date,
            custom_kind: None,
            value: None,
            unit: None,
            dimension: None,
            date: Some(date.iso()),
            recurrence: None,
            timezone: None,
            range: None,
            provenance: None,
            approximate: Some(false),
            confidence: Some(confidence),
        }
    }

    pub fn range(from: Reading, to: Reading, confidence: f64) -> Self {
        Self {
            kind: Kind::Range,
            custom_kind: None,
            value: None,
            unit: None,
            dimension: None,
            date: None,
            recurrence: None,
            timezone: None,
            range: Some(Box::new(RangeReading { from, to })),
            provenance: None,
            approximate: Some(false),
            confidence: Some(confidence),
        }
    }

    pub fn recurrence(rrule: &str, confidence: f64) -> Self {
        Self {
            kind: Kind::Recurrence,
            custom_kind: None,
            value: None,
            unit: None,
            dimension: None,
            date: None,
            recurrence: Some(rrule.to_owned()),
            timezone: None,
            range: None,
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

#[derive(Clone, Debug, PartialEq)]
pub struct CompletionReading {
    pub text: String,
    pub reading: Reading,
    pub score: f64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceView {
    pub object: String,
    pub summary: String,
    pub fields: Vec<ResourceField>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceField {
    pub name: String,
    pub value: String,
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
    TimezoneUnsupported,
    RecurrenceUnsupported,
    RejectedByPolicy,
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
            Self::TimezoneUnsupported => "TIMEZONE_UNSUPPORTED",
            Self::RecurrenceUnsupported => "RECURRENCE_UNSUPPORTED",
            Self::RejectedByPolicy => "REJECTED_BY_POLICY",
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
}

impl IssueSeverity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RankedIssue {
    pub code: IssueCode,
    pub severity: IssueSeverity,
    pub rank: u16,
    pub recoverable: bool,
    pub ref_text: String,
    pub reason: String,
    pub span: Span,
}

pub fn ranked_findings(parsed: &Parsed) -> Vec<RankedIssue> {
    let mut issues = Vec::new();

    for issue in &parsed.findings.skipped {
        issues.push(ranked_issue(
            issue.code,
            issue.ref_text.clone(),
            issue.reason.clone(),
            issue.span.clone(),
        ));
    }
    for issue in &parsed.findings.ambiguities {
        issues.push(ranked_issue(
            issue.code,
            issue.ref_text.clone(),
            issue.reason.clone(),
            issue.span.clone(),
        ));
    }
    for issue in &parsed.findings.approximations {
        issues.push(ranked_issue(
            issue.code,
            issue.ref_text.clone(),
            issue.reason.clone(),
            issue.span.clone(),
        ));
    }

    issues.sort_by(|a, b| {
        b.rank
            .cmp(&a.rank)
            .then_with(|| a.ref_text.cmp(&b.ref_text))
    });
    issues
}

fn ranked_issue(code: IssueCode, ref_text: String, reason: String, span: Span) -> RankedIssue {
    RankedIssue {
        code,
        severity: issue_severity(code),
        rank: issue_rank(code),
        recoverable: issue_recoverable(code),
        ref_text,
        reason,
        span,
    }
}

fn issue_severity(code: IssueCode) -> IssueSeverity {
    match code {
        IssueCode::Empty
        | IssueCode::NoValue
        | IssueCode::UnknownUnit
        | IssueCode::TimezoneUnsupported
        | IssueCode::RecurrenceUnsupported
        | IssueCode::RejectedByPolicy => IssueSeverity::Error,
        IssueCode::TypoCorrected
        | IssueCode::AmbiguousNumber
        | IssueCode::AmbiguousDate
        | IssueCode::AmbiguousUnit
        | IssueCode::AmbiguousCurrency
        | IssueCode::Approximation => IssueSeverity::Warning,
        IssueCode::UnitAssumed => IssueSeverity::Info,
    }
}

fn issue_rank(code: IssueCode) -> u16 {
    match code {
        IssueCode::Empty | IssueCode::NoValue => 100,
        IssueCode::TimezoneUnsupported
        | IssueCode::RecurrenceUnsupported
        | IssueCode::RejectedByPolicy => 90,
        IssueCode::UnknownUnit => 80,
        IssueCode::TypoCorrected => 65,
        IssueCode::AmbiguousDate
        | IssueCode::AmbiguousNumber
        | IssueCode::AmbiguousUnit
        | IssueCode::AmbiguousCurrency => 55,
        IssueCode::UnitAssumed => 40,
        IssueCode::Approximation => 30,
    }
}

fn issue_recoverable(code: IssueCode) -> bool {
    !matches!(code, IssueCode::Empty | IssueCode::NoValue)
}

struct AmbiguousParse {
    best: Option<Reading>,
    alternatives: Vec<Reading>,
    ambiguity: Ambiguity,
}

struct ParsedReading {
    reading: Reading,
    approximations: Vec<Approximation>,
}

pub fn parse(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, &ctx);

    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }

    match ctx.purpose {
        ParsePurpose::General => parse_normalized_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Quantity => parse_quantity_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Number => parse_number_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Date => parse_date_fast_into(trimmed, &ctx, &mut parsed),
        ParsePurpose::Recurrence => parse_recurrence_fast_into(trimmed, &mut parsed),
        ParsePurpose::DimensionEditor => parse_editor_dimension_into(trimmed, &ctx, &mut parsed),
    }
    parsed
}

pub fn parse_quantity_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    parse_quantity_fast_with_ctx(text, &ctx)
}

fn parse_quantity_fast_with_ctx(text: &str, ctx: &ParseCtx) -> Parsed {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_quantity_fast_into(trimmed, ctx, &mut parsed);
    parsed
}

pub fn parse_number_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    parse_number_fast_with_ctx(text, &ctx)
}

fn parse_number_fast_with_ctx(text: &str, ctx: &ParseCtx) -> Parsed {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_number_fast_into(trimmed, ctx, &mut parsed);
    parsed
}

pub fn parse_recurrence_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, &ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_recurrence_fast_into(trimmed, &mut parsed);
    parsed
}

pub fn parse_date_fast(text: &str, ctx: Option<ParseCtx>) -> Parsed {
    let ctx = ctx.unwrap_or_default();
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    let mut parsed = parsed_shell(text, &ctx);
    if trimmed.is_empty() {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "empty input"));
        return parsed;
    }
    parse_date_fast_into(trimmed, &ctx, &mut parsed);
    parsed
}

fn parsed_shell(text: &str, ctx: &ParseCtx) -> Parsed {
    Parsed {
        input: text.to_owned(),
        locale: ctx.locale.clone(),
        best: None,
        alternatives: Vec::new(),
        suggestions: Vec::new(),
        findings: Findings::default(),
    }
}

pub fn parse_all(text: &str, ctx: Option<ParseCtx>) -> Vec<ParsedMatch> {
    let ctx = ctx.unwrap_or_default();
    let mut matches = Vec::new();
    for_clause_spans(text, |start, end| {
        push_clause_matches(&mut matches, text, start, end, &ctx);
    });
    sorted_non_overlapping_matches(matches)
}

fn push_clause_matches(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
) {
    match broad_clause_dispatch(&text[start..end]) {
        BroadClauseDispatch::None => {
            push_numeric_window_matches(matches, text, start, end, ctx);
            return;
        }
        BroadClauseDispatch::Prefix => {
            match push_broad_clause_match(matches, text, start, end, ctx) {
                Some(true) => return,
                Some(false) if clause_has_numeric_candidate(text, start, end) => {
                    matches.pop();
                }
                Some(false) => return,
                None => {}
            }
            push_numeric_window_matches(matches, text, start, end, ctx);
            return;
        }
        BroadClauseDispatch::Short => {}
    }

    let mut first_numeric = None;
    let mut numeric_count = 0usize;
    for_numeric_candidate_spans(text, start, end, |candidate| {
        numeric_count += 1;
        if first_numeric.is_none() {
            first_numeric = Some(candidate);
        }
        true
    });

    let clause_bounds = trimmed_bounds(text, start, end);
    if numeric_count == 1
        && let Some(candidate) = first_numeric
        && Some((candidate.start, candidate.end)) == clause_bounds
    {
        let _ = push_parsed_match(matches, text, candidate, ctx);
        return;
    }

    match push_broad_clause_match(matches, text, start, end, ctx) {
        Some(true) => return,
        Some(false) if numeric_count > 0 => {
            matches.pop();
        }
        _ => {}
    }

    if let Some(candidate) = first_numeric {
        if numeric_count == 1 {
            let _ = push_parsed_match(matches, text, candidate, ctx);
        } else {
            push_numeric_window_matches(matches, text, start, end, ctx);
        }
    }
}

fn push_broad_clause_match(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
) -> Option<bool> {
    push_parsed_match(
        matches,
        text,
        CandidateSpan {
            start,
            end,
            parser: CandidateParser::Broad,
        },
        ctx,
    )
}

fn push_numeric_window_matches(
    matches: &mut Vec<ParsedMatch>,
    text: &str,
    start: usize,
    end: usize,
    ctx: &ParseCtx,
) {
    for_numeric_candidate_spans(text, start, end, |candidate| {
        let _ = push_parsed_match(matches, text, candidate, ctx);
        true
    });
}

pub fn parse_dimensions_for_editor(text: &str, ctx: Option<ParseCtx>) -> Vec<ParsedMatch> {
    let mut ctx = ctx.unwrap_or_default();
    ctx.purpose = ParsePurpose::DimensionEditor;
    ctx.expect = Some(Kind::Quantity);

    let mut matches = Vec::new();
    for_clause_spans(text, |clause_start, clause_end| {
        for_numeric_candidate_spans(text, clause_start, clause_end, |candidate| {
            if candidate_starts_with_currency(text, candidate.start) {
                return true;
            }
            push_editor_dimension_match(&mut matches, text, candidate, clause_start, &ctx);
            true
        });
    });

    sorted_non_overlapping_matches(matches)
}

fn sorted_non_overlapping_matches(mut matches: Vec<ParsedMatch>) -> Vec<ParsedMatch> {
    if matches.len() <= 1 {
        return matches;
    }

    matches.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| right.end.cmp(&left.end))
    });

    let mut non_overlapping: Vec<ParsedMatch> = Vec::with_capacity(matches.len());
    for candidate in matches {
        if non_overlapping.last().is_some_and(|existing| {
            spans_overlap(existing.start, existing.end, candidate.start, candidate.end)
        }) {
            continue;
        }
        non_overlapping.push(candidate);
    }
    non_overlapping
}

fn push_parsed_match(
    matches: &mut Vec<ParsedMatch>,
    source: &str,
    candidate: CandidateSpan,
    ctx: &ParseCtx,
) -> Option<bool> {
    let start = candidate.start;
    let end = candidate.end;
    if start >= end
        || matches
            .last()
            .is_some_and(|item| item.start == start && item.end == end)
    {
        return None;
    }
    let text = source.get(start..end).map(str::trim)?;
    if text.is_empty() {
        return None;
    }
    let parsed = match candidate.parser {
        CandidateParser::Broad => parse(text, Some(ctx.clone())),
        CandidateParser::TokenWindow => parse_token_window(text, ctx),
    };
    if !parsed_has_actionable_match(&parsed) {
        return None;
    }
    let suppresses_inner_tokens = parsed_suppresses_inner_tokens(&parsed);
    let leading = source[start..end].len() - source[start..end].trim_start().len();
    let trailing = source[start..end].len() - source[start..end].trim_end().len();
    matches.push(ParsedMatch {
        start: start + leading,
        end: end - trailing,
        text: text.to_owned(),
        parsed,
    });
    Some(suppresses_inner_tokens)
}

fn parsed_suppresses_inner_tokens(parsed: &Parsed) -> bool {
    parsed.best.is_some()
        || !parsed.alternatives.is_empty()
        || !parsed.findings.ambiguities.is_empty()
        || !parsed.findings.approximations.is_empty()
        || parsed.findings.skipped.iter().any(|issue| {
            matches!(
                issue.code,
                IssueCode::Approximation
                    | IssueCode::TypoCorrected
                    | IssueCode::TimezoneUnsupported
                    | IssueCode::RecurrenceUnsupported
            )
        })
}

fn push_editor_dimension_match(
    matches: &mut Vec<ParsedMatch>,
    source: &str,
    candidate: CandidateSpan,
    clause_start: usize,
    ctx: &ParseCtx,
) {
    let start = candidate.start;
    let end = candidate.end;
    if start >= end
        || matches
            .last()
            .is_some_and(|item| item.start == start && item.end == end)
    {
        return;
    }
    let Some(text) = source.get(start..end).map(str::trim) else {
        return;
    };
    if text.is_empty() {
        return;
    }

    let hint = editor_dimension_hint(source, clause_start, start);
    if hint.is_none() && candidate_has_identifier_prefix(source, clause_start, start) {
        return;
    }
    let local_ctx_storage;
    let local_ctx = if ctx.expected_dimension.is_none() {
        if let Some(hint) = hint {
            let mut updated = ctx.clone();
            updated.expected_dimension = Some(hint);
            local_ctx_storage = updated;
            &local_ctx_storage
        } else {
            ctx
        }
    } else {
        ctx
    };
    let mut parsed = parsed_shell(text, local_ctx);
    parse_editor_dimension_into(text, local_ctx, &mut parsed);
    if !parsed_is_editor_dimension(&parsed, hint, ctx.expected_dimension) {
        return;
    }

    let leading = source[start..end].len() - source[start..end].trim_start().len();
    let trailing = source[start..end].len() - source[start..end].trim_end().len();
    matches.push(ParsedMatch {
        start: start + leading,
        end: end - trailing,
        text: text.to_owned(),
        parsed,
    });
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct CandidateSpan {
    start: usize,
    end: usize,
    parser: CandidateParser,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateParser {
    Broad,
    TokenWindow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BroadClauseDispatch {
    None,
    Short,
    Prefix,
}

fn broad_clause_dispatch(clause: &str) -> BroadClauseDispatch {
    let trimmed = clause.trim();
    if trimmed.is_empty() {
        return BroadClauseDispatch::None;
    }
    if trimmed.starts_with('約') {
        return BroadClauseDispatch::Prefix;
    }
    let lower = ascii_lower_cow(trimmed);
    if lower.starts_with("between ")
        || lower.starts_with("from ")
        || lower.starts_with("about ")
        || lower.starts_with("around ")
        || lower.starts_with("roughly ")
        || lower.starts_with("approximately ")
    {
        BroadClauseDispatch::Prefix
    } else if has_at_most_three_words(trimmed) {
        BroadClauseDispatch::Short
    } else {
        BroadClauseDispatch::None
    }
}

fn has_at_most_three_words(text: &str) -> bool {
    text.split_whitespace().take(4).count() <= 3
}

fn clause_has_numeric_candidate(text: &str, start: usize, end: usize) -> bool {
    let mut found = false;
    for_numeric_candidate_spans(text, start, end, |_| {
        found = true;
        false
    });
    found
}

fn for_clause_spans<F>(text: &str, mut emit: F)
where
    F: FnMut(usize, usize),
{
    let mut start = 0;
    for (idx, ch) in text.char_indices() {
        if is_clause_separator(text, idx, ch) {
            if start < idx {
                emit(start, idx);
            }
            start = idx + ch.len_utf8();
        }
    }
    if start < text.len() {
        emit(start, text.len());
    }
}

fn trimmed_bounds(text: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let span = text.get(start..end)?;
    let leading = span.len() - span.trim_start().len();
    let trailing = span.len() - span.trim_end().len();
    let trimmed_start = start + leading;
    let trimmed_end = end - trailing;
    (trimmed_start < trimmed_end).then_some((trimmed_start, trimmed_end))
}

fn is_clause_separator(text: &str, idx: usize, ch: char) -> bool {
    match ch {
        '、' | ';' | '；' | '\n' | '\t' => true,
        ',' => {
            let previous = text[..idx].chars().rev().find(|ch| !ch.is_whitespace());
            let next = text[idx + ch.len_utf8()..]
                .chars()
                .find(|ch| !ch.is_whitespace());
            !matches!((previous, next), (Some(left), Some(right)) if left.is_ascii_digit() && right.is_ascii_digit())
        }
        _ => false,
    }
}

fn for_numeric_candidate_spans<F>(text: &str, start: usize, end: usize, mut emit: F)
where
    F: FnMut(CandidateSpan) -> bool,
{
    let mut cursor = start;
    while cursor < end {
        let Some((idx, ch)) = text[cursor..end].char_indices().next() else {
            break;
        };
        let abs = cursor + idx;
        if is_candidate_start_at(text, abs, ch) {
            let candidate_end = candidate_end(text, abs, end);
            if candidate_end > abs {
                let should_continue = emit(CandidateSpan {
                    start: abs,
                    end: candidate_end,
                    parser: CandidateParser::TokenWindow,
                });
                if !should_continue {
                    return;
                }
            }
            cursor = candidate_end.max(abs + ch.len_utf8());
        } else {
            cursor = abs + ch.len_utf8();
        }
    }
}

fn parse_token_window(text: &str, ctx: &ParseCtx) -> Parsed {
    let quantity = parse_quantity_fast_with_ctx(text, ctx);
    if parsed_has_actionable_match(&quantity) {
        return quantity;
    }
    parse_number_fast_with_ctx(text, ctx)
}

fn parsed_has_actionable_match(parsed: &Parsed) -> bool {
    parsed.best.is_some()
        || !parsed.alternatives.is_empty()
        || !parsed.suggestions.is_empty()
        || !parsed.findings.ambiguities.is_empty()
        || !parsed.findings.approximations.is_empty()
        || parsed
            .findings
            .skipped
            .iter()
            .any(|issue| !matches!(issue.code, IssueCode::NoValue | IssueCode::UnknownUnit))
}

fn candidate_starts_with_currency(text: &str, start: usize) -> bool {
    text[start..]
        .chars()
        .next()
        .is_some_and(|ch| matches!(ch, '$' | '€' | '£' | '¥' | '￥'))
}

fn parsed_is_editor_dimension(
    parsed: &Parsed,
    hint: Option<Dimension>,
    expected_dimension: Option<Dimension>,
) -> bool {
    let allowed_dimension = expected_dimension.or(hint);
    if let Some(best) = parsed.best.as_ref() {
        if reading_is_dimension_quantity(best, allowed_dimension) {
            return true;
        }
        if best.kind == Kind::Number {
            return allowed_dimension == Some(Dimension::Length)
                && parsed
                    .alternatives
                    .iter()
                    .any(|reading| reading.dimension == Some(Dimension::Length));
        }
    }
    parsed
        .alternatives
        .iter()
        .any(|reading| reading_is_dimension_quantity(reading, allowed_dimension))
}

fn candidate_has_identifier_prefix(
    source: &str,
    clause_start: usize,
    candidate_start: usize,
) -> bool {
    source
        .get(clause_start..candidate_start)
        .and_then(|before| before.chars().next_back())
        .is_some_and(is_embedded_identifier_char)
}

fn is_embedded_identifier_char(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphanumeric() || matches!(ch, 'Ａ'..='Ｚ' | 'ａ'..='ｚ' | '０'..='９')
}

fn reading_is_dimension_quantity(reading: &Reading, expected_dimension: Option<Dimension>) -> bool {
    if reading.kind != Kind::Quantity {
        return false;
    }
    match reading.dimension {
        Some(Dimension::Length | Dimension::Area) => match expected_dimension {
            Some(dimension) => reading.dimension == Some(dimension),
            None => true,
        },
        _ => false,
    }
}

fn editor_dimension_hint(
    source: &str,
    clause_start: usize,
    candidate_start: usize,
) -> Option<Dimension> {
    let before = source.get(clause_start..candidate_start)?.trim_end();
    let before = before
        .trim_end_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(ch, ':' | '：' | '=' | '＝' | '-' | 'ー' | '―' | '–' | '—')
        })
        .trim_end();
    let lower = ascii_lower_cow(before);
    let mut compact = None;

    for (label, dimension) in EDITOR_DIMENSION_LABELS {
        if editor_label_matches(before, lower.as_ref(), &mut compact, label) {
            return Some(*dimension);
        }
    }
    None
}

fn editor_label_matches(
    before: &str,
    lower_before: &str,
    compact: &mut Option<String>,
    label: &str,
) -> bool {
    if label.len() == 1 && label.as_bytes()[0].is_ascii_alphabetic() {
        let trimmed = before.trim_end();
        let Some((idx, ch)) = trimmed.char_indices().next_back() else {
            return false;
        };
        return ch.eq_ignore_ascii_case(&char::from(label.as_bytes()[0]))
            && trimmed[..idx]
                .chars()
                .next_back()
                .is_none_or(|previous| !previous.is_ascii_alphanumeric());
    }
    if matches!(label, "area" | "width" | "height" | "depth" | "length") {
        return ascii_label_suffix_matches(lower_before, label);
    }
    if let Some(spaced_label) = compound_editor_label(label) {
        if ascii_label_suffix_matches(lower_before, spaced_label)
            || ascii_label_suffix_matches(lower_before, label)
        {
            return true;
        }
        let compact = compact.get_or_insert_with(|| {
            lower_before
                .chars()
                .filter(|ch| !ch.is_whitespace())
                .collect()
        });
        return ascii_label_suffix_matches(compact, label);
    }
    lower_before.ends_with(label)
}

fn compound_editor_label(label: &str) -> Option<&'static str> {
    match label {
        "floorarea" => Some("floor area"),
        "sitearea" => Some("site area"),
        "wallthickness" => Some("wall thickness"),
        _ => None,
    }
}

fn ascii_label_suffix_matches(lower_before: &str, label: &str) -> bool {
    let lower = lower_before.trim_end();
    if !lower.ends_with(label) {
        return false;
    }
    let prefix = &lower[..lower.len() - label.len()];
    prefix
        .chars()
        .next_back()
        .is_none_or(|previous| !previous.is_ascii_alphanumeric())
}

fn is_candidate_start_at(text: &str, idx: usize, ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(ch, '０'..='９' | '$' | '€' | '£' | '¥' | '￥')
        || is_cjk_number_char(ch)
        || (ch == '約'
            && text[idx + ch.len_utf8()..]
                .chars()
                .next()
                .is_some_and(is_candidate_number_start))
}

fn is_candidate_number_start(ch: char) -> bool {
    ch.is_ascii_digit()
        || matches!(ch, '０'..='９' | '$' | '€' | '£' | '¥' | '￥')
        || is_cjk_number_char(ch)
}

fn candidate_end(text: &str, start: usize, limit: usize) -> usize {
    let mut end = start;
    let mut saw_unit = false;
    let mut saw_number = false;
    let mut previous_was_digit = false;
    let mut after_number_gap = false;

    for (idx, ch) in text[start..limit].char_indices() {
        let abs = start + idx;
        if idx > 0 && is_candidate_boundary(text, abs, ch) {
            break;
        }
        if idx == 0 && ch == '約' {
            end = abs + ch.len_utf8();
            continue;
        }
        if is_numeric_body_char(ch) {
            saw_number = true;
            previous_was_digit = is_digit_like(ch);
            after_number_gap = false;
            end = abs + ch.len_utf8();
            continue;
        }
        if is_candidate_space(ch) {
            if previous_was_digit && next_nonspace_is_digit(text, abs + ch.len_utf8(), limit) {
                end = abs + ch.len_utf8();
                continue;
            }
            if saw_number && !saw_unit && next_nonspace_is_unit(text, abs + ch.len_utf8(), limit) {
                after_number_gap = true;
                end = abs + ch.len_utf8();
                continue;
            }
            break;
        }
        if saw_number && is_candidate_unit_char(ch) {
            saw_unit = true;
            previous_was_digit = false;
            after_number_gap = false;
            end = abs + ch.len_utf8();
            continue;
        }
        if after_number_gap {
            break;
        }
        if idx == 0 && matches!(ch, '$' | '€' | '£' | '¥' | '￥') {
            end = abs + ch.len_utf8();
            continue;
        }
        break;
    }

    while end > start
        && text[start..end]
            .chars()
            .last()
            .is_some_and(char::is_whitespace)
    {
        let Some((idx, _)) = text[start..end].char_indices().last() else {
            break;
        };
        end = start + idx;
    }

    if !saw_number {
        return start;
    }
    end
}

fn is_digit_like(ch: char) -> bool {
    ch.is_ascii_digit() || matches!(ch, '０'..='９') || is_cjk_number_char(ch)
}

fn is_numeric_body_char(ch: char) -> bool {
    is_digit_like(ch)
        || matches!(
            ch,
            '.' | ',' | '+' | '-' | '．' | '，' | '万' | '億' | '兆' | '/' | '／'
        )
}

fn is_candidate_space(ch: char) -> bool {
    ch.is_whitespace() || matches!(ch, '\u{00A0}' | '\u{202F}' | '\u{2009}' | '\u{2007}')
}

fn next_nonspace_is_digit(text: &str, mut cursor: usize, limit: usize) -> bool {
    while cursor < limit {
        let Some(ch) = text[cursor..limit].chars().next() else {
            return false;
        };
        if is_candidate_space(ch) {
            cursor += ch.len_utf8();
            continue;
        }
        return is_digit_like(ch);
    }
    false
}

fn next_nonspace_is_unit(text: &str, mut cursor: usize, limit: usize) -> bool {
    while cursor < limit {
        let Some(ch) = text[cursor..limit].chars().next() else {
            return false;
        };
        if is_candidate_space(ch) {
            cursor += ch.len_utf8();
            continue;
        }
        return is_candidate_unit_char(ch);
    }
    false
}

fn is_candidate_unit_char(ch: char) -> bool {
    ch.is_ascii_alphabetic()
        || matches!(ch, 'Ａ'..='Ｚ' | 'ａ'..='ｚ')
        || matches!(
            ch,
            'μ' | 'µ'
                | '°'
                | '%'
                | '/'
                | '^'
                | '²'
                | '³'
                | '₂'
                | '尺'
                | '寸'
                | '間'
                | '帖'
                | '畳'
                | '坪'
                | '平'
                | '米'
                | '㎡'
                | '円'
                | '度'
                | 'キ'
                | 'ロ'
                | 'グ'
                | 'ラ'
                | 'ム'
                | '公'
                | '斤'
                | '千'
                | '克'
                | 'リ'
                | 'ッ'
                | 'ト'
                | 'ル'
                | '半'
        )
}

fn is_candidate_boundary(text: &str, idx: usize, ch: char) -> bool {
    if matches!(ch, '、' | ';' | '；' | '\n' | '\t' | '(' | ')' | '[' | ']') {
        return true;
    }
    if matches!(ch, '×' | '*') {
        return text[idx + ch.len_utf8()..]
            .chars()
            .find(|next| !next.is_whitespace())
            .is_some_and(is_candidate_number_start);
    }
    false
}

fn spans_overlap(left_start: usize, left_end: usize, right_start: usize, right_end: usize) -> bool {
    left_start < right_end && right_start < left_end
}

fn parse_normalized_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    let features = InputFeatures::new(trimmed);

    if let Some(result) = parse_qualified_reading(trimmed, ctx) {
        if ctx.strictness == Strictness::Strict {
            parsed.findings.skipped.push(skipped_with_span(
                trimmed,
                "approximate qualifier requires confirmation in strict mode",
                IssueCode::Approximation,
                span(trimmed),
            ));
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    if let Some(result) = parse_fuzzy_reading(trimmed, ctx) {
        if ctx.strictness == Strictness::Strict {
            parsed.findings.skipped.push(skipped_with_span(
                trimmed,
                "fuzzy reading requires confirmation in strict mode",
                IssueCode::Approximation,
                span(trimmed),
            ));
        } else if !ctx.accept.fuzzy {
            reject_candidate(
                parsed,
                trimmed,
                result.reading,
                "fuzzy readings are disabled by acceptance policy",
            );
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    if features.has_slash
        && let Some(ambiguous) = parse_ambiguous_slash_date_or_fraction(trimmed, ctx)
    {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
        return;
    }

    if features.maybe_date
        && let Some(reading) = parse_relative_date(trimmed, ctx)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_recurrence
        && let Some(reading) = parse_recurrence(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_range
        && let Some(reading) = parse_plus_minus_range(trimmed, ctx)
    {
        if !ctx.accept.ranges {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "range readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_range
        && let Some(reading) = parse_upper_bound_range(trimmed, ctx)
    {
        if !ctx.accept.ranges {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "range readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_range
        && let Some(reading) = parse_range(trimmed, ctx)
    {
        if !ctx.accept.ranges {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "range readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_conversion
        && let Some(reading) = parse_conversion_request(trimmed, ctx)
    {
        if !ctx.accept.conversions {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "conversion readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_japanese_length
        && let Some(reading) = parse_japanese_length(trimmed)
    {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Japanese customary length converted to SI meters.",
        ));
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_tatami
        && let Some(reading) = parse_tatami_area(trimmed)
    {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Tatami area uses a trade-custom regional approximation of 1.62 m2.",
        ));
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_tsubo
        && let Some(reading) = parse_tsubo_area(trimmed)
    {
        parsed.findings.approximations.push(approximation(
            trimmed,
            "Tsubo area converted through Japanese customary area.",
        ));
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_area
        && let Some(reading) = parse_square_meter(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_temperature
        && let Some(reading) = parse_temperature(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_compound_quantity
        && let Some(reading) = parse_compound_registered_quantity_ctx(trimmed, ctx)
    {
        if !ctx.accept.compounds {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "compound quantity readings are disabled by acceptance policy",
            );
            return;
        }
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_quantity
        && let Some(reading) = parse_registered_quantity(trimmed, ctx)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_metric_length
        && let Some(reading) = parse_metric_length(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_mass
        && let Some(reading) = parse_mass(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_timezone_clock
        && let Some(reading) = parse_timezone_clock_time(trimmed, ctx)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_clock
        && let Some(reading) = parse_clock_time(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_duration
        && let Some(reading) = parse_duration(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_feet_inches
        && let Some(reading) = parse_feet_inches(trimmed)
    {
        parsed.best = Some(reading);
        return;
    }

    if features.maybe_cups
        && let Some((best, alternatives, ambiguity)) = parse_cups(trimmed, ctx)
    {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        parsed.findings.ambiguities.push(ambiguity);
        return;
    }

    if features.maybe_currency
        && let Some((best, alternatives, ambiguity)) = parse_currency(trimmed, ctx)
    {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        if let Some(ambiguity) = ambiguity {
            parsed.findings.ambiguities.push(ambiguity);
        }
        return;
    }

    if features.maybe_number
        && let Some(ambiguous) = parse_ambiguous_number(trimmed, ctx)
    {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
        return;
    }

    if features.maybe_number
        && let Some(reading) = parse_plain_number_ctx(trimmed, ctx)
    {
        set_plain_number_result(trimmed, ctx, reading, parsed);
        return;
    }

    if features.maybe_quantity
        && let Some((reading, suggestion, unit_text)) =
            parse_typo_corrected_quantity_ctx(trimmed, ctx)
    {
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
        return;
    }

    if features.maybe_timezone_clock
        && let Some(timezone) = unsupported_timezone_suffix(trimmed)
    {
        parsed.findings.skipped.push(skipped_with_span(
            timezone,
            "unsupported timezone conversion requires an explicit adapter policy",
            IssueCode::TimezoneUnsupported,
            span_token_in(trimmed, timezone),
        ));
        return;
    }

    if features.maybe_recurrence
        && let Some(recurrence) = unsupported_recurrence_phrase(trimmed)
    {
        parsed.findings.skipped.push(skipped_with_span(
            recurrence,
            "recurring date/time expressions require a recurrence adapter and are not interpreted by the core parser",
            IssueCode::RecurrenceUnsupported,
            span_token_in(trimmed, recurrence),
        ));
        return;
    }

    if features.maybe_suggestion {
        parsed.suggestions = suggestions_for(trimmed);
    }
    parsed
        .findings
        .skipped
        .push(skipped(trimmed, "no supported reading matched"));
}

fn parse_editor_dimension_into(text: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();

    if is_editor_plain_number_candidate(trimmed) {
        parse_editor_dimension_number_into(trimmed, ctx, parsed);
        return;
    }

    parse_quantity_fast_into(trimmed, ctx, parsed);
    if parsed_is_editor_dimension(parsed, ctx.expected_dimension, ctx.expected_dimension) {
        return;
    }

    parse_editor_dimension_number_into(trimmed, ctx, parsed);
}

fn parse_editor_dimension_number_into(text: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    let expected_dimension = ctx.expected_dimension.unwrap_or(Dimension::Length);
    let mut number = parsed_shell(text, ctx);
    if let Some(ambiguous) = parse_ambiguous_number(text, ctx) {
        number.best = ambiguous.best;
        number.alternatives = ambiguous.alternatives;
        number.findings.ambiguities.push(ambiguous.ambiguity);
    } else if let Some(reading) = parse_plain_number_ctx(text, ctx) {
        set_editor_plain_number_result(text, expected_dimension, reading, &mut number);
    } else {
        number
            .findings
            .skipped
            .push(skipped(text, "no supported number matched"));
    }

    if parsed_is_editor_dimension(&number, Some(expected_dimension), Some(expected_dimension)) {
        *parsed = number;
        return;
    }

    parsed.best = None;
    parsed.alternatives.clear();
    parsed.suggestions.clear();
    parsed.findings = Findings::default();
    parsed
        .findings
        .skipped
        .push(skipped(text, "no supported editor dimension matched"));
}

fn set_editor_plain_number_result(
    text: &str,
    expected_dimension: Dimension,
    reading: Reading,
    parsed: &mut Parsed,
) {
    if expected_dimension == Dimension::Length {
        parsed.alternatives.push(Reading::quantity(
            reading.value.unwrap_or_default(),
            "mm",
            Dimension::Length,
            Provenance::SiMultiple,
            false,
            0.41,
        ));
        parsed.findings.ambiguities.push(ambiguity(
            text,
            "Plain number could be unitless or a context-implied millimeter length.",
            Some(2),
            IssueCode::UnitAssumed,
        ));
    }
    parsed.best = Some(reading);
}

fn is_editor_plain_number_candidate(text: &str) -> bool {
    let mut saw_number = false;
    for ch in text.chars() {
        if is_digit_like(ch) {
            saw_number = true;
            continue;
        }
        if matches!(
            ch,
            '.' | ','
                | '+'
                | '-'
                | '．'
                | '，'
                | '/'
                | '／'
                | '万'
                | '億'
                | '兆'
                | ' '
                | '_'
                | '\''
                | '\u{00A0}'
                | '\u{202F}'
                | '\u{2009}'
        ) {
            continue;
        }
        return false;
    }
    saw_number
}

fn parse_quantity_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    if let Some(result) = parse_qualified_reading(trimmed, ctx) {
        if ctx.strictness == Strictness::Strict {
            parsed.findings.skipped.push(skipped_with_span(
                trimmed,
                "approximate qualifier requires confirmation in strict mode",
                IssueCode::Approximation,
                span(trimmed),
            ));
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    if let Some(result) = parse_fuzzy_reading(trimmed, ctx) {
        if !ctx.accept.fuzzy {
            reject_candidate(
                parsed,
                trimmed,
                result.reading,
                "fuzzy readings are disabled by acceptance policy",
            );
        } else {
            parsed.best = Some(result.reading);
            parsed.findings.approximations = result.approximations;
        }
        return;
    }

    for parser in [
        parse_japanese_length as fn(&str) -> Option<Reading>,
        parse_tatami_area,
        parse_tsubo_area,
        parse_square_meter,
        parse_temperature,
        parse_metric_length,
        parse_mass,
        parse_clock_time,
        parse_duration,
        parse_feet_inches,
    ] {
        if let Some(reading) = parser(trimmed) {
            if reading.approximate == Some(true) {
                parsed
                    .findings
                    .approximations
                    .push(approximation(trimmed, "Approximate quantity conversion."));
            }
            parsed.best = Some(reading);
            return;
        }
    }

    if let Some(reading) = parse_compound_registered_quantity_ctx(trimmed, ctx) {
        if !ctx.accept.compounds {
            reject_candidate(
                parsed,
                trimmed,
                reading,
                "compound quantity readings are disabled by acceptance policy",
            );
        } else {
            parsed.best = Some(reading);
        }
        return;
    }

    if let Some(reading) = parse_registered_quantity(trimmed, ctx) {
        parsed.best = Some(reading);
        return;
    }

    if let Some((best, alternatives, ambiguity)) = parse_cups(trimmed, ctx) {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        parsed.findings.ambiguities.push(ambiguity);
        return;
    }

    if let Some((best, alternatives, ambiguity)) = parse_currency(trimmed, ctx) {
        parsed.best = Some(best);
        parsed.alternatives = alternatives;
        if let Some(ambiguity) = ambiguity {
            parsed.findings.ambiguities.push(ambiguity);
        }
        return;
    }

    if let Some((reading, suggestion, unit_text)) = parse_typo_corrected_quantity_ctx(trimmed, ctx)
    {
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
        return;
    }

    parsed
        .findings
        .skipped
        .push(skipped(trimmed, "no supported quantity matched"));
}

fn parse_number_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    if let Some(ambiguous) = parse_ambiguous_number(trimmed, ctx) {
        parsed.best = ambiguous.best;
        parsed.alternatives = ambiguous.alternatives;
        parsed.findings.ambiguities.push(ambiguous.ambiguity);
    } else if let Some(reading) = parse_plain_number_ctx(trimmed, ctx) {
        set_plain_number_result(trimmed, ctx, reading, parsed);
    } else {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "no supported number matched"));
    }
}

fn parse_date_fast_into(trimmed: &str, ctx: &ParseCtx, parsed: &mut Parsed) {
    if let Some(reading) = parse_relative_date(trimmed, ctx) {
        parsed.best = Some(reading);
    } else {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "no supported date matched"));
    }
}

fn parse_recurrence_fast_into(trimmed: &str, parsed: &mut Parsed) {
    if let Some(reading) = parse_recurrence(trimmed) {
        parsed.best = Some(reading);
    } else if let Some(recurrence) = unsupported_recurrence_phrase(trimmed) {
        parsed.findings.skipped.push(skipped_with_span(
            recurrence,
            "recurring date/time expressions require a recurrence adapter and are not interpreted by the core parser",
            IssueCode::RecurrenceUnsupported,
            span_token_in(trimmed, recurrence),
        ));
    } else {
        parsed
            .findings
            .skipped
            .push(skipped(trimmed, "no supported recurrence matched"));
    }
}

fn set_plain_number_result(text: &str, ctx: &ParseCtx, reading: Reading, parsed: &mut Parsed) {
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
            text,
            "Plain number could be unitless or a context-implied millimeter length.",
            Some(2),
            IssueCode::UnitAssumed,
        ));
    }
    parsed.best = Some(reading);
}

fn reject_candidate(parsed: &mut Parsed, text: &str, reading: Reading, reason: &str) {
    parsed.alternatives.push(reading);
    parsed.findings.skipped.push(skipped_with_span(
        text,
        reason,
        IssueCode::RejectedByPolicy,
        span(text),
    ));
}

#[derive(Clone, Copy, Debug, Default)]
struct InputFeatures {
    maybe_number: bool,
    maybe_quantity: bool,
    maybe_compound_quantity: bool,
    maybe_japanese_length: bool,
    maybe_tatami: bool,
    maybe_tsubo: bool,
    maybe_area: bool,
    maybe_temperature: bool,
    maybe_metric_length: bool,
    maybe_mass: bool,
    maybe_duration: bool,
    maybe_clock: bool,
    maybe_timezone_clock: bool,
    maybe_feet_inches: bool,
    maybe_cups: bool,
    maybe_currency: bool,
    maybe_conversion: bool,
    maybe_range: bool,
    maybe_date: bool,
    maybe_recurrence: bool,
    maybe_suggestion: bool,
    has_slash: bool,
}

impl InputFeatures {
    fn new(text: &str) -> Self {
        let trimmed = text.trim();
        let lower_cow = ascii_lower_cow(trimmed);
        let lower = lower_cow.as_ref();
        let has_ascii_digit = trimmed.as_bytes().iter().any(u8::is_ascii_digit);
        let has_cjk_number = trimmed.chars().any(is_cjk_number_char);
        let has_number_word = lower.split_whitespace().any(is_english_number_word_like);
        let has_number = has_ascii_digit
            || has_cjk_number
            || has_number_word
            || lower
                .split(|ch: char| !ch.is_ascii_alphabetic() && ch != '-')
                .any(|word| small_number_word(word).is_some() || matches!(word, "a" | "an"));
        let has_ascii_alpha =
            trimmed.is_ascii() && trimmed.bytes().any(|byte| byte.is_ascii_alphabetic());
        let maybe_date = lower.starts_with("next ")
            || lower.starts_with("last ")
            || lower.starts_with("this ")
            || lower.starts_with("in ")
            || lower.ends_with(" ago")
            || matches!(lower, "today" | "tomorrow" | "yesterday")
            || [
                "mañana",
                "pasado mañana",
                "demain",
                "amanhã",
                "vendredi prochain",
                "sexta-feira que vem",
                "viernes próximo",
                "viernes proximo",
                "后天",
            ]
            .iter()
            .any(|token| lower == *token || lower.contains(token))
            || trimmed.contains('日')
            || trimmed.contains('週')
            || trimmed.contains('周')
            || (has_ascii_digit && (trimmed.contains('-') || trimmed.contains('/')))
            || matches!(
                trimmed,
                "今日" | "明日" | "昨日" | "一昨日" | "明天" | "昨天" | "前天"
            );
        let maybe_recurrence = lower.starts_with("every ")
            || lower.starts_with("monthly")
            || lower.starts_with("daily")
            || lower.starts_with("freq=")
            || trimmed.starts_with("毎")
            || trimmed.starts_with("每");
        let maybe_clock = lower.contains("am")
            || lower.contains("pm")
            || lower == "noon"
            || lower == "midnight"
            || trimmed.contains(':')
            || trimmed.contains('時');
        let maybe_currency = trimmed.starts_with(['$', '€', '£', '¥', '￥'])
            || [
                "usd", "eur", "gbp", "jpy", "bucks", "dollars", "euros", "pounds", "yen", "円",
                "cent", "cents", "pence",
            ]
            .iter()
            .any(|token| lower.contains(token));
        let maybe_temperature = trimmed.contains('°')
            || trimmed.contains('℃')
            || trimmed.contains('℉')
            || trimmed.contains("摂氏")
            || trimmed.contains("華氏")
            || ["celsius", "fahrenheit", "kelvin"]
                .iter()
                .any(|token| lower.contains(token))
            || (has_number && lower.ends_with(['c', 'f', 'k']));
        let maybe_range = trimmed.contains('±')
            || lower.contains("+/-")
            || lower.contains(" to ")
            || lower.contains("between ")
            || lower.contains("from ")
            || trimmed.contains(['〜', '～'])
            || trimmed.contains("..")
            || trimmed.contains('≤')
            || trimmed.contains('<')
            || trimmed.contains('-')
            || ["less than ", "under ", "below ", "up to ", "at most "]
                .iter()
                .any(|prefix| lower.starts_with(prefix))
            || trimmed.ends_with("以下")
            || trimmed.ends_with("未満")
            || trimmed.ends_with("まで");
        let maybe_duration = lower.starts_with('p')
            || lower.contains("hour")
            || lower.contains("minute")
            || lower.contains("min")
            || lower.contains("day")
            || lower.contains("week")
            || lower.contains("few ")
            || lower.contains("an hour")
            || (has_number
                && [
                    "h", "hr", "hrs", "m", "min", "mins", "s", "sec", "secs", "d",
                ]
                .iter()
                .any(|unit| lower.contains(unit)))
            || trimmed.ends_with('日');
        let maybe_quantity = has_number
            || trimmed.starts_with('約')
            || lower.starts_with("about ")
            || lower.starts_with("around ")
            || lower.starts_with("roughly ")
            || lower.starts_with("approximately ");

        Self {
            maybe_number: has_number,
            maybe_quantity,
            maybe_compound_quantity: maybe_quantity && trimmed.split_whitespace().count() >= 4,
            maybe_japanese_length: maybe_quantity && trimmed.contains(['尺', '寸', '間']),
            maybe_tatami: maybe_quantity && trimmed.contains(['帖', '畳']),
            maybe_tsubo: maybe_quantity && trimmed.contains('坪'),
            maybe_area: maybe_quantity
                && (trimmed.contains('㎡')
                    || trimmed.contains('²')
                    || lower.contains("m2")
                    || lower.contains("m^2")
                    || trimmed.contains("平米")
                    || trimmed.contains("平方米")),
            maybe_temperature,
            maybe_metric_length: maybe_quantity
                && (["cm", "mm", "in", "inch", "inches", "ft", "feet", "m"]
                    .iter()
                    .any(|suffix| lower.ends_with(suffix))
                    || lower.contains('m')),
            maybe_mass: (maybe_quantity
                && [
                    "kg",
                    "kilogram",
                    "kilograms",
                    "lb",
                    "lbs",
                    "pound",
                    "pounds",
                    "ounce",
                    "ounces",
                    "oz",
                    "g",
                ]
                .iter()
                .any(|suffix| lower.ends_with(suffix)))
                || ["公斤", "千克", "キログラム", "キロ"]
                    .iter()
                    .any(|suffix| trimmed.ends_with(suffix)),
            maybe_duration,
            maybe_clock,
            maybe_timezone_clock: maybe_clock && trimmed.split_whitespace().count() >= 2,
            maybe_feet_inches: maybe_quantity
                && (lower.contains("ft") || lower.contains("feet") || trimmed.contains('\'')),
            maybe_cups: maybe_quantity && (lower.ends_with("cup") || lower.ends_with("cups")),
            maybe_currency,
            maybe_conversion: lower.contains(" to "),
            maybe_range,
            maybe_date,
            maybe_recurrence,
            maybe_suggestion: has_ascii_alpha && trimmed.len() <= 160,
            has_slash: trimmed.contains('/'),
        }
    }
}

fn normalize_input_cow(text: &str) -> Cow<'_, str> {
    if !text.chars().any(needs_input_normalization) {
        return Cow::Borrowed(text);
    }
    Cow::Owned(normalize_input(text))
}

fn ascii_lower_cow(text: &str) -> Cow<'_, str> {
    if text.bytes().any(|byte| byte.is_ascii_uppercase()) {
        Cow::Owned(text.to_ascii_lowercase())
    } else {
        Cow::Borrowed(text)
    }
}

fn is_english_number_word_like(word: &str) -> bool {
    word.split('-').filter(|part| !part.is_empty()).all(|part| {
        small_number_word(part).is_some()
            || matches!(part, "hundred" | "thousand" | "a" | "an" | "and")
    })
}

fn normalize_input(text: &str) -> String {
    let mut normalized = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' => {}
            '\u{00A0}' | '\u{202F}' | '\u{2009}' | '\u{2007}' => normalized.push(' '),
            '０'..='９' => {
                let digit = (ch as u32) - ('０' as u32);
                normalized.push(char::from_u32(('0' as u32) + digit).unwrap_or(ch));
            }
            'Ａ'..='Ｚ' => {
                let letter = (ch as u32) - ('Ａ' as u32);
                normalized.push(char::from_u32(('A' as u32) + letter).unwrap_or(ch));
            }
            'ａ'..='ｚ' => {
                let letter = (ch as u32) - ('ａ' as u32);
                normalized.push(char::from_u32(('a' as u32) + letter).unwrap_or(ch));
            }
            '．' | '。' => normalized.push('.'),
            '，' | '、' if looks_numeric_separator(&normalized) => normalized.push(','),
            '＋' => normalized.push('+'),
            '－' | '−' | '–' => normalized.push('-'),
            '／' => normalized.push('/'),
            '＊' | '×' => normalized.push('*'),
            '＾' => normalized.push('^'),
            '％' => normalized.push('%'),
            '　' => normalized.push(' '),
            '㍍' => normalized.push('m'),
            '㌢' => normalized.push_str("cm"),
            '㍉' => normalized.push_str("mm"),
            '㌔' => normalized.push_str("キロ"),
            '㌘' => normalized.push('g'),
            '㎏' => normalized.push_str("kg"),
            '㎎' => normalized.push_str("mg"),
            '㎜' => normalized.push_str("mm"),
            '㎝' => normalized.push_str("cm"),
            '㎞' => normalized.push_str("km"),
            '㏄' => normalized.push_str("cc"),
            _ => normalized.push(ch),
        }
    }
    normalized
}

fn needs_input_normalization(ch: char) -> bool {
    matches!(
        ch,
        '\u{200B}'
            | '\u{200C}'
            | '\u{200D}'
            | '\u{FEFF}'
            | '\u{00A0}'
            | '\u{202F}'
            | '\u{2009}'
            | '\u{2007}'
            | '０'..='９'
            | 'Ａ'..='Ｚ'
            | 'ａ'..='ｚ'
            | '．'
            | '。'
            | '，'
            | '＋'
            | '－'
            | '−'
            | '–'
            | '／'
            | '＊'
            | '×'
            | '＾'
            | '％'
            | '　'
            | '㍍'
            | '㌢'
            | '㍉'
            | '㌔'
            | '㌘'
            | '㎏'
            | '㎎'
            | '㎜'
            | '㎝'
            | '㎞'
            | '㏄'
    )
}

fn looks_numeric_separator(prefix: &str) -> bool {
    prefix
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch.is_ascii_digit())
}

fn is_cjk_number_char(ch: char) -> bool {
    cjk_digit(ch).is_some() || matches!(ch, '十' | '百' | '千' | '万' | '億' | '兆')
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

pub fn complete_readings(text: &str, ctx: Option<ParseCtx>) -> Vec<CompletionReading> {
    let ctx = ctx.unwrap_or_default();
    let parsed = parse(text, Some(ctx.clone()));
    let mut completions = Vec::new();
    if let Some(best) = parsed.best {
        completions.push(CompletionReading {
            text: text.to_owned(),
            score: best.confidence.unwrap_or(0.0),
            reading: best,
            reason: "best".to_owned(),
        });
    }
    for alternative in parsed.alternatives {
        completions.push(CompletionReading {
            text: text.to_owned(),
            score: alternative.confidence.unwrap_or(0.0),
            reading: alternative,
            reason: "alternative".to_owned(),
        });
    }

    if let Some(value) = parse_number_ctx(text, &ctx) {
        push_unit_fanout(text, value, &ctx, &mut completions);
    }

    completions.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.text.cmp(&right.text))
    });
    completions.truncate(24);
    completions
}

fn push_unit_fanout(
    text: &str,
    value: f64,
    ctx: &ParseCtx,
    completions: &mut Vec<CompletionReading>,
) {
    let mut units = units_for_completion_fanout(ctx);
    units.truncate(12);
    for unit in units {
        push_completion_reading_if_new(
            completions,
            CompletionReading {
                text: format!("{} {}", text.trim(), unit.id),
                reading: Reading::quantity(
                    value * unit.factor,
                    unit.canonical_unit,
                    unit.dimension,
                    unit.provenance,
                    unit.approximate,
                    0.45,
                ),
                score: 0.45,
                reason: "unit_fanout".to_owned(),
            },
        );
    }

    for unit in &ctx.custom_units {
        if let Some(expected) = ctx.expected_dimension
            && expected != unit.dimension
        {
            continue;
        }
        let mut reading = Reading::quantity(
            value * unit.factor,
            &unit.canonical_unit,
            unit.dimension,
            Provenance::TradeCustom,
            unit.approximate,
            0.42,
        );
        reading.custom_kind = unit.kind_id.clone();
        push_completion_reading_if_new(
            completions,
            CompletionReading {
                text: format!("{} {}", text.trim(), unit.id),
                reading,
                score: 0.42,
                reason: "custom_unit_fanout".to_owned(),
            },
        );
    }
}

fn push_completion_reading_if_new(
    completions: &mut Vec<CompletionReading>,
    completion: CompletionReading,
) {
    if completions
        .iter()
        .any(|existing| existing.text == completion.text && existing.reading == completion.reading)
    {
        return;
    }
    completions.push(completion);
}

fn units_for_completion_fanout(ctx: &ParseCtx) -> Vec<&'static UnitDef> {
    let dimension = ctx.expected_dimension;
    let mut units = Vec::new();
    for unit in UNIT_DEFS {
        if dimension.is_none_or(|expected| expected == unit.dimension) {
            units.push(unit);
        }
    }
    units
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalizeRequest {
    pub field: String,
    pub text: String,
    pub ctx: Option<ParseCtx>,
}

impl CanonicalizeRequest {
    pub fn new(field: &str, text: &str, ctx: Option<ParseCtx>) -> Self {
        Self {
            field: field.to_owned(),
            text: text.to_owned(),
            ctx,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalizedValue {
    pub field: String,
    pub input: String,
    pub ok: bool,
    pub canonical: Option<Reading>,
    pub parsed: Parsed,
    pub message: Option<String>,
}

pub fn canonicalize_values(requests: &[CanonicalizeRequest]) -> Vec<CanonicalizedValue> {
    requests
        .iter()
        .map(|request| {
            let parsed = parse(&request.text, request.ctx.clone());
            let ok = adapter_accepts(&parsed, request.ctx.as_ref());
            let canonical = ok.then(|| parsed.best.clone()).flatten();
            let message = (!ok).then(|| adapter_message(&request.field, &parsed));
            CanonicalizedValue {
                field: request.field.clone(),
                input: request.text.clone(),
                ok,
                canonical,
                parsed,
                message,
            }
        })
        .collect()
}

pub fn repair_tool_call_message(field: &str, text: &str, ctx: Option<ParseCtx>) -> Option<String> {
    let request = CanonicalizeRequest::new(field, text, ctx);
    canonicalize_values(&[request])
        .into_iter()
        .next()
        .and_then(|value| value.message)
}

fn adapter_accepts(parsed: &Parsed, ctx: Option<&ParseCtx>) -> bool {
    if parsed.best.is_none() || !parsed.findings.skipped.is_empty() {
        return false;
    }
    let strictness = ctx.map_or(Strictness::Forgiving, |ctx| ctx.strictness);
    if strictness != Strictness::Forgiving
        && (!parsed.findings.ambiguities.is_empty() || !parsed.findings.approximations.is_empty())
    {
        return false;
    }
    true
}

fn adapter_message(field: &str, parsed: &Parsed) -> String {
    let (code, reason, ref_text) = parsed
        .findings
        .skipped
        .first()
        .map(|issue| (issue.code, issue.reason.as_str(), issue.ref_text.as_str()))
        .or_else(|| {
            parsed
                .findings
                .ambiguities
                .first()
                .map(|issue| (issue.code, issue.reason.as_str(), issue.ref_text.as_str()))
        })
        .or_else(|| {
            parsed
                .findings
                .approximations
                .first()
                .map(|issue| (issue.code, issue.reason.as_str(), issue.ref_text.as_str()))
        })
        .unwrap_or((
            IssueCode::NoValue,
            "no supported reading matched",
            parsed.input.as_str(),
        ));
    let suggestion = parsed
        .suggestions
        .first()
        .map(|suggestion| format!(" Did you mean `{}`?", suggestion.to))
        .unwrap_or_default();
    format!(
        "[{}] {field}: {reason} at `{ref_text}`.{suggestion}",
        code.as_str()
    )
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
        (_, Kind::Recurrence, _, _) => value
            .recurrence
            .clone()
            .unwrap_or_else(|| "unknown recurrence".to_owned()),
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

pub fn describe_reading(reading: &Reading) -> ResourceView {
    let object = match reading.kind {
        Kind::Quantity => "unravel.quantity",
        Kind::Date => "unravel.date",
        Kind::Range => "unravel.range",
        Kind::Number => "unravel.number",
        Kind::Recurrence => "unravel.recurrence",
    }
    .to_owned();
    let mut fields = Vec::new();
    push_resource_field(&mut fields, "kind", kind_str(reading.kind));
    if let Some(custom_kind) = &reading.custom_kind {
        push_resource_field(&mut fields, "custom_kind", custom_kind);
    }
    if let Some(value) = reading.value {
        push_resource_field(&mut fields, "value", &format_number(value));
    }
    if let Some(unit) = &reading.unit {
        push_resource_field(&mut fields, "unit", unit);
    }
    if let Some(dimension) = reading.dimension {
        push_resource_field(&mut fields, "dimension", dimension.as_str());
    }
    if let Some(date) = &reading.date {
        push_resource_field(&mut fields, "date", date);
    }
    if let Some(recurrence) = &reading.recurrence {
        push_resource_field(&mut fields, "recurrence", recurrence);
    }
    if let Some(timezone) = &reading.timezone {
        push_resource_field(&mut fields, "timezone", timezone);
    }
    if let Some(provenance) = reading.provenance {
        push_resource_field(&mut fields, "provenance", provenance.as_str());
    }
    if let Some(approximate) = reading.approximate {
        push_resource_field(
            &mut fields,
            "approximate",
            if approximate { "true" } else { "false" },
        );
    }
    if let Some(confidence) = reading.confidence {
        push_resource_field(&mut fields, "confidence", &format_number(confidence));
    }
    let summary = humanize(reading, None);
    ResourceView {
        object,
        summary,
        fields,
    }
}

pub fn describe_parsed(parsed: &Parsed) -> ResourceView {
    let mut fields = Vec::new();
    push_resource_field(&mut fields, "input", &parsed.input);
    if let Some(locale) = &parsed.locale {
        push_resource_field(&mut fields, "locale", locale.as_str());
    }
    push_resource_field(
        &mut fields,
        "ok",
        if parsed.best.is_some() && parsed.findings.skipped.is_empty() {
            "true"
        } else {
            "false"
        },
    );
    push_resource_field(
        &mut fields,
        "skipped",
        &parsed.findings.skipped.len().to_string(),
    );
    push_resource_field(
        &mut fields,
        "ambiguities",
        &parsed.findings.ambiguities.len().to_string(),
    );
    push_resource_field(
        &mut fields,
        "approximations",
        &parsed.findings.approximations.len().to_string(),
    );
    push_resource_field(
        &mut fields,
        "alternatives",
        &parsed.alternatives.len().to_string(),
    );
    let summary = parsed
        .best
        .as_ref()
        .map(|reading| humanize(reading, None))
        .unwrap_or_else(|| "no supported reading".to_owned());
    ResourceView {
        object: "unravel.parsed".to_owned(),
        summary,
        fields,
    }
}

fn push_resource_field(fields: &mut Vec<ResourceField>, name: &str, value: &str) {
    fields.push(ResourceField {
        name: name.to_owned(),
        value: value.to_owned(),
    });
}

fn parse_japanese_length(text: &str) -> Option<Reading> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    for (suffix, factor) in [("間半", KEN_M), ("尺半", SHAKU_M)] {
        if let Some(number_text) = compact.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                (value + 0.5) * factor,
                "m",
                Dimension::Length,
                Provenance::JapaneseStatute,
                true,
                0.94,
            ));
        }
    }

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
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    for suffix in ["帖半", "畳半"] {
        if let Some(number_text) = compact.strip_suffix(suffix) {
            let value = parse_number(number_text.trim())?;
            return Some(Reading::quantity(
                (value + 0.5) * TATAMI_M2,
                "m2",
                Dimension::Area,
                Provenance::TradeCustom,
                true,
                0.92,
            ));
        }
    }

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
    let value = parse_number_ctx(number_text, ctx)?;
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
    let mut reading = Reading::quantity(
        value * unit.factor,
        &unit.canonical_unit,
        unit.dimension,
        Provenance::TradeCustom,
        unit.approximate,
        0.93,
    );
    reading.custom_kind = unit.kind_id.clone();
    Some(reading)
}

fn parse_compound_registered_quantity_ctx(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    parse_compound_registered_quantity_with_format(text, ctx.number_format)
}

fn parse_compound_registered_quantity_with_format(
    text: &str,
    number_format: NumberFormat,
) -> Option<Reading> {
    let parts: Vec<&str> = text.split_whitespace().collect();
    if parts.len() < 4 || !parts.len().is_multiple_of(2) {
        return None;
    }

    let mut total = 0.0;
    let mut dimension = None;
    let mut canonical_unit = None;
    let mut provenance = Provenance::SiMultiple;
    let mut approximate = false;

    for pair in parts.chunks_exact(2) {
        let value = parse_number_with_format(pair[0], number_format)?;
        let unit = unit_by_alias(pair[1])?;
        if let Some(current_dimension) = dimension {
            if current_dimension != unit.dimension || canonical_unit != Some(unit.canonical_unit) {
                return None;
            }
        } else {
            dimension = Some(unit.dimension);
            canonical_unit = Some(unit.canonical_unit);
            provenance = unit.provenance;
        }
        total += value * unit.factor;
        approximate |= unit.approximate;
    }

    Some(Reading::quantity(
        total,
        canonical_unit?,
        dimension?,
        provenance,
        approximate,
        0.94,
    ))
}

fn parse_typo_corrected_quantity_ctx(
    text: &str,
    ctx: &ParseCtx,
) -> Option<(Reading, Suggestion, String)> {
    let (number_text, unit_text) = split_number_unit(text)?;
    let value = parse_number_ctx(number_text, ctx)?;
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
        if is_number_prefix_char(ch) {
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

fn is_number_prefix_char(ch: char) -> bool {
    ch.is_ascii_digit() || is_cjk_number_char(ch)
}

fn unit_by_alias(alias: &str) -> Option<&'static UnitDef> {
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
enum AliasMatchMode {
    Exact,
    AsciiCase,
}

fn fast_unit_by_alias(alias: &str, mode: AliasMatchMode) -> Option<&'static UnitDef> {
    FAST_UNIT_ALIASES
        .iter()
        .find_map(|(candidate, unit_id)| alias_matches(candidate, alias, mode).then_some(*unit_id))
        .and_then(unit_by_id)
}

fn unit_by_id(id: &str) -> Option<&'static UnitDef> {
    UNIT_DEFS.iter().find(|unit| unit.id == id)
}

fn fallback_unit_by_alias(alias: &str, mode: AliasMatchMode) -> Option<&'static UnitDef> {
    UNIT_DEFS.iter().find(|unit| {
        unit_lookup_aliases(unit).any(|candidate| alias_matches(candidate.trim(), alias, mode))
    })
}

fn alias_matches(candidate: &str, alias: &str, mode: AliasMatchMode) -> bool {
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
    normalize_input(alias).trim().to_ascii_lowercase()
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

fn parse_timezone_clock_time(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let trimmed = text.trim();
    let zone = trimmed.split_whitespace().last()?;
    let head = trimmed.strip_suffix(zone)?.trim_end();
    let seconds = parse_clock_seconds(head)?;
    let offset = timezone_offset_seconds(zone)
        .or_else(|| iana_timezone_offset_seconds(zone, seconds, ctx.reference_date))?;
    let utc_seconds = modulo_day(seconds - f64::from(offset));
    let mut reading = Reading::quantity(
        utc_seconds,
        "s",
        Dimension::Time,
        Provenance::TradeCustom,
        false,
        0.9,
    );
    reading.timezone = Some("UTC".to_owned());
    Some(reading)
}

fn unsupported_timezone_suffix(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let timezone = trimmed.split_whitespace().last()?;
    let head = trimmed.strip_suffix(timezone)?.trim_end();
    parse_clock_time(head)?;
    is_timezone_token(timezone).then_some(timezone)
}

fn timezone_offset_seconds(text: &str) -> Option<i32> {
    match text {
        "UTC" | "GMT" => Some(0),
        "EST" => Some(-5 * 3600),
        "EDT" => Some(-4 * 3600),
        "CST" => Some(-6 * 3600),
        "CDT" => Some(-5 * 3600),
        "MST" => Some(-7 * 3600),
        "MDT" => Some(-6 * 3600),
        "PST" => Some(-8 * 3600),
        "PDT" => Some(-7 * 3600),
        "JST" => Some(9 * 3600),
        "KST" => Some(9 * 3600),
        "CET" => Some(3600),
        "CEST" => Some(2 * 3600),
        "BST" => Some(3600),
        "AEST" => Some(10 * 3600),
        "AEDT" => Some(11 * 3600),
        _ => parse_utc_offset_seconds(text),
    }
}

fn parse_utc_offset_seconds(text: &str) -> Option<i32> {
    let offset = text
        .strip_prefix("UTC")
        .or_else(|| text.strip_prefix("GMT"))?;
    let (sign, signless) = if let Some(signless) = offset.strip_prefix('+') {
        (1, signless)
    } else if let Some(signless) = offset.strip_prefix('-') {
        (-1, signless)
    } else {
        return None;
    };
    let (hours, minutes) = signless.split_once(':').unwrap_or((signless, "00"));
    if hours.is_empty()
        || hours.len() > 2
        || minutes.len() != 2
        || !hours.chars().all(|ch| ch.is_ascii_digit())
        || !minutes.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    let hours = hours.parse::<i32>().ok()?;
    let minutes = minutes.parse::<i32>().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

#[cfg(feature = "timezones-jiff")]
fn iana_timezone_offset_seconds(
    zone: &str,
    seconds: f64,
    reference_date: Option<Date>,
) -> Option<i32> {
    if !is_iana_timezone_name(zone) {
        return None;
    }
    let date = reference_date?;
    let year = i16::try_from(date.year).ok()?;
    let clock_seconds = seconds.round() as i64;
    let hour = i8::try_from(clock_seconds / 3600).ok()?;
    let minute = i8::try_from((clock_seconds % 3600) / 60).ok()?;
    let second = i8::try_from(clock_seconds % 60).ok()?;
    let datetime =
        jiff::civil::date(year, date.month as i8, date.day as i8).at(hour, minute, second, 0);
    let (canonical_name, data) = jiff_tzdb::get(zone)?;
    let timezone = jiff::tz::TimeZone::tzif(canonical_name, data).ok()?;
    datetime
        .to_zoned(timezone)
        .ok()
        .map(|zoned| zoned.offset().seconds())
}

#[cfg(not(feature = "timezones-jiff"))]
fn iana_timezone_offset_seconds(
    _zone: &str,
    _seconds: f64,
    _reference_date: Option<Date>,
) -> Option<i32> {
    None
}

fn modulo_day(seconds: f64) -> f64 {
    seconds.rem_euclid(86_400.0)
}

fn is_timezone_token(text: &str) -> bool {
    if timezone_offset_seconds(text).is_some() {
        return true;
    }
    if is_iana_timezone_name(text) {
        return true;
    }
    if matches!(
        text,
        "UTC"
            | "GMT"
            | "EST"
            | "EDT"
            | "CST"
            | "CDT"
            | "MST"
            | "MDT"
            | "PST"
            | "PDT"
            | "JST"
            | "KST"
            | "CET"
            | "CEST"
            | "BST"
            | "AEST"
            | "AEDT"
    ) {
        return true;
    }

    let Some(offset) = text
        .strip_prefix("UTC")
        .or_else(|| text.strip_prefix("GMT"))
    else {
        return false;
    };
    let Some(signless) = offset
        .strip_prefix('+')
        .or_else(|| offset.strip_prefix('-'))
    else {
        return false;
    };
    let (hours, minutes) = signless.split_once(':').unwrap_or((signless, "00"));
    hours.len() <= 2
        && !hours.is_empty()
        && minutes.len() == 2
        && hours.chars().all(|ch| ch.is_ascii_digit())
        && minutes.chars().all(|ch| ch.is_ascii_digit())
}

fn is_iana_timezone_name(text: &str) -> bool {
    text.contains('/')
        && text
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '+'))
}

fn unsupported_recurrence_phrase(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    let lowered = trimmed.to_ascii_lowercase();
    if let Some(rest) = lowered.strip_prefix("every ")
        && !rest.trim().is_empty()
    {
        return trimmed.get(.."every".len());
    }
    if trimmed.starts_with("毎週") || trimmed.starts_with("毎日") || trimmed.starts_with("毎月")
    {
        return trimmed.get(.."毎".len());
    }
    None
}

fn parse_recurrence(text: &str) -> Option<Reading> {
    let trimmed = text.trim();
    if is_supported_rrule(trimmed) {
        return Some(Reading::recurrence(trimmed, 0.99));
    }

    let lowered = trimmed.to_ascii_lowercase();
    if let Some(bysetpos) = parse_english_business_day_recurrence(&lowered) {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYSETPOS={bysetpos};BYDAY=MO,TU,WE,TH,FR"),
            0.8,
        ));
    }
    if let Some(bysetpos) = parse_japanese_business_day_recurrence(trimmed) {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYSETPOS={bysetpos};BYDAY=MO,TU,WE,TH,FR"),
            0.8,
        ));
    }
    if let Some(day_text) = lowered.strip_prefix("monthly on the ") {
        if let Some(byday) = parse_english_ordinal_weekday(day_text.trim()) {
            return Some(Reading::recurrence(
                &format!("FREQ=MONTHLY;BYDAY={byday}"),
                0.84,
            ));
        }
        let day = parse_ordinal_month_day(day_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYMONTHDAY={day}"),
            0.88,
        ));
    }
    if let Some(day_text) = lowered.strip_prefix("every month on the ") {
        if let Some(byday) = parse_english_ordinal_weekday(day_text.trim()) {
            return Some(Reading::recurrence(
                &format!("FREQ=MONTHLY;BYDAY={byday}"),
                0.84,
            ));
        }
        let day = parse_ordinal_month_day(day_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYMONTHDAY={day}"),
            0.88,
        ));
    }
    if let Some(byday) = trimmed
        .strip_prefix("毎月")
        .and_then(parse_japanese_ordinal_weekday)
    {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYDAY={byday}"),
            0.84,
        ));
    }
    if let Some(day_text) = trimmed
        .strip_prefix("毎月")
        .and_then(|tail| tail.strip_suffix('日'))
    {
        let day = parse_whole_i64(day_text.trim())?;
        if (1..=31).contains(&day) {
            return Some(Reading::recurrence(
                &format!("FREQ=MONTHLY;BYMONTHDAY={day}"),
                0.88,
            ));
        }
    }

    let rrule = if matches!(lowered.as_str(), "every day" | "daily") || trimmed == "毎日" {
        "FREQ=DAILY"
    } else if matches!(lowered.as_str(), "every month" | "monthly") || trimmed == "毎月" {
        "FREQ=MONTHLY"
    } else if let Some(weekday_text) = lowered.strip_prefix("every ") {
        return parse_english_every_recurrence(weekday_text.trim());
    } else if let Some(weekday_text) = trimmed.strip_prefix("毎週") {
        let day = recurrence_weekday(weekday_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;BYDAY={day}"),
            0.9,
        ));
    } else if let Some(weekday_text) = trimmed.strip_prefix("每周") {
        let day = recurrence_weekday(weekday_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;BYDAY={day}"),
            0.9,
        ));
    } else {
        return None;
    };
    Some(Reading::recurrence(rrule, 0.92))
}

fn is_supported_rrule(text: &str) -> bool {
    matches!(text, "FREQ=DAILY" | "FREQ=MONTHLY")
        || text
            .strip_prefix("FREQ=DAILY;INTERVAL=")
            .is_some_and(valid_positive_i64)
        || text
            .strip_prefix("FREQ=WEEKLY;INTERVAL=")
            .is_some_and(valid_positive_i64)
        || text
            .strip_prefix("FREQ=WEEKLY;INTERVAL=")
            .is_some_and(valid_weekly_interval_byday)
        || text
            .strip_prefix("FREQ=MONTHLY;INTERVAL=")
            .is_some_and(valid_positive_i64)
        || text
            .strip_prefix("FREQ=MONTHLY;BYMONTHDAY=")
            .is_some_and(valid_month_day)
        || text
            .strip_prefix("FREQ=MONTHLY;BYDAY=")
            .is_some_and(valid_monthly_byday)
        || text
            .strip_prefix("FREQ=MONTHLY;BYSETPOS=")
            .is_some_and(valid_monthly_business_day)
        || text
            .strip_prefix("FREQ=WEEKLY;BYDAY=")
            .is_some_and(valid_weekly_byday)
}

fn parse_english_every_recurrence(text: &str) -> Option<Reading> {
    if let Some((base, count)) = split_recurrence_count(text) {
        let day = recurrence_weekday(base.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;BYDAY={day};COUNT={count}"),
            0.86,
        ));
    }

    if let Some(weekday_text) = text.strip_prefix("other ") {
        let day = recurrence_weekday(weekday_text.trim())?;
        return Some(Reading::recurrence(
            &format!("FREQ=WEEKLY;INTERVAL=2;BYDAY={day}"),
            0.84,
        ));
    }
    if let Some(day_text) = text.strip_prefix("month on the ")
        && let Some(byday) = parse_english_ordinal_weekday(day_text.trim())
    {
        return Some(Reading::recurrence(
            &format!("FREQ=MONTHLY;BYDAY={byday}"),
            0.84,
        ));
    }

    let mut parts = text.split_whitespace();
    let first = parts.next()?;
    let second = parts.next();
    if parts.next().is_none()
        && let Some(unit) = second
    {
        let interval = parse_whole_i64(first)?;
        if interval <= 0 {
            return None;
        }
        let freq = match unit {
            "day" | "days" => "DAILY",
            "week" | "weeks" => "WEEKLY",
            "month" | "months" => "MONTHLY",
            _ => return None,
        };
        return Some(Reading::recurrence(
            &format!("FREQ={freq};INTERVAL={interval}"),
            0.88,
        ));
    }

    let day = recurrence_weekday(text.trim())?;
    Some(Reading::recurrence(
        &format!("FREQ=WEEKLY;BYDAY={day}"),
        0.9,
    ))
}

fn split_recurrence_count(text: &str) -> Option<(&str, i64)> {
    let (base, count_text) = text.rsplit_once(" for ")?;
    let count = count_text
        .strip_suffix(" times")
        .or_else(|| count_text.strip_suffix(" occurrences"))
        .or_else(|| count_text.strip_suffix(" occurrence"))?;
    let count = parse_whole_i64(count.trim())?;
    (count > 0).then_some((base, count))
}

fn parse_ordinal_month_day(text: &str) -> Option<i64> {
    let lower = text.trim().to_ascii_lowercase();
    let number_text = lower
        .strip_suffix("st")
        .or_else(|| lower.strip_suffix("nd"))
        .or_else(|| lower.strip_suffix("rd"))
        .or_else(|| lower.strip_suffix("th"))
        .unwrap_or(lower.as_str());
    let day = parse_whole_i64(number_text.trim())?;
    (1..=31).contains(&day).then_some(day)
}

fn parse_english_ordinal_weekday(text: &str) -> Option<String> {
    let text = text.strip_suffix(" of the month").unwrap_or(text).trim();
    let (ordinal_text, weekday_text) = text.split_once(' ')?;
    let ordinal = parse_recurrence_ordinal(ordinal_text)?;
    let weekday = recurrence_weekday(weekday_text.trim())?;
    Some(format!("{ordinal}{weekday}"))
}

fn parse_japanese_ordinal_weekday(text: &str) -> Option<String> {
    let text = text.strip_prefix('第')?;
    let digit_end = text
        .char_indices()
        .find(|(_, ch)| !ch.is_ascii_digit())
        .map(|(idx, _)| idx)?;
    let ordinal = parse_whole_i64(&text[..digit_end])?;
    if !(1..=5).contains(&ordinal) {
        return None;
    }
    let weekday = recurrence_weekday(text[digit_end..].trim())?;
    Some(format!("{ordinal}{weekday}"))
}

fn parse_recurrence_ordinal(text: &str) -> Option<String> {
    let ordinal = match text {
        "first" => 1,
        "second" => 2,
        "third" => 3,
        "fourth" => 4,
        "fifth" => 5,
        "last" => return Some("-1".to_owned()),
        _ => parse_ordinal_month_day(text)?,
    };
    (1..=5).contains(&ordinal).then(|| ordinal.to_string())
}

fn parse_english_business_day_recurrence(text: &str) -> Option<String> {
    let text = text
        .strip_prefix("every ")
        .or_else(|| text.strip_prefix("monthly on the "))
        .or_else(|| text.strip_prefix("every month on the "))?;
    let text = text.strip_suffix(" of the month").unwrap_or(text).trim();
    let ordinal_text = text
        .strip_suffix(" business day")
        .or_else(|| text.strip_suffix(" business days"))?
        .trim();
    parse_recurrence_ordinal(ordinal_text)
}

fn parse_japanese_business_day_recurrence(text: &str) -> Option<String> {
    let text = text.strip_prefix("毎月第")?;
    let ordinal_text = text
        .strip_suffix("営業日")
        .or_else(|| text.strip_suffix("業務日"))?;
    let ordinal = parse_whole_i64(ordinal_text)?;
    (1..=5).contains(&ordinal).then(|| ordinal.to_string())
}

fn valid_positive_i64(text: &str) -> bool {
    parse_whole_i64(text).is_some_and(|value| value > 0)
}

fn valid_month_day(text: &str) -> bool {
    parse_whole_i64(text).is_some_and(|value| (1..=31).contains(&value))
}

fn valid_weekly_byday(text: &str) -> bool {
    if let Some((day, count_text)) = text.split_once(";COUNT=") {
        return matches!(day, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU")
            && valid_positive_i64(count_text);
    }
    matches!(text, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU")
}

fn valid_weekly_interval_byday(text: &str) -> bool {
    let Some((interval_text, byday)) = text.split_once(";BYDAY=") else {
        return false;
    };
    valid_positive_i64(interval_text) && valid_weekly_byday(byday)
}

fn valid_monthly_byday(text: &str) -> bool {
    if text.len() < 3 {
        return false;
    }
    let (ordinal_text, weekday_text) = text.split_at(text.len() - 2);
    matches!(weekday_text, "MO" | "TU" | "WE" | "TH" | "FR" | "SA" | "SU")
        && matches!(ordinal_text, "-1" | "1" | "2" | "3" | "4" | "5")
}

fn valid_monthly_business_day(text: &str) -> bool {
    let Some((bysetpos, byday)) = text.split_once(";BYDAY=") else {
        return false;
    };
    matches!(bysetpos, "-1" | "1" | "2" | "3" | "4" | "5") && byday == "MO,TU,WE,TH,FR"
}

fn recurrence_weekday(text: &str) -> Option<&'static str> {
    match text {
        "monday" | "mon" | "月曜日" | "月曜" | "月" | "周一" | "星期一" | "一" => {
            Some("MO")
        }
        "tuesday" | "tue" | "tues" | "火曜日" | "火曜" | "火" | "周二" | "星期二" | "二" => {
            Some("TU")
        }
        "wednesday" | "wed" | "水曜日" | "水曜" | "水" | "周三" | "星期三" | "三" => {
            Some("WE")
        }
        "thursday" | "thu" | "thur" | "thurs" | "木曜日" | "木曜" | "木" | "周四" | "星期四"
        | "四" => Some("TH"),
        "friday" | "fri" | "金曜日" | "金曜" | "金" | "周五" | "星期五" | "五" => {
            Some("FR")
        }
        "saturday" | "sat" | "土曜日" | "土曜" | "土" | "周六" | "星期六" | "六" => {
            Some("SA")
        }
        "sunday" | "sun" | "日曜日" | "日曜" | "日" | "周日" | "星期日" | "星期天" | "天" => {
            Some("SU")
        }
        _ => None,
    }
}

fn parse_clock_seconds(text: &str) -> Option<f64> {
    if let Some(seconds) = parse_japanese_clock_seconds(text) {
        return Some(seconds);
    }

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

fn parse_japanese_clock_seconds(text: &str) -> Option<f64> {
    let compact: String = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    let (body, meridiem) = if let Some(body) = compact.strip_prefix("午前") {
        (body, Some("am"))
    } else if let Some(body) = compact.strip_prefix("午後") {
        (body, Some("pm"))
    } else {
        (compact.as_str(), None)
    };

    let (hour_text, minute_tail) = body.split_once('時')?;
    if hour_text.is_empty() {
        return None;
    }
    let mut hour = hour_text.parse::<u8>().ok()?;
    let minute = if minute_tail.is_empty() {
        0
    } else if minute_tail == "半" {
        30
    } else {
        minute_tail.strip_suffix('分')?.parse::<u8>().ok()?
    };
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
            if hour > 23 {
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
    let value = parse_number_ctx(number_text, ctx)?;

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

fn parse_currency(
    text: &str,
    ctx: &ParseCtx,
) -> Option<(Reading, Vec<Reading>, Option<Ambiguity>)> {
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
            let value = parse_number_ctx(number_text.trim(), ctx)?;
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
            let value = parse_number_ctx(number_text.trim(), ctx)?;
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
            let value = parse_number_ctx(number_text.trim(), ctx)?;
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
        let value = parse_number_ctx(number_text.trim(), ctx)?;
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
        let value = parse_number_ctx(number_text.trim(), ctx)?;
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

fn parse_plain_number_ctx(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    parse_number_ctx(text, ctx).map(|value| Reading::number(value, 0.99))
}

fn parse_qualified_reading(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    let (qualifier, rest) = strip_approximate_qualifier(text)?;
    let mut reading = parse_endpoint(rest, ctx)?;
    mark_approximate(&mut reading);
    Some(ParsedReading {
        reading,
        approximations: vec![approximation_with_span(
            qualifier,
            "Approximate qualifier was preserved as an approximation finding.",
            span_in(text, qualifier),
        )],
    })
}

fn strip_approximate_qualifier(text: &str) -> Option<(&str, &str)> {
    let trimmed = text.trim();
    for prefix in [
        "approximately ",
        "approx. ",
        "approx ",
        "around ",
        "roughly ",
        "about ",
    ] {
        if let Some(rest) = strip_prefix_ascii_case(trimmed, prefix)
            && !rest.trim().is_empty()
        {
            return Some((trimmed.get(..prefix.len())?.trim(), rest.trim()));
        }
    }
    if let Some(rest) = trimmed.strip_prefix('約')
        && !rest.trim().is_empty()
    {
        return Some(("約", rest.trim()));
    }
    for suffix in [" (approx.)", " approx.", " approximately"] {
        if let Some(rest) = strip_suffix_ascii_case(trimmed, suffix)
            && !rest.trim().is_empty()
        {
            return Some((
                trimmed.get(trimmed.len() - suffix.len()..)?.trim(),
                rest.trim(),
            ));
        }
    }
    None
}

fn parse_fuzzy_reading(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    if let Some(rest) = strip_prefix_ascii_case(text.trim(), "a few ")
        && !rest.trim().is_empty()
    {
        let from = parse_endpoint(&format!("2 {}", rest.trim()), ctx)?;
        let to = parse_endpoint(&format!("4 {}", rest.trim()), ctx)?;
        if from.kind == to.kind && from.dimension == to.dimension {
            let mut reading = Reading::range(from, to, 0.72);
            mark_approximate(&mut reading);
            return Some(ParsedReading {
                reading,
                approximations: vec![approximation_with_span(
                    "a few",
                    "Fuzzy small-count phrase normalized to a 2 to 4 range.",
                    span_in(text, "a few"),
                )],
            });
        }
    }

    parse_custom_fuzzy_profile(text, ctx).or_else(|| parse_fuzzy_temperature(text, ctx))
}

fn parse_custom_fuzzy_profile(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    let normalized = normalize_alias(text);
    for profile in &ctx.fuzzy_profiles {
        if let Some(expected_dimension) = ctx.expected_dimension
            && expected_dimension != profile.dimension
        {
            continue;
        }
        let Some(target_unit) = unit_by_alias(&profile.unit) else {
            continue;
        };
        if target_unit.dimension != profile.dimension {
            continue;
        }
        for term in &profile.terms {
            if normalize_alias(&term.term) != normalized {
                continue;
            }
            let mut reading = Reading::range(
                Reading::quantity(
                    term.low * target_unit.factor,
                    target_unit.canonical_unit,
                    profile.dimension,
                    target_unit.provenance,
                    target_unit.approximate,
                    0.72,
                ),
                Reading::quantity(
                    term.high * target_unit.factor,
                    target_unit.canonical_unit,
                    profile.dimension,
                    target_unit.provenance,
                    target_unit.approximate,
                    0.72,
                ),
                0.72,
            );
            mark_approximate(&mut reading);
            return Some(ParsedReading {
                reading,
                approximations: vec![approximation_with_span(
                    text,
                    "Custom fuzzy vocabulary normalized to a configured range.",
                    span(text),
                )],
            });
        }
    }
    None
}

fn parse_fuzzy_temperature(text: &str, ctx: &ParseCtx) -> Option<ParsedReading> {
    if ctx.expected_dimension != Some(Dimension::Temperature) {
        return None;
    }

    let normalized = text.trim().to_ascii_lowercase();
    let (label, low, high) = if text.contains("暑い") {
        ("暑い", 27.0, 35.0)
    } else if text.contains("暖か") {
        ("暖か", 20.0, 27.0)
    } else if text.contains("寒い") {
        ("寒い", 0.0, 10.0)
    } else if normalized.contains("hot") {
        ("hot", 27.0, 35.0)
    } else if normalized.contains("warm") {
        ("warm", 20.0, 27.0)
    } else if normalized.contains("cold") {
        ("cold", 0.0, 10.0)
    } else {
        return None;
    };

    let mut reading = Reading::range(
        temperature_celsius(low, 0.68),
        temperature_celsius(high, 0.68),
        0.68,
    );
    mark_approximate(&mut reading);
    Some(ParsedReading {
        reading,
        approximations: vec![approximation_with_span(
            label,
            "Fuzzy temperature phrase normalized to a broad Celsius range.",
            span_in(text, label),
        )],
    })
}

fn parse_plus_minus_range(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let (left, right) = text
        .split_once('±')
        .or_else(|| split_once_ascii_case(text, "+/-"))?;
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() || right.is_empty() {
        return None;
    }

    let left_suffix = unit_suffix(left, ctx);
    let right_suffix = unit_suffix(right, ctx);
    let left_owned;
    let right_owned;
    let center_text = if left_suffix.is_none() {
        if let Some(suffix) = right_suffix {
            left_owned = format!("{left}{suffix}");
            left_owned.as_str()
        } else {
            left
        }
    } else {
        left
    };
    let delta_text = if right_suffix.is_none() {
        if let Some(suffix) = left_suffix {
            right_owned = format!("{right}{suffix}");
            right_owned.as_str()
        } else {
            right
        }
    } else {
        right
    };

    let center = parse_endpoint(center_text, ctx)?;
    let delta = parse_endpoint(delta_text, ctx)?;
    let (center_value, delta_value) = (center.value?, delta.value?);
    if center.kind != Kind::Quantity
        || delta.kind != Kind::Quantity
        || center.dimension != delta.dimension
        || center.unit != delta.unit
    {
        return None;
    }

    let unit = center.unit.as_deref()?;
    let dimension = center.dimension?;
    let provenance = center.provenance.unwrap_or(Provenance::TradeCustom);
    let approximate = center.approximate.unwrap_or(false) || delta.approximate.unwrap_or(false);
    Some(Reading::range(
        Reading::quantity(
            center_value - delta_value,
            unit,
            dimension,
            provenance,
            approximate,
            0.93,
        ),
        Reading::quantity(
            center_value + delta_value,
            unit,
            dimension,
            provenance,
            approximate,
            0.93,
        ),
        0.93,
    ))
}

fn parse_upper_bound_range(text: &str, ctx: &ParseCtx) -> Option<Reading> {
    let trimmed = text.trim();
    let rest = ["less than ", "under ", "below ", "up to ", "at most "]
        .into_iter()
        .find_map(|prefix| strip_prefix_ascii_case(trimmed, prefix))
        .or_else(|| trimmed.strip_prefix("最大"))
        .or_else(|| trimmed.strip_prefix("上限"))
        .or_else(|| trimmed.strip_prefix('≤'))
        .or_else(|| trimmed.strip_prefix('<'))
        .or_else(|| {
            ["以下", "未満", "まで"]
                .into_iter()
                .find_map(|suffix| trimmed.strip_suffix(suffix))
        })?
        .trim();
    if rest.is_empty() {
        return None;
    }

    let to = parse_endpoint(rest, ctx)?;
    if to.kind != Kind::Quantity || to.value? < 0.0 {
        return None;
    }
    let from = zero_like_quantity(&to)?;
    Some(Reading::range(from, to, 0.86))
}

fn zero_like_quantity(reading: &Reading) -> Option<Reading> {
    Some(Reading::quantity(
        0.0,
        reading.unit.as_deref()?,
        reading.dimension?,
        reading.provenance.unwrap_or(Provenance::TradeCustom),
        reading.approximate.unwrap_or(false),
        0.86,
    ))
}

fn mark_approximate(reading: &mut Reading) {
    reading.approximate = Some(true);
    if let Some(confidence) = reading.confidence.as_mut() {
        *confidence *= 0.9;
    }
    if let Some(range) = reading.range.as_mut() {
        mark_approximate(&mut range.from);
        mark_approximate(&mut range.to);
    }
}

fn strip_prefix_ascii_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    if text.len() < prefix.len() {
        return None;
    }
    let candidate = text.get(..prefix.len())?;
    candidate
        .eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

fn parse_ambiguous_number(text: &str, ctx: &ParseCtx) -> Option<AmbiguousParse> {
    if ctx.number_format != NumberFormat::Auto {
        return None;
    }
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
    let builtin_suffixes = [
        "㎡", "m^2", "m2", "平米", "帖", "畳", "坪", "cm", "mm", "m", "kg", "g", "minutes",
        "minute", "mins", "min", "hours", "hour", "hrs", "hr", "days", "day", "日",
    ];
    let all_builtin_suffixes = || {
        builtin_suffixes.into_iter().chain(
            UNIT_DEFS
                .iter()
                .flat_map(|unit| unit.aliases.iter().copied()),
        )
    };
    let mut best = all_builtin_suffixes()
        .filter(|suffix| trimmed.ends_with(suffix))
        .max_by_key(|suffix| suffix.len())
        .or_else(|| {
            all_builtin_suffixes()
                .filter(|suffix| ends_with_ascii_case(trimmed, suffix))
                .max_by_key(|suffix| suffix.len())
        });

    for unit in &ctx.custom_units {
        for suffix in
            core::iter::once(unit.id.as_str()).chain(unit.aliases.iter().map(String::as_str))
        {
            if (trimmed.ends_with(suffix)
                || best.is_none() && ends_with_ascii_case(trimmed, suffix))
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
    let normalized = normalize_input_cow(text);
    let text = normalized.trim();
    let features = InputFeatures::new(text);

    if features.maybe_date
        && let Some(reading) = parse_relative_date(text, ctx)
    {
        return Some(reading);
    }
    if features.maybe_japanese_length
        && let Some(reading) = parse_japanese_length(text)
    {
        return Some(reading);
    }
    if features.maybe_tatami
        && let Some(reading) = parse_tatami_area(text)
    {
        return Some(reading);
    }
    if features.maybe_tsubo
        && let Some(reading) = parse_tsubo_area(text)
    {
        return Some(reading);
    }
    if features.maybe_area
        && let Some(reading) = parse_square_meter(text)
    {
        return Some(reading);
    }
    if features.maybe_temperature
        && let Some(reading) = parse_temperature(text)
    {
        return Some(reading);
    }
    if features.maybe_quantity
        && let Some(reading) = parse_registered_quantity(text, ctx)
    {
        return Some(reading);
    }
    if features.maybe_metric_length
        && let Some(reading) = parse_metric_length(text)
    {
        return Some(reading);
    }
    if features.maybe_mass
        && let Some(reading) = parse_mass(text)
    {
        return Some(reading);
    }
    if features.maybe_clock
        && let Some(reading) = parse_clock_time(text)
    {
        return Some(reading);
    }
    if features.maybe_duration
        && let Some(reading) = parse_duration(text)
    {
        return Some(reading);
    }
    if features.maybe_feet_inches
        && let Some(reading) = parse_feet_inches(text)
    {
        return Some(reading);
    }
    if features.maybe_currency
        && let Some((best, _, _)) = parse_currency(text, ctx)
    {
        return Some(best);
    }
    if features.maybe_number {
        return parse_plain_number(text);
    }
    None
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

    if lowered == "yesterday" || text == "昨日" || text == "昨天" {
        return from_jiff_date(base.checked_sub(1.day()).ok()?)
            .map(|date| Reading::date(date, 0.98));
    }

    if text == "一昨日" || text == "前天" {
        return from_jiff_date(base.checked_sub(2.days()).ok()?)
            .map(|date| Reading::date(date, 0.97));
    }

    if lowered == "tomorrow"
        || text == "mañana"
        || text == "demain"
        || text == "amanhã"
        || text == "明天"
    {
        return from_jiff_date(base.tomorrow().ok()?).map(|date| Reading::date(date, 0.98));
    }

    if text == "明日" {
        return from_jiff_date(base.tomorrow().ok()?).map(|date| Reading::date(date, 0.98));
    }

    if text == "明後日" || text == "后天" || text == "後天" || text == "pasado mañana" {
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

    if let Some(days_text) = lowered
        .strip_suffix(" days ago")
        .or_else(|| lowered.strip_suffix(" day ago"))
    {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_sub(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(days_text) = text.strip_suffix("日後") {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_add(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(days_text) = text.strip_suffix("日前") {
        let days = parse_whole_i64(days_text.trim())?;
        return from_jiff_date(base.checked_sub(days.days()).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_prefix("next ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_suffix(" prochain") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_suffix(" que vem") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = lowered.strip_prefix("this ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return date_in_current_week(base, weekday).map(|date| Reading::date(date, 0.95));
    }

    if let Some(weekday_text) = lowered.strip_prefix("last ") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(-1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.95));
    }

    if let Some(weekday_text) = text.strip_prefix("来週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_prefix("下周") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.96));
    }

    if let Some(weekday_text) = text.strip_prefix("今週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return date_in_current_week(base, weekday).map(|date| Reading::date(date, 0.95));
    }

    if let Some(weekday_text) = text.strip_prefix("先週") {
        let weekday = parse_weekday(weekday_text.trim())?;
        return from_jiff_date(base.nth_weekday(-1, weekday).ok()?)
            .map(|date| Reading::date(date, 0.95));
    }

    if let Some(date) = parse_numeric_slash_date(text, ctx) {
        return Some(Reading::date(date, 0.94));
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
fn parse_numeric_slash_date(text: &str, ctx: &ParseCtx) -> Option<Date> {
    let mut parts = text.trim().split('/');
    let first = parse_whole_i64(parts.next()?.trim())?;
    let second = parse_whole_i64(parts.next()?.trim())?;
    let year = parse_whole_i64(parts.next()?.trim())?;
    if parts.next().is_some() {
        return None;
    }
    let year = i32::try_from(year).ok()?;
    let (month, day) = if ctx.locale == Some(Locale::EnGb) {
        (second, first)
    } else {
        (first, second)
    };
    let date = jiff::civil::Date::new(
        i16::try_from(year).ok()?,
        i8::try_from(month).ok()?,
        i8::try_from(day).ok()?,
    )
    .ok()?;
    from_jiff_date(date)
}

#[cfg(feature = "dates-jiff")]
fn date_in_current_week(base: jiff::civil::Date, weekday: jiff::civil::Weekday) -> Option<Date> {
    use jiff::ToSpan;

    let delta = (weekday_number(weekday) - weekday_number(base.weekday()) + 7) % 7;
    let date = base.checked_add(i64::from(delta).days()).ok()?;
    from_jiff_date(date)
}

#[cfg(feature = "dates-jiff")]
fn weekday_number(weekday: jiff::civil::Weekday) -> i32 {
    match weekday {
        jiff::civil::Weekday::Monday => 1,
        jiff::civil::Weekday::Tuesday => 2,
        jiff::civil::Weekday::Wednesday => 3,
        jiff::civil::Weekday::Thursday => 4,
        jiff::civil::Weekday::Friday => 5,
        jiff::civil::Weekday::Saturday => 6,
        jiff::civil::Weekday::Sunday => 7,
    }
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
        "lunes" => Some(jiff::civil::Weekday::Monday),
        "martes" => Some(jiff::civil::Weekday::Tuesday),
        "miércoles" | "miercoles" => Some(jiff::civil::Weekday::Wednesday),
        "jueves" => Some(jiff::civil::Weekday::Thursday),
        "viernes" => Some(jiff::civil::Weekday::Friday),
        "sábado" | "sabado" => Some(jiff::civil::Weekday::Saturday),
        "domingo" => Some(jiff::civil::Weekday::Sunday),
        "lundi" => Some(jiff::civil::Weekday::Monday),
        "mardi" => Some(jiff::civil::Weekday::Tuesday),
        "mercredi" => Some(jiff::civil::Weekday::Wednesday),
        "jeudi" => Some(jiff::civil::Weekday::Thursday),
        "vendredi" => Some(jiff::civil::Weekday::Friday),
        "samedi" => Some(jiff::civil::Weekday::Saturday),
        "dimanche" => Some(jiff::civil::Weekday::Sunday),
        "segunda-feira" | "segunda" => Some(jiff::civil::Weekday::Monday),
        "terça-feira" | "terca-feira" | "terça" | "terca" => Some(jiff::civil::Weekday::Tuesday),
        "quarta-feira" | "quarta" => Some(jiff::civil::Weekday::Wednesday),
        "quinta-feira" | "quinta" => Some(jiff::civil::Weekday::Thursday),
        "sexta-feira" | "sexta" => Some(jiff::civil::Weekday::Friday),
        "月曜日" | "月曜" | "月" => Some(jiff::civil::Weekday::Monday),
        "火曜日" | "火曜" | "火" => Some(jiff::civil::Weekday::Tuesday),
        "水曜日" | "水曜" | "水" => Some(jiff::civil::Weekday::Wednesday),
        "木曜日" | "木曜" | "木" => Some(jiff::civil::Weekday::Thursday),
        "金曜日" | "金曜" | "金" => Some(jiff::civil::Weekday::Friday),
        "土曜日" | "土曜" | "土" => Some(jiff::civil::Weekday::Saturday),
        "日曜日" | "日曜" | "日" => Some(jiff::civil::Weekday::Sunday),
        "周一" | "星期一" | "一" => Some(jiff::civil::Weekday::Monday),
        "周二" | "星期二" | "二" => Some(jiff::civil::Weekday::Tuesday),
        "周三" | "星期三" | "三" => Some(jiff::civil::Weekday::Wednesday),
        "周四" | "星期四" | "四" => Some(jiff::civil::Weekday::Thursday),
        "周五" | "星期五" | "五" => Some(jiff::civil::Weekday::Friday),
        "周六" | "星期六" | "六" => Some(jiff::civil::Weekday::Saturday),
        "周日" | "星期日" | "星期天" | "天" => Some(jiff::civil::Weekday::Sunday),
        _ => None,
    }
}

fn parse_number(text: &str) -> Option<f64> {
    parse_number_with_format(text, NumberFormat::Auto)
}

fn parse_number_ctx(text: &str, ctx: &ParseCtx) -> Option<f64> {
    parse_number_with_format(text, ctx.number_format)
}

fn parse_number_with_format(text: &str, number_format: NumberFormat) -> Option<f64> {
    let normalized_input = normalize_input_cow(text);
    let trimmed = normalized_input.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(value) = parse_japanese_large_number(trimmed) {
        return Some(value);
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

    let normalized = normalize_locale_number(trimmed, number_format)?;

    if normalized
        .chars()
        .all(|ch| ch.is_ascii_digit() || matches!(ch, '.' | '-' | '+'))
    {
        normalized.parse::<f64>().ok()
    } else {
        None
    }
}

fn normalize_locale_number(text: &str, number_format: NumberFormat) -> Option<String> {
    let compact = text
        .chars()
        .filter(|ch| !matches!(ch, ' ' | '_' | '\'' | '\u{00A0}' | '\u{202F}' | '\u{2009}'))
        .collect::<String>();
    if compact.is_empty() {
        return None;
    }

    if compact.contains(',') && compact.contains('.') {
        let (decimal, grouping) = match number_format {
            NumberFormat::CommaDecimal => (',', '.'),
            NumberFormat::DotDecimal => ('.', ','),
            NumberFormat::Auto => {
                let comma = compact.rfind(',')?;
                let dot = compact.rfind('.')?;
                if comma > dot { (',', '.') } else { ('.', ',') }
            }
        };
        return normalize_decimal_grouped_number(&compact, decimal, grouping);
    }

    if compact.contains(',') {
        if number_format == NumberFormat::CommaDecimal {
            return Some(compact.replace(',', "."));
        }
        if number_format == NumberFormat::DotDecimal {
            return normalize_grouped_decimal_free_number(&compact, ',');
        }
        if valid_grouped_number(&compact) || valid_indian_grouped_number(&compact) {
            return Some(compact.replace(',', ""));
        }
        if compact.matches(',').count() == 1 {
            return Some(compact.replace(',', "."));
        }
        return None;
    }

    if compact.matches('.').count() > 1 {
        if valid_dot_grouped_number(&compact) {
            return Some(compact.replace('.', ""));
        }
        return None;
    }

    Some(compact)
}

fn normalize_grouped_decimal_free_number(text: &str, grouping: char) -> Option<String> {
    let ungrouped = text.replace(grouping, "");
    if ungrouped
        .trim_start_matches(['-', '+'])
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.')
    {
        Some(ungrouped)
    } else {
        None
    }
}

fn normalize_decimal_grouped_number(text: &str, decimal: char, grouping: char) -> Option<String> {
    let (whole, fraction) = text.rsplit_once(decimal)?;
    if fraction.is_empty() || !fraction.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let whole_without_groups = whole.replace(grouping, "");
    if whole_without_groups.is_empty()
        || !whole_without_groups
            .trim_start_matches(['-', '+'])
            .chars()
            .all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    Some(format!("{whole_without_groups}.{fraction}"))
}

fn parse_japanese_large_number(text: &str) -> Option<f64> {
    if !text.contains(['万', '億', '兆']) {
        return None;
    }
    let mut total = 0.0;
    let mut rest = text.trim();
    for (unit, factor) in [
        ('兆', 1_000_000_000_000.0),
        ('億', 100_000_000.0),
        ('万', 10_000.0),
    ] {
        if let Some((head, tail)) = rest.split_once(unit) {
            let value = if head.trim().is_empty() {
                1.0
            } else {
                parse_number_without_large_units(head.trim())?
            };
            total += value * factor;
            rest = tail;
        }
    }
    if !rest.trim().is_empty() {
        total += parse_number_without_large_units(rest.trim())?;
    }
    Some(total)
}

fn parse_number_without_large_units(text: &str) -> Option<f64> {
    if let Some(value) = parse_cjk_number(text) {
        return Some(value as f64);
    }
    normalize_locale_number(text, NumberFormat::Auto)?
        .parse::<f64>()
        .ok()
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
            '億' => {
                section += number;
                total += section * 100_000_000;
                section = 0;
                number = 0;
                saw = true;
                continue;
            }
            '兆' => {
                section += number;
                total += section * 1_000_000_000_000;
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

fn valid_dot_grouped_number(text: &str) -> bool {
    let signless = text.trim_start_matches(['-', '+']);
    let groups: Vec<&str> = signless.split('.').collect();
    groups.len() > 1
        && !groups[0].is_empty()
        && groups[0].len() <= 3
        && groups.iter().enumerate().all(|(idx, group)| {
            group.chars().all(|ch| ch.is_ascii_digit()) && (idx == 0 || group.len() == 3)
        })
}

fn valid_indian_grouped_number(text: &str) -> bool {
    let (whole, decimal) = text.split_once('.').unwrap_or((text, ""));
    if !decimal.is_empty() && !decimal.chars().all(|ch| ch.is_ascii_digit()) {
        return false;
    }
    let signless = whole.trim_start_matches(['-', '+']);
    let groups: Vec<&str> = signless.split(',').collect();
    if groups.len() < 3 || groups[0].is_empty() || groups[0].len() > 2 {
        return false;
    }
    let last_is_three = groups
        .last()
        .is_some_and(|group| group.len() == 3 && group.chars().all(|ch| ch.is_ascii_digit()));
    let middle_are_two = groups[1..groups.len() - 1]
        .iter()
        .all(|group| group.len() == 2 && group.chars().all(|ch| ch.is_ascii_digit()));
    groups[0].chars().all(|ch| ch.is_ascii_digit()) && middle_are_two && last_is_three
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
    "yesterday",
    "next monday",
    "next tuesday",
    "next wednesday",
    "next thursday",
    "next friday",
    "next saturday",
    "next sunday",
    "this friday",
    "last friday",
    "mañana",
    "demain",
    "amanhã",
    "明天",
    "今日",
    "明日",
    "昨日",
    "来週月曜日",
    "来週火曜日",
    "来週水曜日",
    "来週木曜日",
    "来週金曜日",
    "来週土曜日",
    "来週日曜日",
    "下周五",
];

const TIME_COMPLETIONS: &[&str] = &["noon", "midnight", "午後3時", "午前9時"];

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
        Some(Kind::Recurrence) => false,
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
            if !same_ascii_first_char(&normalized, alias) {
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
    .or_else(|| suggest_non_ascii_unit(token))
}

fn suggest_non_ascii_unit(token: &str) -> Option<Suggestion> {
    if token.is_ascii() || token.chars().count() > 8 {
        return None;
    }
    let mut best: Option<(&'static str, usize, usize)> = None;
    for unit in UNIT_DEFS {
        for alias in unit_lookup_aliases(unit) {
            if alias.is_ascii() {
                continue;
            }
            let alias_len = alias.chars().count();
            if token.chars().count().abs_diff(alias_len) > 2 {
                continue;
            }
            let distance = levenshtein_chars(token, alias);
            let limit = if alias_len <= 2 { 1 } else { 2 };
            if distance > 0 && distance <= limit && best.is_none_or(|(_, best, _)| distance < best)
            {
                best = Some((unit.id, distance, alias_len));
            }
        }
    }
    best.map(|(to, distance, alias_len)| {
        let max_len = token.chars().count().max(alias_len) as f64;
        Suggestion {
            from: token.to_owned(),
            to: to.to_owned(),
            score: Some(1.0 - distance as f64 / max_len),
        }
    })
}

fn same_ascii_first_char(left: &str, right: &str) -> bool {
    match (left.as_bytes().first(), right.as_bytes().first()) {
        (Some(left), Some(right)) => left.eq_ignore_ascii_case(right),
        _ => false,
    }
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

fn levenshtein_chars(left: &str, right: &str) -> usize {
    let right_chars: Vec<char> = right.chars().collect();
    let mut prev: Vec<usize> = (0..=right_chars.len()).collect();
    let mut curr = vec![0; right_chars.len() + 1];

    for (i, left_char) in left.chars().enumerate() {
        curr[0] = i + 1;
        for (j, right_char) in right_chars.iter().enumerate() {
            let substitution = prev[j] + usize::from(left_char != *right_char);
            let insertion = curr[j] + 1;
            let deletion = prev[j + 1] + 1;
            curr[j + 1] = substitution.min(insertion).min(deletion);
        }
        core::mem::swap(&mut prev, &mut curr);
    }

    prev[right_chars.len()]
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
    approximation_with_span(ref_text, reason, span(ref_text))
}

fn approximation_with_span(ref_text: &str, reason: &str, span: Span) -> Approximation {
    Approximation {
        code: IssueCode::Approximation,
        ref_text: ref_text.to_owned(),
        reason: reason.to_owned(),
        relative_error: None,
        span,
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

#[cfg(any(feature = "wasm", test))]
fn parsed_summary_json(parsed: &Parsed) -> String {
    let mut json = String::new();
    json.push_str("{\"ok\":");
    json.push_str(if parsed.best.is_some() {
        "true"
    } else {
        "false"
    });
    json.push_str(",\"input\":");
    push_json_string(&mut json, &parsed.input);
    json.push_str(",\"best\":");
    if let Some(best) = &parsed.best {
        push_reading_json(&mut json, best);
    } else {
        json.push_str("null");
    }
    json.push_str(",\"issues\":[");
    for (idx, issue) in ranked_findings(parsed).iter().enumerate() {
        if idx > 0 {
            json.push(',');
        }
        json.push_str("{\"code\":");
        push_json_string(&mut json, issue.code.as_str());
        json.push_str(",\"severity\":");
        push_json_string(&mut json, issue.severity.as_str());
        json.push_str(",\"rank\":");
        json.push_str(&issue.rank.to_string());
        json.push_str(",\"ref_text\":");
        push_json_string(&mut json, &issue.ref_text);
        json.push('}');
    }
    json.push_str("]}");
    json
}

#[cfg(feature = "wasm")]
fn parsed_matches_summary_json(source: &str, matches: &[ParsedMatch]) -> String {
    let mut json = String::new();
    json.push('[');
    for (idx, parsed_match) in matches.iter().enumerate() {
        if idx > 0 {
            json.push(',');
        }
        json.push_str("{\"start\":");
        json.push_str(&parsed_match.start.to_string());
        json.push_str(",\"end\":");
        json.push_str(&parsed_match.end.to_string());
        json.push_str(",\"byteStart\":");
        json.push_str(&parsed_match.start.to_string());
        json.push_str(",\"byteEnd\":");
        json.push_str(&parsed_match.end.to_string());
        let char_start = byte_to_char_offset(source, parsed_match.start);
        let char_end = byte_to_char_offset(source, parsed_match.end);
        json.push_str(",\"charStart\":");
        json.push_str(&char_start.to_string());
        json.push_str(",\"charEnd\":");
        json.push_str(&char_end.to_string());
        json.push_str(",\"text\":");
        push_json_string(&mut json, &parsed_match.text);
        json.push_str(",\"parsed\":");
        json.push_str(&parsed_summary_json(&parsed_match.parsed));
        json.push('}');
    }
    json.push(']');
    json
}

#[cfg(feature = "wasm")]
fn byte_to_char_offset(text: &str, byte_offset: usize) -> usize {
    text[..byte_offset].chars().count()
}

#[cfg(any(feature = "wasm", test))]
fn push_reading_json(json: &mut String, reading: &Reading) {
    json.push_str("{\"kind\":");
    push_json_string(json, kind_str(reading.kind));
    if let Some(custom_kind) = &reading.custom_kind {
        json.push_str(",\"customKind\":");
        push_json_string(json, custom_kind);
    }
    if let Some(value) = reading.value {
        json.push_str(",\"value\":");
        json.push_str(&format_number(value));
    }
    if let Some(unit) = &reading.unit {
        json.push_str(",\"unit\":");
        push_json_string(json, unit);
    }
    if let Some(dimension) = reading.dimension {
        json.push_str(",\"dimension\":");
        push_json_string(json, dimension.as_str());
    }
    if let Some(date) = &reading.date {
        json.push_str(",\"date\":");
        push_json_string(json, date);
    }
    if let Some(recurrence) = &reading.recurrence {
        json.push_str(",\"recurrence\":");
        push_json_string(json, recurrence);
    }
    if let Some(timezone) = &reading.timezone {
        json.push_str(",\"timezone\":");
        push_json_string(json, timezone);
    }
    json.push('}');
}

fn kind_str(kind: Kind) -> &'static str {
    match kind {
        Kind::Quantity => "quantity",
        Kind::Date => "date",
        Kind::Range => "range",
        Kind::Number => "number",
        Kind::Recurrence => "recurrence",
    }
}

#[cfg(any(feature = "wasm", test))]
fn push_json_string(json: &mut String, value: &str) {
    json.push('"');
    for ch in value.chars() {
        match ch {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            ch if ch.is_control() => json.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => json.push(ch),
        }
    }
    json.push('"');
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
        assert!(parse_input_schema_json().contains("\"timezone\""));
        assert!(parsed_output_schema_json().contains("\"findings\""));
        assert!(parsed_output_schema_json().contains("\"currency\""));
        assert!(parsed_output_schema_json().contains("\"temperature\""));
        assert!(parsed_output_schema_json().contains("\"recurrence\""));
        assert!(parsed_output_schema_json().contains("\"timezone\""));
        assert!(mcp_tool_schema_json().contains("unravel_nl_parse"));
        assert!(mcp_tool_schema_json().contains("inputSchema"));
        assert!(mcp_tool_schema_json().contains("outputSchema"));
        assert_eq!(
            IssueCode::TimezoneUnsupported.as_str(),
            "TIMEZONE_UNSUPPORTED"
        );
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
    fn serializes_parsed_summary_json_for_adapters() {
        let parsed = parse("5尺3寸", None);
        let json = parsed_summary_json(&parsed);
        assert!(json.contains("\"ok\":true"));
        assert!(json.contains("\"kind\":\"quantity\""));
        assert!(json.contains("\"unit\":\"m\""));
        assert!(json.contains("\"dimension\":\"length\""));

        let failed = parsed_summary_json(&parse("3pm Europe/Paris", None));
        assert!(failed.contains("\"ok\":false"));
        assert!(failed.contains("\"code\":\"TIMEZONE_UNSUPPORTED\""));
        assert!(failed.contains("\"severity\":\"error\""));
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

        let japanese_afternoon = parse("午後3時", None).best.expect("Japanese afternoon");
        assert_eq!(japanese_afternoon.unit.as_deref(), Some("s"));
        assert_eq!(japanese_afternoon.dimension, Some(Dimension::Time));
        assert_close(japanese_afternoon.value.unwrap(), 15.0 * 3600.0);

        let japanese_morning = parse("午前9時30分", None).best.expect("Japanese morning");
        assert_close(japanese_morning.value.unwrap(), 9.5 * 3600.0);

        let japanese_half = parse("午後3時半", None).best.expect("Japanese half hour");
        assert_close(japanese_half.value.unwrap(), 15.5 * 3600.0);
    }

    #[test]
    fn parses_timezone_qualified_clock_to_utc() {
        let parsed = parse("3pm EST", None);
        let best = parsed.best.expect("timezone clock");
        assert_eq!(best.unit.as_deref(), Some("s"));
        assert_eq!(best.dimension, Some(Dimension::Time));
        assert_eq!(best.timezone.as_deref(), Some("UTC"));
        assert_close(best.value.unwrap(), 20.0 * 3600.0);

        let tokyo = parse("9:30 JST", None).best.expect("JST clock");
        assert_eq!(tokyo.timezone.as_deref(), Some("UTC"));
        assert_close(tokyo.value.unwrap(), 30.0 * 60.0);
    }

    #[cfg(feature = "timezones-jiff")]
    #[test]
    fn parses_iana_timezone_with_explicit_reference_date() {
        let summer = parse(
            "3pm Europe/Paris",
            Some(ParseCtx {
                reference_date: Date::new(2026, 7, 20),
                ..ParseCtx::default()
            }),
        )
        .best
        .expect("summer IANA timezone");
        assert_eq!(summer.timezone.as_deref(), Some("UTC"));
        assert_close(summer.value.unwrap(), 13.0 * 3600.0);

        let winter = parse(
            "3pm Europe/Paris",
            Some(ParseCtx {
                reference_date: Date::new(2026, 1, 20),
                ..ParseCtx::default()
            }),
        )
        .best
        .expect("winter IANA timezone");
        assert_eq!(winter.timezone.as_deref(), Some("UTC"));
        assert_close(winter.value.unwrap(), 14.0 * 3600.0);
    }

    #[test]
    fn rejects_unsupported_timezone_policy() {
        let parsed = parse("3pm Europe/Paris", None);
        assert!(parsed.best.is_none());
        assert_eq!(
            parsed.findings.skipped[0].code,
            IssueCode::TimezoneUnsupported
        );
        assert_eq!(parsed.findings.skipped[0].ref_text, "Europe/Paris");
        assert_eq!(parsed.findings.skipped[0].span.start, 4);
        assert_eq!(parsed.findings.skipped[0].span.end, 16);
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
    fn parses_tolerance_and_bound_ranges() {
        let tolerance = parse("10 ± 0.5 mm", None).best.expect("tolerance");
        assert_eq!(tolerance.kind, Kind::Range);
        let range = tolerance.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("m"));
        assert_close(range.from.value.unwrap(), 0.0095);
        assert_close(range.to.value.unwrap(), 0.0105);

        let upper = parse("under 10 minutes", None).best.expect("upper bound");
        assert_eq!(upper.kind, Kind::Range);
        let range = upper.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("s"));
        assert_close(range.from.value.unwrap(), 0.0);
        assert_close(range.to.value.unwrap(), 600.0);

        let japanese_upper = parse("10mm以下", None).best.expect("Japanese upper bound");
        let range = japanese_upper.range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("m"));
        assert_close(range.from.value.unwrap(), 0.0);
        assert_close(range.to.value.unwrap(), 0.01);
    }

    #[test]
    fn parses_approximate_and_fuzzy_readings() {
        let approx = parse("about 20C", None);
        let best = approx.best.expect("approximate temperature");
        assert_eq!(best.dimension, Some(Dimension::Temperature));
        assert_eq!(best.approximate, Some(true));
        assert_eq!(approx.findings.approximations[0].ref_text, "about");

        let japanese_approx = parse("約20kg", None);
        let best = japanese_approx.best.expect("Japanese approximate mass");
        assert_eq!(best.dimension, Some(Dimension::Mass));
        assert_eq!(best.approximate, Some(true));
        assert_eq!(japanese_approx.findings.approximations[0].ref_text, "約");

        let strict = parse(
            "about 20C",
            Some(ParseCtx {
                strictness: Strictness::Strict,
                ..ParseCtx::default()
            }),
        );
        assert!(strict.best.is_none());
        assert_eq!(strict.findings.skipped[0].code, IssueCode::Approximation);

        let few = parse("a few minutes", None);
        let range = few.best.expect("few range").range.expect("range");
        assert_close(range.from.value.unwrap(), 120.0);
        assert_close(range.to.value.unwrap(), 240.0);
        assert_eq!(few.findings.approximations[0].ref_text, "a few");

        let hot = parse(
            "it's hot",
            Some(ParseCtx {
                expected_dimension: Some(Dimension::Temperature),
                ..ParseCtx::default()
            }),
        );
        let range = hot.best.expect("hot range").range.expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("C"));
        assert_close(range.from.value.unwrap(), 27.0);
        assert_close(range.to.value.unwrap(), 35.0);

        let japanese_hot = parse(
            "今日は暑い",
            Some(ParseCtx {
                expected_dimension: Some(Dimension::Temperature),
                ..ParseCtx::default()
            }),
        );
        let range = japanese_hot
            .best
            .expect("Japanese hot range")
            .range
            .expect("range");
        assert_eq!(range.from.unit.as_deref(), Some("C"));
        assert_close(range.from.value.unwrap(), 27.0);
        assert_close(range.to.value.unwrap(), 35.0);
        assert_eq!(japanese_hot.findings.approximations[0].ref_text, "暑い");
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
    fn typo_corrects_japanese_units_in_forgiving_mode() {
        let parsed = parse("10平目", None);
        let best = parsed.best.expect("best reading");
        assert_eq!(best.unit.as_deref(), Some("m2"));
        assert_eq!(best.dimension, Some(Dimension::Area));
        assert_close(best.value.unwrap(), 10.0);
        assert_eq!(parsed.suggestions[0].from, "平目");
        assert_eq!(parsed.suggestions[0].to, "m2");
        assert_eq!(
            parsed.findings.ambiguities[0].code,
            IssueCode::TypoCorrected
        );

        let confirm = parse(
            "10平目",
            Some(ParseCtx {
                strictness: Strictness::Confirm,
                ..ParseCtx::default()
            }),
        );
        assert!(confirm.best.is_none());
        assert_eq!(confirm.suggestions[0].to, "m2");
        assert_eq!(confirm.findings.skipped[0].code, IssueCode::TypoCorrected);
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
            timezone: Some("Asia/Tokyo".to_owned()),
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
    fn parses_broader_relative_dates_with_jiff() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::En),
            reference_date: Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        });

        assert_eq!(
            parse("yesterday", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-18")
        );
        assert_eq!(
            parse("2 days ago", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-17")
        );
        assert_eq!(
            parse("this friday", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-24")
        );
        assert_eq!(
            parse("last friday", ctx).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
    }

    #[cfg(feature = "dates-jiff")]
    #[test]
    fn parses_broader_japanese_relative_dates_with_jiff() {
        let ctx = Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: Date::new(2026, 7, 19),
            timezone: Some("Asia/Tokyo".to_owned()),
            ..ParseCtx::default()
        });

        assert_eq!(
            parse("昨日", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-18")
        );
        assert_eq!(
            parse("一昨日", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
        assert_eq!(
            parse("2日前", ctx.clone()).best.unwrap().date.as_deref(),
            Some("2026-07-17")
        );
        assert_eq!(
            parse("今週金曜日", ctx.clone())
                .best
                .unwrap()
                .date
                .as_deref(),
            Some("2026-07-24")
        );
        assert_eq!(
            parse("先週金曜日", ctx).best.unwrap().date.as_deref(),
            Some("2026-07-17")
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
