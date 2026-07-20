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
/// rendered in shakkanhō units and a temperature as `摂氏20度`.
///
/// The rendered string is **not** a reliable indicator of whether a value is
/// approximate. The ` (approx.)` marker is produced by exactly two renderings —
/// the [`Locale::Ja`] shakkanhō length (`5尺3寸 (approx.)`) and the
/// [`Locale::Ja`] tatami or tsubo area (`6帖 (approx.)`) — and by nothing else.
/// Every other rendering prints the number bare however the reading was
/// obtained: `parse("5尺3寸", ..)` yields a reading with
/// `approximate: Some(true)`, yet it humanizes to `1.606061 m` under both no
/// locale and [`Locale::En`]; `1.5 cups` humanizes to `0.354882 L` and
/// `about 20kg` to `20 kg` even under [`Locale::Ja`]. Callers that need to know
/// must read [`Reading::approximate`] (or [`Findings::approximations`]) rather
/// than inspect this string. The same applies to [`ResourceView::summary`],
/// which is this function called with no locale and so never carries the
/// marker at all.
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
    let locale = ctx.as_ref().and_then(|ctx| ctx.locale.as_ref());
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
        // The endpoints are rendered with the caller's context, not with `None`:
        // a range is still the same reading it would be on its own, so dropping
        // the context here would have silently turned off locale-sensitive
        // rendering exactly when the reading happens to be a range.
        (_, Kind::Range, _, _) => value.range.as_ref().map_or_else(
            || "unresolved range".to_owned(),
            |range| {
                format!(
                    "{} to {}",
                    humanize(&range.from, ctx.clone()),
                    humanize(&range.to, ctx.clone())
                )
            },
        ),
        _ => "unresolved".to_owned(),
    }
}

/// Flattens a reading into a labelled field list for UI and tool layers.
///
/// Only the fields the reading actually carries are emitted, so the view can be
/// rendered generically without checking each `Option`. A range reading carries
/// its payload in its endpoints, which are flattened into the same list under
/// dotted names — `range.from.value`, `range.to.unit`, and so on — so no part of
/// the reading is left out of the machine-readable view.
///
/// `value` is written at full precision, as the shortest decimal that reads
/// back as the same `f64`: this list is what a tool layer acts on, so it must
/// not lose data the way a display rendering may. The six-decimal rounding
/// belongs to [`ResourceView::summary`], which is the human-readable
/// [`humanize`] line and is locale-independent here.
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
    push_reading_fields(&mut fields, "", reading);
    let summary = humanize(reading, None);
    ResourceView {
        object,
        summary,
        fields,
    }
}

