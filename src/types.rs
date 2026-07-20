use crate::*;

/// Classifies a completion candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletionKind {
    /// A unit completion.
    Unit,
    /// A date completion.
    Date,
    /// A time completion.
    Time,
    /// A currency completion.
    Currency,
}

impl CompletionKind {
    /// Returns the stable lowercase identifier for this completion kind.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Date => "date",
            Self::Time => "time",
            Self::Currency => "currency",
        }
    }
}

/// Specifies a locale hint used during parsing or presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Locale {
    /// Japanese.
    Ja,
    /// Generic English.
    En,
    /// United States English.
    EnUs,
    /// British English.
    EnGb,
    /// A caller-provided locale tag.
    Other(String),
}

impl Locale {
    /// Returns the locale tag represented by this value.
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

/// Identifies the top-level kind of a parsed reading.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// A quantity with a unit or dimension.
    Quantity,
    /// A calendar date.
    Date,
    /// A bounded range of readings.
    Range,
    /// A unitless number.
    Number,
    /// A recurring schedule.
    Recurrence,
}

/// Identifies the dimension associated with a quantity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dimension {
    /// Length.
    Length,
    /// Area.
    Area,
    /// Mass.
    Mass,
    /// Time duration.
    Time,
    /// Volume.
    Volume,
    /// Monetary value.
    Currency,
    /// Temperature.
    Temperature,
    /// Speed.
    Speed,
    /// Digital data quantity.
    Data,
    /// Digital data transfer rate.
    DataRate,
    /// Volumetric flow rate.
    FlowRate,
    /// Amount concentration.
    Concentration,
    /// Acceleration.
    Acceleration,
    /// Force.
    Force,
    /// Torque.
    Torque,
    /// Pressure.
    Pressure,
    /// Power.
    Power,
    /// Electric charge.
    Charge,
    /// Electric potential difference.
    Voltage,
    /// Electric current.
    Current,
    /// Electrical resistance.
    Resistance,
    /// Illuminance.
    Illuminance,
    /// Radiation equivalent dose.
    RadiationEquivalentDose,
    /// Radioactivity.
    Radioactivity,
}

impl Dimension {
    /// Returns the stable lowercase identifier for this dimension.
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

/// Controls how readily the parser accepts ambiguous or informal input.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Strictness {
    /// Accepts forgiving interpretations of informal input.
    #[default]
    Forgiving,
    /// Retains interpretations that may require user confirmation.
    Confirm,
    /// Requires the strictest supported interpretation.
    Strict,
}

/// Selects how decimal and grouping punctuation is interpreted.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum NumberFormat {
    /// Infers punctuation from the input and parsing context.
    #[default]
    Auto,
    /// Treats a comma as the decimal separator.
    CommaDecimal,
    /// Treats a dot as the decimal separator.
    DotDecimal,
}

/// Hints which parser grammar is appropriate for the caller's task.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ParsePurpose {
    /// General-purpose parsing.
    #[default]
    General,
    /// Quantity-oriented parsing.
    Quantity,
    /// Number-oriented parsing.
    Number,
    /// Date-oriented parsing.
    Date,
    /// Recurrence-oriented parsing.
    Recurrence,
    /// Parsing restricted to dimensions accepted by an editor field.
    DimensionEditor,
}

impl ParsePurpose {
    /// Returns the stable lowercase identifier for this parsing purpose.
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

/// Controls which broad input shapes the parser may accept.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptOptions {
    /// Whether range expressions are accepted.
    pub ranges: bool,
    /// Whether conversion expressions are accepted.
    pub conversions: bool,
    /// Whether compound expressions are accepted.
    pub compounds: bool,
    /// Whether fuzzy quantity terms are accepted.
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

/// Identifies the standard or convention behind a quantity conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Provenance {
    /// An exact conversion defined by an international standard.
    InternationalExact,
    /// A conversion defined by Japanese statute.
    JapaneseStatute,
    /// A customary trade conversion.
    TradeCustom,
    /// An SI unit or SI multiple conversion.
    SiMultiple,
}

impl Provenance {
    /// Returns the stable lowercase identifier for this provenance.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InternationalExact => "international_exact",
            Self::JapaneseStatute => "japanese_statute",
            Self::TradeCustom => "trade_custom",
            Self::SiMultiple => "si_multiple",
        }
    }
}

/// Represents a civil calendar date.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Date {
    /// Calendar year.
    pub year: i32,
    /// Calendar month numbered from 1 through 12.
    pub month: u8,
    /// Day of the month numbered from 1 through 31.
    pub day: u8,
}

