//! Adapters for presenting readings and for AI/tool call boundaries.
//!
//! [`humanize`] turns a canonical reading back into a sentence. The
//! canonicalize functions wrap parsing in an accept/reject decision suitable
//! for validating machine-supplied field values, returning an explanatory
//! message instead of a value when the input is not acceptable.

use crate::*;

/// One field to canonicalize, as submitted by a caller or a tool call.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalizeRequest {
    /// Field name, used to address the value in the resulting message.
    pub field: String,
    /// The raw text submitted for this field.
    pub text: String,
    /// Parsing context for this field, if any.
    pub ctx: Option<ParseCtx>,
}

impl CanonicalizeRequest {
    /// Builds a request for `field` from the submitted `text`.
    pub fn new(field: &str, text: &str, ctx: Option<ParseCtx>) -> Self {
        Self {
            field: field.to_owned(),
            text: text.to_owned(),
            ctx,
        }
    }
}

/// The verdict on one canonicalized field.
#[derive(Clone, Debug, PartialEq)]
pub struct CanonicalizedValue {
    /// Field name copied from the request.
    pub field: String,
    /// The raw text that was submitted.
    pub input: String,
    /// Whether the value was accepted under the request's strictness.
    pub ok: bool,
    /// The accepted reading. `None` whenever `ok` is `false`.
    pub canonical: Option<Reading>,
    /// The full parse, available whether or not the value was accepted.
    pub parsed: Parsed,
    /// Why the value was rejected. `None` whenever `ok` is `true`.
    pub message: Option<String>,
}

/// Canonicalizes a batch of submitted field values, accepting or rejecting each.
///
/// A value is accepted only if a reading was found and nothing was skipped.
/// Under [`Strictness::Confirm`] or [`Strictness::Strict`], any ambiguity or
/// approximation also rejects it — so an accepted value is one the parser did
/// not have to guess at. Rejections carry a message tagged with the
/// [`IssueCode`], plus a did-you-mean suggestion when one is available.
///
/// ```
/// use unravel_nl::{canonicalize_values, CanonicalizeRequest, ParseCtx, Strictness};
///
/// let values = canonicalize_values(&[CanonicalizeRequest::new(
///     "weight",
///     "about 20kg",
///     Some(ParseCtx {
///         strictness: Strictness::Strict,
///         ..ParseCtx::default()
///     }),
/// )]);
///
/// assert!(!values[0].ok);
/// assert!(values[0].message.as_ref().unwrap().contains("[APPROXIMATION]"));
/// ```
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

/// Canonicalizes one field and returns only the rejection message, if any.
///
/// Intended for repairing a machine-generated tool call: `None` means the value
/// was acceptable, `Some(message)` is text that can be handed back to the
/// caller explaining what to fix.
pub fn repair_tool_call_message(field: &str, text: &str, ctx: Option<ParseCtx>) -> Option<String> {
    let request = CanonicalizeRequest::new(field, text, ctx);
    canonicalize_values(&[request])
        .into_iter()
        .next()
        .and_then(|value| value.message)
}

