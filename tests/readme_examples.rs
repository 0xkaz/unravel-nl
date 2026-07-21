//! Keeps the README and README.ja.md examples honest.

use unravel_nl::{HumanizeCtx, Locale, ParseCtx, humanize, parse, parse_dimensions_for_editor};

#[test]
fn readme_shakkanho_example() {
    let parsed = parse(
        "5尺3寸",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );
    let best = parsed.best.expect("a canonical reading");
    assert_eq!(best.unit.as_deref(), Some("m"));
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
fn readme_editor_dimensions_example() {
    let matches = parse_dimensions_for_editor(
        "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            ..ParseCtx::default()
        }),
    );
    assert_eq!(matches.len(), 4);
}

#[cfg(feature = "dates-jiff")]
#[test]
fn readme_relative_date_example() {
    use unravel_nl::Date;

    let parsed = parse(
        "来週金曜日",
        Some(ParseCtx {
            locale: Some(Locale::Ja),
            reference_date: Date::new(2026, 7, 19),
            ..ParseCtx::default()
        }),
    );
    assert_eq!(parsed.best.unwrap().date.as_deref(), Some("2026-07-24"));
}
