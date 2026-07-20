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
    /// Infers punctuation from the input alone.
    ///
    /// [`ParseCtx::locale`] is **not** consulted: `1.234` reads the same under
    /// every locale, including a comma-decimal one. Only the string decides.
    ///
    /// Input whose shape leaves both readings open is reported as ambiguous
    /// rather than settled — `1.234` yields 1.234 with 1234 in
    /// [`Parsed::alternatives`] and an [`IssueCode::AmbiguousNumber`] finding.
    /// Input whose shape settles the question is read one way with no finding:
    /// `1.5` and `1.2345` are decimals, `1.234.567` is 1234567, and
    /// `1.234,56` is 1234.56.
    #[default]
    Auto,
    /// Treats a comma as the decimal separator and a dot as the grouping
    /// separator.
    ///
    /// The grouping is validated rather than guessed: a bare number whose dots
    /// do not delimit well-formed digit groups is refused with an
    /// [`IssueCode::NoValue`] finding instead of being silently regrouped into a
    /// different number.
    ///
    /// - Decimal comma: `1,5` is 1.5, `1,23` is 1.23, and `1.234,56` is 1234.56.
    /// - Grouping dot: `1.234` is 1234 and `1.234.567` is 1234567.
    /// - Refused: `1.5`, `1.23`, `1.2.3`, and `.5` — under this format those
    ///   dots would have to be grouping, and none of them groups three digits,
    ///   so reading them as decimals would contradict the declared format.
    ///
    /// The declared format is applied wherever a grammar reads its number
    /// through the parsing context, which includes quantities: `1.234 kg` is
    /// 1234 kg, and `$1.5` and `1.5 m 2.5 cm` are refused with an
    /// [`IssueCode::NoValue`] finding just as `1.5` alone is.
    ///
    /// `1.5 kg` is the exception, and parses as 1.5 kg: it is read by a
    /// fallback grammar that does not consult the declared format. The
    /// asymmetry is accepted rather than intended — do not read it as a rule
    /// that quantities are exempt from the format.
    CommaDecimal,
    /// Treats a dot as the decimal separator and a comma as the grouping
    /// separator.
    ///
    /// The mirror image of [`NumberFormat::CommaDecimal`], with the same
    /// grouping validation and the same [`IssueCode::NoValue`] refusal.
    ///
    /// - Decimal dot: `1.5` is 1.5 and `1,234.56` is 1234.56.
    /// - Grouping comma: `1,234` is 1234, `1,234,567` is 1234567, and the
    ///   Indian `12,34,567` shape is 1234567.
    /// - Refused: `1,5`, `1,23`, and `1,2,3` — under this format those commas
    ///   would have to be grouping, and none of them groups three digits. Also
    ///   refused: `1.234.567` and `1.2.3`, which carry more than one decimal
    ///   separator, exactly as `CommaDecimal` refuses `1,234,567`.
    DotDecimal,
}