pub(crate) fn adapter_accepts(parsed: &Parsed, ctx: Option<&ParseCtx>) -> bool {
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

pub(crate) fn adapter_message(field: &str, parsed: &Parsed) -> String {
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

/// Renders a canonical reading as a human-readable string.
///
/// Output is locale-sensitive: with [`Locale::Ja`] a length in metres is
/// rendered in shakkanhō units and a temperature as `摂氏20度`. Values that came
/// from an approximate conversion are marked as approximate rather than
/// presented as exact.
///
/// A reading that carries no usable value still renders as a word rather than
/// an empty string, so the output is always displayable: `unknown date`,
/// `unknown recurrence`, `unresolved range`, `unresolved`, and — for a value
/// that is infinite or `NaN` — `unrepresentable` in place of the number, so a
/// quantity keeps its unit and renders as `unrepresentable m`. No library entry
/// point hands back such a value: [`parse`] reports it as a loss and
/// [`complete_readings`] leaves the candidate out. It reaches [`humanize`] only
/// through a [`Reading`] the caller assembled or edited themselves.
///
/// ```
/// use unravel_nl::{humanize, parse, HumanizeCtx, Locale, ParseCtx};
///
/// let parsed = parse(
///     "5尺3寸",
///     Some(ParseCtx {
///         locale: Some(Locale::Ja),
///         ..ParseCtx::default()
///     }),
/// );
/// let best = parsed.best.expect("a canonical reading");
///
/// assert_eq!(
///     humanize(&best, Some(HumanizeCtx { locale: Some(Locale::Ja) })),
///     "5尺3寸 (approx.)"
/// );
/// ```
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

/// Flattens a reading into a labelled field list for UI and tool layers.
///
/// Only the fields the reading actually carries are emitted, so the view can be
/// rendered generically without checking each `Option`. The view's summary is
/// the locale-independent [`humanize`] rendering.
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

/// Flattens a whole parse, including finding counts, into a field list.
///
/// Unlike [`describe_reading`], this reports the outcome as well as the value:
/// the `ok` field is `true` only when a reading was found and nothing was
/// skipped, and the finding counts show how much the parser had to guess.
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

pub(crate) fn push_resource_field(fields: &mut Vec<ResourceField>, name: &str, value: &str) {
    fields.push(ResourceField {
        name: name.to_owned(),
        value: value.to_owned(),
    });
}

pub(crate) fn humanize_japanese_length(meters: f64) -> String {
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

pub(crate) fn humanize_japanese_area(area: f64) -> String {
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

/// Rendered in place of a value that has no finite decimal representation.
///
/// `format_number` never emits `inf`, `-inf`, or `NaN`: those are not numbers a
/// caller can act on, and they are not valid JSON numbers either. Parsing
/// rejects non-finite readings outright (see `finalize_parsed`) and the
/// completion fan-out drops non-finite candidates (see `push_unit_fanout`), so
/// this token surfaces only for a [`Reading`] the caller assembled or edited
/// themselves — including one built from a [`CustomUnit`] factor applied
/// outside the library.
pub(crate) const NON_FINITE_TEXT: &str = "unrepresentable";

pub(crate) fn format_number(value: f64) -> String {
    if !value.is_finite() {
        return NON_FINITE_TEXT.to_owned();
    }
    // Rounding to six decimals overflows to infinity above ~1.7e302, and above
    // 2^53 there is no fractional part left to round, so skip it for magnitudes
    // that cannot carry one.
    let rounded = if value.abs() < 1e15 {
        (value * 1_000_000.0).round() / 1_000_000.0
    } else {
        value
    };
    if rounded == 0.0 {
        // Normalizes -0.0, which would otherwise render as "-0".
        return "0".to_owned();
    }
    if (rounded - rounded.trunc()).abs() < f64::EPSILON {
        // `f64` Display renders integral values without a fractional part and
        // without an exponent, and unlike `as i64` it does not saturate.
        format!("{rounded}")
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn formats_large_finite_values_without_saturating() {
        // `value as i64` used to saturate here and render i64::MAX.
        let best = parse("100000000000000000000", None).best.expect("number");
        assert_eq!(best.value, Some(1e20));
        assert_eq!(humanize(&best, None), "100000000000000000000");
        assert_eq!(
            describe_reading(&best).summary,
            "100000000000000000000".to_owned()
        );

        assert_eq!(format_number(1e20), "100000000000000000000");
        assert_eq!(format_number(-1e20), "-100000000000000000000");
        assert_eq!(format_number(1e300), format!("1{}", "0".repeat(300)));
        // Above 2^53 the shortest round-tripping decimal is used, which maps
        // back to exactly this f64 — unlike the old i64 saturation.
        let big = 9_223_372_036_854_775_807.0_f64 * 4.0;
        assert_eq!(format_number(big), "36893488147419103000");
        assert_eq!("36893488147419103000".parse::<f64>().unwrap(), big);
    }

    #[test]
    fn never_formats_non_finite_values_as_inf_or_nan() {
        for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            let text = format_number(value);
            assert_eq!(text, NON_FINITE_TEXT, "{value}");
            assert!(!text.contains("inf") && !text.contains("NaN"), "{value}");
        }

        // A caller can still hand `humanize` a non-finite reading directly.
        let reading = Reading::number(f64::INFINITY, 0.9);
        assert_eq!(humanize(&reading, None), NON_FINITE_TEXT);
    }

    #[test]
    fn formats_signed_zero_as_plain_zero() {
        assert_eq!(format_number(-0.0), "0");
        assert_eq!(format_number(0.0), "0");
    }
}
