use crate::*;

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

pub(crate) struct AmbiguousParse {
    pub(crate) best: Option<Reading>,
    pub(crate) alternatives: Vec<Reading>,
    pub(crate) ambiguity: Ambiguity,
}

pub(crate) struct ParsedReading {
    pub(crate) reading: Reading,
    pub(crate) approximations: Vec<Approximation>,
}