/// Selects which parser grammar [`parse`] runs.
///
/// Carried by [`ParseCtx::purpose`], where it is a hard filter rather than a
/// hint: input the selected grammar does not read is refused rather than handed
/// to another grammar.
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
    /// Whether same-dimension registry compounds are accepted.
    ///
    /// This governs one grammar only: two or more whitespace-separated
    /// `<number> <unit>` pairs whose units all live in the unit registry and
    /// all share a dimension and canonical unit, such as `3 yd 2 ft` or
    /// `1 hour 30 minutes`. With `compounds: false` those are refused and
    /// reported as [`IssueCode::RejectedByPolicy`].
    ///
    /// Other multi-part grammars are *not* gated by this flag and still parse
    /// with `compounds: false`: feet-and-inches (`5ft 11in` → `1.8034 m`) and
    /// shakkanhō (`5尺3寸` → `1.60606… m`).
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
    ///
    /// The year is zero-padded to four characters, not to four digits, and
    /// [`Date::new`] does not bound it. Outside `0..=9999` the result is
    /// therefore not ISO 8601 `YYYY-MM-DD`: a negative year spends part of the
    /// width on its sign (`Date::new(-1, 1, 1)` formats as `-001-01-01`) and a
    /// year past 9999 simply grows (`Date::new(12345, 1, 1)` formats as
    /// `12345-01-01`). Callers that need a strict `YYYY-MM-DD` must keep the
    /// year in `0..=9999` themselves.
    pub fn iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Supplies optional hints, policies, and custom registries to parsing operations.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParseCtx {
    /// Locale hint used to disambiguate units.
    ///
    /// It settles locale-dependent units: `1.5 cups` reads as the US cup by
    /// default and as the imperial cup under [`Locale::EnGb`].
    ///
    /// It does **not** affect numeric punctuation. `1.234` produces a
    /// byte-identical [`Parsed`] under `None`, [`Locale::Ja`], [`Locale::En`],
    /// [`Locale::EnUs`], [`Locale::EnGb`], and any [`Locale::Other`] tag —
    /// including a comma-decimal one such as `de-DE`. Only
    /// [`ParseCtx::number_format`] decides that; see [`NumberFormat::Auto`].
    pub locale: Option<Locale>,
    /// Expected top-level reading kind. This does **not** constrain parsing.
    ///
    /// [`parse`] ignores it when choosing a grammar: `parse("5 kg", ..)` with
    /// `expect: Some(Kind::Date)` or `Some(Kind::Recurrence)` still returns the
    /// `5 kg` quantity. Only two places read it:
    ///
    /// - [`complete`] and [`complete_readings`] filter candidates by it, so
    ///   `Some(Kind::Date)` keeps date completions only and
    ///   `Some(Kind::Number)` or `Some(Kind::Recurrence)` drops every candidate.
    /// - A bare number parsed with `Some(Kind::Quantity)` gains a millimetre
    ///   length alternative and a [`IssueCode::UnitAssumed`] ambiguity.
    ///
    /// To actually restrict what is parsed, set [`ParseCtx::purpose`].
    pub expect: Option<Kind>,
    /// Expected quantity dimension. This is a hint, not a filter: [`parse`] and
    /// [`parse_quantity_fast`] report whatever dimension they read, so `5 kg`
    /// still parses as a mass when a length was expected. Only
    /// [`parse_dimensions_for_editor`] enforces it.
    pub expected_dimension: Option<Dimension>,
    /// Numeric punctuation policy.
    pub number_format: NumberFormat,
    /// Selects which grammar [`parse`] runs. This is a hard filter, not a hint.
    ///
    /// Input that the selected grammar does not read is refused rather than
    /// handed to another grammar: `parse("5 kg", ..)` with
    /// `purpose: ParsePurpose::Number` and `parse("next friday", ..)` with
    /// `purpose: ParsePurpose::Quantity` both return `best: None` and report
    /// [`IssueCode::NoValue`].
    ///
    /// For the four whole-string purposes this is exactly the corresponding
    /// narrow entry point: [`ParsePurpose::Quantity`] is
    /// [`parse_quantity_fast`], [`ParsePurpose::Number`] is
    /// [`parse_number_fast`], [`ParsePurpose::Date`] is [`parse_date_fast`],
    /// and [`ParsePurpose::Recurrence`] is [`parse_recurrence_fast`].
    ///
    /// [`ParsePurpose::DimensionEditor`] is the exception, and is **not**
    /// equivalent to [`parse_dimensions_for_editor`]. That function is an
    /// extractor: it scans free text for numeric candidates, infers an expected
    /// dimension from a neighbouring label, and keeps only the candidates that
    /// survive that filter. This purpose runs the same editor grammar over the
    /// whole input with neither step, so the two diverge in both directions:
    /// `parse("幅3m", ..)` with this purpose returns `best: None` while
    /// `parse_dimensions_for_editor("幅3m", ..)` extracts `3 m` from after the
    /// label, and `parse("3640", ..)` with this purpose returns the unitless
    /// number while `parse_dimensions_for_editor("3640", ..)` returns nothing,
    /// because an unlabelled bare number is not a dimension.
    ///
    /// Contrast [`ParseCtx::expected_dimension`], which really is only a hint.
    pub purpose: ParsePurpose,
    /// Controls which broad parser shapes are accepted.
    pub accept: AcceptOptions,
    /// Civil reference date used to resolve relative dates.
    pub reference_date: Option<Date>,
    /// Reserved for adapter layers. **Currently ignored by the parser.**
    ///
    /// No parsing path reads this field today: a parse with it set is identical
    /// to the same parse with it unset, and it never reaches
    /// [`Reading::timezone`], which is populated only from a timezone written
    /// in the input text. Setting it is harmless but has no effect.
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
///
/// The parser matches [`CurrencyRate::from`] and [`CurrencyRate::to`] against
/// normalized currency codes by exact string comparison, so both fields must
/// already hold the normalized (uppercase) form. Building the struct with a
/// literal and lowercase codes compiles but silently never matches:
/// `CurrencyRate { from: "usd".into(), to: "jpy".into(), factor: 150.0 }` leaves
/// `parse("USD 10 to JPY", ..)` with `best: None`, while the same rate built
/// with [`CurrencyRate::new`] converts it. Prefer [`CurrencyRate::new`], which
/// normalizes both codes for you.
#[derive(Clone, Debug, PartialEq)]
pub struct CurrencyRate {
    /// Source currency code, in normalized (uppercase) form such as `USD`.
    pub from: String,
    /// Target currency code, in normalized (uppercase) form such as `JPY`.
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::assert_close;

    /// `ParseCtx::locale` disambiguates units only. Numeric punctuation is
    /// decided by `ParseCtx::number_format` alone, so `1.234` parses
    /// identically under every locale, comma-decimal ones included.
    #[test]
    fn locale_disambiguates_units_but_not_number_formats() {
        let parse_with = |text: &str, locale: Option<Locale>| {
            parse(
                text,
                Some(ParseCtx {
                    locale,
                    ..ParseCtx::default()
                }),
            )
        };

        let baseline = parse_with("1.234", None);
        for locale in [
            Some(Locale::Ja),
            Some(Locale::En),
            Some(Locale::EnUs),
            Some(Locale::EnGb),
            Some(Locale::Other("de-DE".to_owned())),
        ] {
            let parsed = parse_with("1.234", locale.clone());
            assert_eq!(parsed.best, baseline.best, "{locale:?}");
            assert_eq!(parsed.alternatives, baseline.alternatives, "{locale:?}");
            assert_eq!(parsed.findings, baseline.findings, "{locale:?}");
        }

        // The unit half of the hint is real.
        let us = parse_with("1.5 cups", None).best.expect("a reading");
        let gb = parse_with("1.5 cups", Some(Locale::EnGb))
            .best
            .expect("a reading");
        assert_ne!(us.value, gb.value);
    }

    /// `ParseCtx::purpose` is a hard filter, not a hint: a grammar that does not
    /// read the input refuses it instead of falling back to another grammar.
    #[test]
    fn purpose_refuses_input_outside_the_selected_grammar() {
        let numbers_only = parse(
            "5 kg",
            Some(ParseCtx {
                purpose: ParsePurpose::Number,
                ..ParseCtx::default()
            }),
        );
        assert!(numbers_only.best.is_none());
        assert!(
            numbers_only
                .findings
                .skipped
                .iter()
                .any(|issue| issue.code == IssueCode::NoValue)
        );

        let quantities_only = parse(
            "next friday",
            Some(ParseCtx {
                purpose: ParsePurpose::Quantity,
                ..ParseCtx::default()
            }),
        );
        assert!(quantities_only.best.is_none());
        assert!(
            quantities_only
                .findings
                .skipped
                .iter()
                .any(|issue| issue.code == IssueCode::NoValue)
        );
    }

    /// `ParseCtx::purpose` dispatches to exactly the matching narrow entry point.
    #[test]
    fn purpose_matches_the_narrow_entry_points() {
        let with_purpose = |purpose| {
            move |text: &str| {
                parse(
                    text,
                    Some(ParseCtx {
                        purpose,
                        ..ParseCtx::default()
                    }),
                )
            }
        };
        assert_eq!(
            with_purpose(ParsePurpose::Quantity)("5 kg"),
            parse_quantity_fast("5 kg", None)
        );
        assert_eq!(
            with_purpose(ParsePurpose::Number)("1,234"),
            parse_number_fast("1,234", None)
        );
        assert_eq!(
            with_purpose(ParsePurpose::Date)("2026-05-06"),
            parse_date_fast("2026-05-06", None)
        );
        assert_eq!(
            with_purpose(ParsePurpose::Recurrence)("every monday"),
            parse_recurrence_fast("every monday", None)
        );
    }

    /// `ParsePurpose::DimensionEditor` is *not* `parse_dimensions_for_editor`.
    ///
    /// The purpose runs the editor grammar over the whole input; the function is
    /// an extractor that finds candidates inside free text and infers an
    /// expected dimension from a neighbouring label. They diverge in both
    /// directions, and the field documentation now says so.
    #[test]
    fn dimension_editor_purpose_differs_from_the_editor_extractor() {
        let with_purpose = |text: &str| {
            parse(
                text,
                Some(ParseCtx {
                    purpose: ParsePurpose::DimensionEditor,
                    ..ParseCtx::default()
                }),
            )
        };

        // A labelled dimension: the extractor reads past the label, the whole
        // string does not parse as a quantity.
        let labelled = with_purpose("幅3m");
        assert!(labelled.best.is_none());
        let extracted = parse_dimensions_for_editor("幅3m", None);
        assert_eq!(extracted.len(), 1);
        let best = extracted[0].parsed.best.as_ref().expect("a reading");
        assert_eq!(best.unit.as_deref(), Some("m"));
        assert_eq!(best.value, Some(3.0));

        // An unlabelled bare number: the purpose reads it, the extractor refuses
        // it because nothing says it is a dimension.
        let bare = with_purpose("3640");
        let best = bare.best.as_ref().expect("a reading");
        assert_eq!(best.kind, Kind::Number);
        assert_eq!(best.value, Some(3640.0));
        assert!(parse_dimensions_for_editor("3640", None).is_empty());
    }

    /// `ParseCtx::expected_dimension` really is only a hint for `parse`.
    #[test]
    fn expected_dimension_is_only_a_hint() {
        let parsed = parse(
            "5 kg",
            Some(ParseCtx {
                expected_dimension: Some(Dimension::Length),
                ..ParseCtx::default()
            }),
        );
        let best = parsed.best.expect("a canonical reading");
        assert_eq!(best.dimension, Some(Dimension::Mass));
        assert_eq!(best.unit.as_deref(), Some("kg"));
    }

    /// `ParseCtx::expect` does not constrain which grammar `parse` runs.
    #[test]
    fn expect_does_not_constrain_parsing() {
        for kind in [Kind::Date, Kind::Recurrence, Kind::Number, Kind::Range] {
            let parsed = parse(
                "5 kg",
                Some(ParseCtx {
                    expect: Some(kind),
                    ..ParseCtx::default()
                }),
            );
            let best = parsed.best.expect("a canonical reading");
            assert_eq!(best.kind, Kind::Quantity);
            assert_eq!(best.unit.as_deref(), Some("kg"));
        }
    }

    /// The two things `ParseCtx::expect` does affect: completion filtering and
    /// the millimetre alternative offered for a bare number.
    #[test]
    fn expect_filters_completions_and_adds_a_millimetre_alternative() {
        let expecting = |kind| {
            move |prefix: &str| {
                complete(
                    prefix,
                    Some(ParseCtx {
                        expect: Some(kind),
                        ..ParseCtx::default()
                    }),
                )
            }
        };

        // `Date` keeps date candidates and drops the rest.
        assert!(
            complete("ma", None)
                .iter()
                .any(|candidate| candidate.kind != CompletionKind::Date)
        );
        let dates_only = expecting(Kind::Date)("ma");
        assert!(!dates_only.is_empty());
        assert!(
            dates_only
                .iter()
                .all(|candidate| candidate.kind == CompletionKind::Date)
        );

        // `Number` and `Recurrence` drop every candidate.
        assert!(!complete("me", None).is_empty());
        assert!(expecting(Kind::Number)("me").is_empty());
        assert!(expecting(Kind::Recurrence)("me").is_empty());

        assert!(parse("42", None).alternatives.is_empty());
        let parsed = parse(
            "42",
            Some(ParseCtx {
                expect: Some(Kind::Quantity),
                ..ParseCtx::default()
            }),
        );
        assert_eq!(
            parsed
                .alternatives
                .iter()
                .map(|reading| reading.unit.as_deref())
                .collect::<Vec<_>>(),
            vec![Some("mm")]
        );
    }

    /// `ParseCtx::timezone` is reserved and currently inert.
    #[test]
    fn timezone_is_ignored_by_the_parser() {
        let with_timezone = |text: &str| {
            parse(
                text,
                Some(ParseCtx {
                    timezone: Some("Asia/Tokyo".to_owned()),
                    ..ParseCtx::default()
                }),
            )
        };
        for text in ["3pm Asia/Tokyo", "3pm", "5 kg", "2026-05-06"] {
            let hinted = with_timezone(text);
            let plain = parse(text, None);
            assert_eq!(hinted.best, plain.best, "best differs for {text}");
            assert_eq!(
                hinted.findings, plain.findings,
                "findings differ for {text}"
            );
        }
        assert_eq!(
            with_timezone("3pm")
                .best
                .expect("a canonical reading")
                .timezone,
            None
        );
    }

    /// `CurrencyRate` fields are matched exactly, so lowercase codes never hit.
    #[test]
    fn currency_rate_fields_must_hold_normalized_codes() {
        let with_rate = |rate: CurrencyRate| {
            parse(
                "USD 10 to JPY",
                Some(ParseCtx {
                    currency_rates: vec![rate],
                    ..ParseCtx::default()
                }),
            )
        };

        let lowercase = CurrencyRate {
            from: "usd".to_owned(),
            to: "jpy".to_owned(),
            factor: 150.0,
        };
        assert!(with_rate(lowercase).best.is_none());

        let uppercase = CurrencyRate {
            from: "USD".to_owned(),
            to: "JPY".to_owned(),
            factor: 150.0,
        };
        assert_eq!(
            with_rate(uppercase).best.expect("a reading").value,
            Some(1500.0)
        );

        assert_eq!(
            with_rate(CurrencyRate::new("usd", "jpy", 150.0))
                .best
                .expect("a reading")
                .value,
            Some(1500.0)
        );
    }

    /// `AcceptOptions::compounds` gates only same-dimension registry compounds.
    #[test]
    fn compounds_gates_only_registry_compounds() {
        let refused = |text: &str| {
            parse(
                text,
                Some(ParseCtx {
                    accept: AcceptOptions {
                        compounds: false,
                        ..AcceptOptions::default()
                    },
                    ..ParseCtx::default()
                }),
            )
        };

        for text in ["3 yd 2 ft", "1 hour 30 minutes"] {
            let parsed = refused(text);
            assert!(parsed.best.is_none(), "{text} should be refused");
            assert!(
                parsed
                    .findings
                    .skipped
                    .iter()
                    .any(|issue| issue.code == IssueCode::RejectedByPolicy),
                "{text} should be rejected by policy"
            );
        }

        let feet_inches = refused("5ft 11in").best.expect("feet-inches still parses");
        assert_eq!(feet_inches.unit.as_deref(), Some("m"));
        assert_close(feet_inches.value.expect("a value"), 1.8034);

        let shakkanho = refused("5尺3寸").best.expect("shakkanhō still parses");
        assert_eq!(shakkanho.unit.as_deref(), Some("m"));
        assert_close(shakkanho.value.expect("a value"), 1.606_060_606_060_606);
    }
}
