use crate::*;

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

pub(crate) fn format_number(value: f64) -> String {
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
}