impl Date {
    /// Creates a date when the month and day fall within the supported numeric ranges.
    ///
    /// This checks only `month` in `1..=12` and `day` in `1..=31`; it does not
    /// validate month-specific day counts or leap years.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if (1..=12).contains(&month) && (1..=31).contains(&day) {
            Some(Self { year, month, day })
        } else {
            None
        }
    }

    /// Formats this date as `YYYY-MM-DD`.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Supplies optional hints, policies, and custom registries to parsing operations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseCtx {
    /// Locale hint used to disambiguate units and number formats.
    pub locale: Option<Locale>,
    /// Expected top-level reading kind.
    pub expect: Option<Kind>,
    /// Expected quantity dimension.
    pub expected_dimension: Option<Dimension>,
    /// Numeric punctuation policy.
    pub number_format: NumberFormat,
    /// Grammar-dispatch hint for the caller's parsing task.
    pub purpose: ParsePurpose,
    /// Controls which broad parser shapes are accepted.
    pub accept: AcceptOptions,
    /// Civil reference date used to resolve relative dates.
    pub reference_date: Option<Date>,
    /// Caller-supplied timezone hint for supported adapter layers.
    pub timezone: Option<String>,
    /// Strictness applied when accepting interpretations.
    pub strictness: Strictness,
    /// Caller-provided currency conversion rates.
    pub currency_rates: Vec<CurrencyRate>,
    /// Caller-provided unit definitions.
    pub custom_units: Vec<CustomUnit>,
    /// Caller-provided mappings for fuzzy quantity terms.
    pub fuzzy_profiles: Vec<FuzzyProfile>,
}

/// Defines a directional conversion between two currencies.
#[derive(Clone, Debug, PartialEq)]
pub struct CurrencyRate {
    /// Source currency code.
    pub from: String,
    /// Target currency code.
    pub to: String,
    /// Multiplier that converts an amount in `from` to an amount in `to`.
    pub factor: f64,
}

impl CurrencyRate {
    /// Creates a currency rate after normalizing both currency codes.
    pub fn new(from: &str, to: &str, factor: f64) -> Self {
        Self {
            from: normalize_currency_code(from).to_owned(),
            to: normalize_currency_code(to).to_owned(),
            factor,
        }
    }
}

/// Defines a caller-provided unit and its canonical conversion.
#[derive(Clone, Debug, PartialEq)]
pub struct CustomUnit {
    /// Stable identifier used to recognize the custom unit.
    pub id: String,
    /// Optional caller-defined reading kind assigned to matches.
    pub kind_id: Option<String>,
    /// Canonical output unit to which [`CustomUnit::factor`] converts values.
    pub canonical_unit: String,
    /// Additional accepted textual representations of the unit.
    pub aliases: Vec<String>,
    /// Dimension measured by the unit.
    pub dimension: Dimension,
    /// Multiplier that converts a value in this unit to [`CustomUnit::canonical_unit`].
    pub factor: f64,
    /// Whether conversion through this definition is approximate.
    pub approximate: bool,
}

impl CustomUnit {
    /// Creates an exact custom unit definition without a custom reading kind.
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

    /// Sets whether this custom unit's conversion is approximate.
    pub fn approximate(mut self, approximate: bool) -> Self {
        self.approximate = approximate;
        self
    }

    /// Assigns a caller-defined reading kind identifier.
    pub fn kind(mut self, kind_id: &str) -> Self {
        self.kind_id = Some(kind_id.to_owned());
        self
    }
}

/// Groups fuzzy quantity terms that share a dimension and canonical unit.
#[derive(Clone, Debug, PartialEq)]
pub struct FuzzyProfile {
    /// Stable identifier for the fuzzy profile.
    pub id: String,
    /// Dimension represented by the profile's terms.
    pub dimension: Dimension,
    /// Unit used for the bounds in each [`FuzzyTerm`].
    pub unit: String,
    /// Fuzzy terms recognized by this profile.
    pub terms: Vec<FuzzyTerm>,
}

impl FuzzyProfile {
    /// Creates a fuzzy profile from the supplied terms.
    pub fn new(id: &str, dimension: Dimension, unit: &str, terms: &[FuzzyTerm]) -> Self {
        Self {
            id: id.to_owned(),
            dimension,
            unit: unit.to_owned(),
            terms: terms.to_vec(),
        }
    }
}

/// Maps an informal quantity term to a numeric interval.
#[derive(Clone, Debug, PartialEq)]
pub struct FuzzyTerm {
    /// Text recognized as the fuzzy term.
    pub term: String,
    /// Inclusive lower bound in the containing profile's unit.
    pub low: f64,
    /// Inclusive upper bound in the containing profile's unit.
    pub high: f64,
}

impl FuzzyTerm {
    /// Creates a fuzzy term with the supplied lower and upper bounds.
    pub fn new(term: &str, low: f64, high: f64) -> Self {
        Self {
            term: term.to_owned(),
            low,
            high,
        }
    }
}