/// Appends one reading's fields to `fields`, each name carrying `prefix`.
///
/// The endpoints of a range are appended by the same function, under
/// `range.from.` and `range.to.`, so an endpoint contributes exactly the fields
/// a top-level reading does. Nesting is as deep as the reading the caller built:
/// [`parse`] never nests a range inside a range, and the endpoints of the ranges
/// it does build carry no endpoints of their own.
fn push_reading_fields(fields: &mut Vec<ResourceField>, prefix: &str, reading: &Reading) {
    let named = |name: &str| format!("{prefix}{name}");
    push_resource_field(fields, &named("kind"), kind_str(reading.kind));
    if let Some(custom_kind) = &reading.custom_kind {
        push_resource_field(fields, &named("custom_kind"), custom_kind);
    }
    if let Some(value) = reading.value {
        push_resource_field(fields, &named("value"), &format_number_exact(value));
    }
    if let Some(unit) = &reading.unit {
        push_resource_field(fields, &named("unit"), unit);
    }
    if let Some(dimension) = reading.dimension {
        push_resource_field(fields, &named("dimension"), dimension.as_str());
    }
    if let Some(date) = &reading.date {
        push_resource_field(fields, &named("date"), date);
    }
    if let Some(recurrence) = &reading.recurrence {
        push_resource_field(fields, &named("recurrence"), recurrence);
    }
    if let Some(timezone) = &reading.timezone {
        push_resource_field(fields, &named("timezone"), timezone);
    }
    if let Some(provenance) = reading.provenance {
        push_resource_field(fields, &named("provenance"), provenance.as_str());
    }
    if let Some(approximate) = reading.approximate {
        push_resource_field(
            fields,
            &named("approximate"),
            if approximate { "true" } else { "false" },
        );
    }
    if let Some(confidence) = reading.confidence {
        push_resource_field(fields, &named("confidence"), &format_number(confidence));
    }
    if let Some(range) = &reading.range {
        push_reading_fields(fields, &named("range.from."), &range.from);
        push_reading_fields(fields, &named("range.to."), &range.to);
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

/// Renders a number for a machine consumer, without display rounding.
///
/// `format_number` rounds to six decimals for a human reader, which collapses
/// `0.0000001` to `0` and truncates `0.473176473` to `0.473176`. A field list or
/// a JSON envelope is acted on rather than read, so it takes this rendering
/// instead: `f64`'s `Display` writes the shortest decimal that round-trips back
/// to the same `f64`, and — unlike `Debug` — never uses exponent notation, so
/// every finite value comes out as a plain decimal number.
pub(crate) fn format_number_exact(value: f64) -> String {
    if !value.is_finite() {
        return NON_FINITE_TEXT.to_owned();
    }
    if value == 0.0 {
        // Normalizes -0.0, which would otherwise render as "-0".
        return "0".to_owned();
    }
    value.to_string()
}

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

    /// The `(approx.)` marker is a property of two Japanese renderings, not of
    /// the reading: an approximate reading renders bare everywhere else, so a
    /// caller must read `Reading::approximate` instead of the string.
    #[test]
    fn approximate_marker_only_appears_in_japanese_length_and_area() {
        let ja = |text: &str| {
            parse(
                text,
                Some(ParseCtx {
                    locale: Some(Locale::Ja),
                    ..ParseCtx::default()
                }),
            )
            .best
            .expect("a reading")
        };

        let shaku = ja("5尺3寸");
        assert_eq!(shaku.approximate, Some(true));
        assert_eq!(humanize(&shaku, None), "1.606061 m");
        assert_eq!(
            humanize(
                &shaku,
                Some(HumanizeCtx {
                    locale: Some(Locale::En)
                })
            ),
            "1.606061 m"
        );
        assert_eq!(describe_reading(&shaku).summary, "1.606061 m");

        // The two renderings that do mark it.
        assert_eq!(
            humanize(
                &shaku,
                Some(HumanizeCtx {
                    locale: Some(Locale::Ja)
                })
            ),
            "5尺3寸 (approx.)"
        );
        let tatami = ja("6帖");
        assert_eq!(
            humanize(
                &tatami,
                Some(HumanizeCtx {
                    locale: Some(Locale::Ja)
                })
            ),
            "6帖 (approx.)"
        );

        // Approximate readings that are not Japanese length or area render bare
        // even under `Locale::Ja`.
        for (text, rendered) in [("1.5 cups", "0.354882 L"), ("about 20kg", "20 kg")] {
            let reading = parse(text, None).best.expect("a reading");
            assert_eq!(reading.approximate, Some(true), "{text}");
            assert_eq!(
                humanize(
                    &reading,
                    Some(HumanizeCtx {
                        locale: Some(Locale::Ja)
                    })
                ),
                rendered,
                "{text}"
            );
        }
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

    /// The field list is the machine-readable side of this adapter, so it must
    /// not lose data to the six-decimal display rounding the way the summary
    /// does: `0.0000001 m` used to describe its value as `0`, and `2 cups` as
    /// `0.473176` for a reading holding `0.473176473`.
    #[test]
    fn describes_values_at_full_precision() {
        let tiny = parse("0.0000001 m", None).best.expect("quantity");
        let view = describe_reading(&tiny);
        assert_eq!(field(&view, "value").as_deref(), Some("0.0000001"));
        assert_eq!(
            field(&view, "value")
                .expect("value")
                .parse::<f64>()
                .expect("round trip"),
            tiny.value.expect("value")
        );
        // The summary stays the human-readable `humanize` line.
        assert_eq!(view.summary, "0 m");

        let cups = parse("2 cups", None).best.expect("quantity");
        let view = describe_reading(&cups);
        assert_eq!(field(&view, "value").as_deref(), Some("0.473176473"));
        assert_eq!(view.summary, "0.473176 L");

        // No finite value may reach the field list as an exponent form.
        for value in [f64::MIN_POSITIVE, 5e-324, f64::MAX, -f64::MAX, 1e20, -0.0] {
            let text = format_number_exact(value);
            assert!(
                !text.contains('e') && !text.contains('E'),
                "{value}: {text}"
            );
            assert_eq!(text.parse::<f64>().expect(&text), value, "{value}");
        }
        assert_eq!(format_number_exact(-0.0), "0");
        assert_eq!(format_number_exact(f64::NAN), NON_FINITE_TEXT);
        assert_eq!(format_number_exact(f64::INFINITY), NON_FINITE_TEXT);
    }

    /// A range reading used to describe itself as `[kind, approximate,
    /// confidence]`: both endpoints, and with them the whole payload, were
    /// missing from the view.
    #[test]
    fn describes_both_range_endpoints() {
        let range = parse("100-120㎡", None).best.expect("range");
        let view = describe_reading(&range);
        assert_eq!(view.object, "unravel.range");
        assert_eq!(field(&view, "range.from.value").as_deref(), Some("100"));
        assert_eq!(field(&view, "range.from.unit").as_deref(), Some("m2"));
        assert_eq!(
            field(&view, "range.from.dimension").as_deref(),
            Some("area")
        );
        assert_eq!(field(&view, "range.to.value").as_deref(), Some("120"));
        assert_eq!(field(&view, "range.to.unit").as_deref(), Some("m2"));
        assert_eq!(field(&view, "range.from.kind").as_deref(), Some("quantity"));
        assert_eq!(field(&view, "range.to.kind").as_deref(), Some("quantity"));
        assert_eq!(view.summary, "100 m2 to 120 m2");
    }

    /// `humanize` recursed into the endpoints with `None`, so a range lost the
    /// locale-sensitive rendering a single reading kept.
    #[test]
    fn humanizes_range_endpoints_in_the_callers_locale() {
        let ja = || {
            Some(ParseCtx {
                locale: Some(Locale::Ja),
                ..ParseCtx::default()
            })
        };
        let ctx = Some(HumanizeCtx {
            locale: Some(Locale::Ja),
        });

        let single = parse("3.3㎡", ja()).best.expect("area");
        assert_eq!(humanize(&single, ctx.clone()), "1坪 (approx.)");

        let range = parse("3.3㎡〜6.6㎡", ja()).best.expect("range");
        assert_eq!(
            humanize(&range, ctx.clone()),
            "1坪 (approx.) to 2坪 (approx.)"
        );
        // Without a locale the endpoints still render locale-independently.
        assert_eq!(humanize(&range, None), "3.3 m2 to 6.6 m2");
    }

    fn field(view: &ResourceView, name: &str) -> Option<String> {
        view.fields
            .iter()
            .find(|field| field.name == name)
            .map(|field| field.value.clone())
    }

    #[test]
    fn formats_signed_zero_as_plain_zero() {
        assert_eq!(format_number(-0.0), "0");
        assert_eq!(format_number(0.0), "0");
    }
}