/// Supplies presentation hints when converting readings to human-readable text.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HumanizeCtx {
    /// Locale used to select human-readable formatting.
    pub locale: Option<Locale>,
}

/// Contains the result of parsing an input string.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    /// Original input supplied to the parser.
    pub input: String,
    /// Locale associated with this parse, when one was supplied.
    pub locale: Option<Locale>,
    /// Highest-ranked reading, or `None` when parsing produced no accepted reading.
    pub best: Option<Reading>,
    /// Other accepted interpretations of the input.
    pub alternatives: Vec<Reading>,
    /// Suggested textual replacements for the input.
    pub suggestions: Vec<Suggestion>,
    /// Structured issues discovered while parsing.
    pub findings: Findings,
}

/// Associates a parsed result with its matched span in a larger input string.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedMatch {
    /// Inclusive byte offset where the match begins in the original input.
    pub start: usize,
    /// Exclusive byte offset where the match ends in the original input.
    pub end: usize,
    /// Source text covered by the byte span.
    pub text: String,
    /// Parse result for the matched text.
    pub parsed: Parsed,
}

/// Represents one canonical interpretation of parsed input.
#[derive(Clone, Debug, PartialEq)]
pub struct Reading {
    /// Top-level kind of this reading.
    pub kind: Kind,
    /// Caller-defined kind identifier for a custom-unit reading.
    pub custom_kind: Option<String>,
    /// Canonical numeric value for quantity and number readings.
    pub value: Option<f64>,
    /// Canonical unit associated with [`Reading::value`].
    pub unit: Option<String>,
    /// Dimension associated with a quantity reading.
    pub dimension: Option<Dimension>,
    /// ISO-formatted date for a date reading.
    pub date: Option<String>,
    /// RRULE-style value for a recurrence reading.
    pub recurrence: Option<String>,
    /// Canonical timezone associated with a timezone-normalized reading.
    pub timezone: Option<String>,
    /// Endpoints for a range reading.
    pub range: Option<Box<RangeReading>>,
    /// Source or standard behind a quantity conversion.
    pub provenance: Option<Provenance>,
    /// Whether the interpretation or conversion is approximate.
    pub approximate: Option<bool>,
    /// Confidence in the range 0.0 to 1.0.
    pub confidence: Option<f64>,
}

impl Reading {
    /// Creates a canonical quantity reading.
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

    /// Creates a unitless number reading.
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

    /// Creates a date reading from a civil date.
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

    /// Creates a range reading from two endpoint readings.
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

    /// Creates a recurrence reading from an RRULE-style string.
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

/// Holds the endpoints of a range reading.
#[derive(Clone, Debug, PartialEq)]
pub struct RangeReading {
    /// Lower or starting endpoint of the range.
    pub from: Reading,
    /// Upper or ending endpoint of the range.
    pub to: Reading,
}

/// Describes a suggested textual replacement.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// Source text to replace.
    pub from: String,
    /// Suggested replacement text.
    pub to: String,
    /// Confidence in the suggestion in the range 0.0 to 1.0.
    pub score: Option<f64>,
}

/// Describes a text-completion candidate.
#[derive(Clone, Debug, PartialEq)]
pub struct Completion {
    /// Text to insert for the completion, such as the unit alias `metre`.
    pub value: String,
    /// Registry identifier this candidate resolves to, such as `m` for `metre`.
    pub canonical: Option<String>,
    /// Category of completion.
    pub kind: CompletionKind,
    /// Dimension associated with a unit completion.
    pub dimension: Option<Dimension>,
    /// Ranking score, higher first. `1.0` for an exact match, otherwise between
    /// `0.6` and `1.0` according to how much of the candidate the prefix covers.
    pub score: f64,
}

/// Describes a completed phrase together with its parsed reading.
#[derive(Clone, Debug, PartialEq)]
pub struct CompletionReading {
    /// Completed source text.
    pub text: String,
    /// Reading produced from the completed text.
    pub reading: Reading,
    /// Ranking score assigned to the completion.
    pub score: f64,
    /// Human-readable explanation of why the completion was produced.
    pub reason: String,
}

/// Presents a parsed object as a summary and named fields.
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceView {
    /// Stable type tag for the represented resource, such as `unravel.quantity`.
    pub object: String,
    /// Human-readable summary, produced by [`humanize`] for a reading.
    pub summary: String,
    /// Named fields exposed for the resource.
    pub fields: Vec<ResourceField>,
}

/// Holds one named field in a [`ResourceView`].
#[derive(Clone, Debug, PartialEq)]
pub struct ResourceField {
    /// Field name.
    pub name: String,
    /// Human-readable field value.
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
