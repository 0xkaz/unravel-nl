# unravel-nl

`unravel-nl` is a deterministic Rust library for turning informal or ambiguous
natural-language quantities into canonical readings, plus human-readable output.

The first slice focuses on:

- Japanese customary length and area input such as `5尺3寸`, `6帖`, and `1坪`
- Square-meter input such as `延床100㎡`
- Ranges such as `100-120㎡`, `2〜3日`, and `between 5 and 10 kg`
- Grouped plain numbers such as `1,234`
- Locale number formats such as `1.234,56 kg`, `1 234,56 m`,
  `1,23,456 kg`, `1万2345`, and `3.5万円`
- Unicode and Japanese input normalization for full-width numbers and
  compatibility units such as `５尺３寸`, `１．５ｍ`, `２㎞`, and `百二十平米`
- Metric and mass examples such as `180cm`, `1m80`, and `1,5 kg`
- Mixed imperial height input such as `5ft 11`
- Locale-sensitive cup volumes with explicit alternatives
- An expanding unit registry for length, mass, area, duration, volume, speed,
  data, data-rate, flow-rate, pressure, power, electrical, lighting, and
  radiation aliases
- Mixed same-dimension compound units such as `3 yd 2 ft` and `4 stone 6 lb`
- Registry-backed unit typo correction such as `5 meterz`
- Forgiving, confirm, and strict parse modes for correction policy
- Compact and ISO-style durations such as `1h30`, `2d4h`, and `PT1H30M`
- Clock times and slots such as `3pm`, `14:30`, and `3pm-4pm`
- Recurrence readings such as `every monday`, `every 2 weeks`,
  `every other monday`, `monthly on the second monday`, `毎週月曜日`,
  `毎月第2月曜日`, `every third business day`, and `毎日`, normalized to
  RRULE-style strings
- Approximate, tolerance, and bounded input such as `about 20C`, `約20kg`,
  `10 ± 0.5 mm`, `a few minutes`, `under 10 minutes`, `10mm以下`, and
  temperature phrases like `it's hot`
- Golden corpus and round-trip tests for representative canonical readings,
  including common examples from public natural-language parsing behavior
- Locale alias slices for en-GB, Spanish, French, Portuguese, Chinese, and
  Japanese inputs such as `05/06/2026`, `1,5 litros`, `2 mètres carrés`,
  `10 quilômetros`, `明天`, `下周五`, `4畳半`, and `1間半`
- Currency amounts such as `USD 12.34`, `12 bucks`, `99 pence`, `¥1,234`, and
  ambiguous `$12`
- Temperature input such as `20°C`, `68 F`, `293.15 K`, and `摂氏20度`
- Typed technical quantities such as `500 GB`, `20 MB/s`, `5 gpm`, `500 mAh`,
  `5 uM`, `10 Nm`, `500 lux`, `20 mSv`, `5 MBq`, `10 inH₂O`, and `1 kgf/cm²`
- Relative dates such as `next friday` and `in 3 days` with the `dates-jiff`
  feature
- Static parse input, parsed output, and MCP tool schemas for AI/tool adapters
- Split API entry points: broad `parse()`, narrower `parse_quantity_fast()`,
  `parse_number_fast()`, `parse_date_fast()`, `parse_recurrence_fast()`,
  multi-value `parse_all()`, editor-focused `parse_dimensions_for_editor()`,
  and reading-level `complete_readings()`
- Multi-value extraction with byte spans via `parse_all()`
- Explicit `NumberFormat` and `AcceptOptions` policies for callers that need
  deterministic punctuation and grammar-shape control
- Core completion candidates for unit, date, time, currency, temperature, and
  custom-unit adapter layers
- Custom unit kind metadata, custom fuzzy vocabulary profiles, and
  `describe_*` resource views for UI/tool adapters
- Feature-gated WASM exports for browser or Node package adapters, including
  single-value JSON parsing and span-preserving multi-value extraction
- Browser adapter TypeScript definitions for UI integration
- No-Silent-Loss findings for skipped, ambiguous, and approximate readings
- A normalized parser dispatch path and exact-first unit alias lookup to keep
  no-match and typo-heavy inputs bounded as the locale catalog grows

The default compute path has no I/O and no runtime dependencies. Calendar
arithmetic is available behind the optional `dates-jiff` feature.

## Example

```rust
use unravel_nl::{parse, humanize, HumanizeCtx, Locale, ParseCtx};

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
    humanize(&best, Some(HumanizeCtx { locale: Some(Locale::Ja) })),
    "5尺3寸 (approx.)"
);
```

## API Entry Points

Use `parse()` as the broad compatibility entry point when the input can be a
quantity, date, range, recurrence, conversion, or plain number. Use narrower
entry points when the UI already knows the field type:

```rust
use unravel_nl::{parse_quantity_fast, Dimension, ParseCtx};

let parsed = parse_quantity_fast(
    "1,234 kg",
    Some(ParseCtx {
        expected_dimension: Some(Dimension::Mass),
        ..ParseCtx::default()
    }),
);

assert_eq!(parsed.best.unwrap().unit.as_deref(), Some("kg"));
```

Available specialized entries are `parse_quantity_fast()`,
`parse_number_fast()`, `parse_date_fast()`, and `parse_recurrence_fast()`.
Use `ParseCtx::purpose` when a single `parse()` call should avoid broad
grammar dispatch. `parse_all()` scans sentences for multiple values,
`parse_dimensions_for_editor()` scans only building dimensions and unitless
dimension fields, and `complete_readings()` returns ranked canonical readings
for completion UIs.

## Multi-Value Extraction

```rust
use unravel_nl::{parse_all, Dimension, Locale, ParseCtx};

let matches = parse_all(
    "延床100㎡、敷地面積120㎡、高さ3.5m",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    }),
);

assert_eq!(matches.len(), 3);
assert_eq!(matches[0].text, "延床100㎡");
assert_eq!(matches[0].parsed.best.as_ref().unwrap().dimension, Some(Dimension::Area));
assert_eq!(matches[2].text, "3.5m");
```

For editor fields that only accept dimensions, use the dedicated scanner. It
extracts length and area values, keeps Japanese building units, and avoids
currency/date/general grammar:

```rust
use unravel_nl::{parse_dimensions_for_editor, Dimension, Locale, ParseCtx};

let matches = parse_dimensions_for_editor(
    "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        ..ParseCtx::default()
    }),
);

assert_eq!(matches.len(), 4);
assert_eq!(matches[0].parsed.best.as_ref().unwrap().dimension, Some(Dimension::Length));
assert_eq!(matches[2].parsed.best.as_ref().unwrap().dimension, Some(Dimension::Area));
```

The Rust scanner preserves byte spans and uses a token-window dispatch path for
dimension-like substrings. WASM JSON includes both byte and character spans,
and browser adapters add JavaScript `codeUnitStart` / `codeUnitEnd` fields for
UI highlighting.

```rust
use unravel_nl::{parse_all, Dimension, Locale, ParseCtx};

let matches = parse_all(
    "3m×4m のLDK",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        expected_dimension: Some(Dimension::Length),
        ..ParseCtx::default()
    }),
);

assert_eq!(matches[0].text, "3m");
assert_eq!(matches[0].start, 0);
assert_eq!(matches[1].text, "4m");
assert_eq!(matches[1].start, 4);
```

## Date Parsing

```rust
use unravel_nl::{parse, Date, Locale, ParseCtx};

let parsed = parse(
    "next friday",
    Some(ParseCtx {
        locale: Some(Locale::En),
        reference_date: Date::new(2026, 7, 19),
        ..ParseCtx::default()
    }),
);

assert_eq!(parsed.best.unwrap().date.as_deref(), Some("2026-07-24"));
```

Enable date arithmetic with:

```toml
unravel-nl = { version = "0.1", features = ["dates-jiff"] }
```

Japanese relative dates are supported with the same feature:

```rust
use unravel_nl::{parse, Date, Locale, ParseCtx};

let parsed = parse(
    "来週金曜日",
    Some(ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        ..ParseCtx::default()
    }),
);

assert_eq!(parsed.best.unwrap().date.as_deref(), Some("2026-07-24"));
```

The core parser never reads the host system clock or timezone. Relative dates
must be given an explicit `reference_date`; adapter layers can pass a `timezone`
hint, but the core does not derive behavior from the Rust process environment.
Timezone-qualified wall-clock strings with explicit offsets or known fixed
abbreviations, such as `3pm EST` or `9:30 JST`, are normalized to UTC seconds.
With the `timezones-jiff` feature and an explicit `reference_date`, IANA-zone
conversion such as `3pm Europe/Paris` uses bundled timezone data and remains
independent of the Rust host environment. IANA-zone input without an explicit
date fails loudly.

Simple recurring expressions are canonicalized as RRULE-style strings:

```rust
use unravel_nl::{parse, Kind};

let parsed = parse("every monday", None);
let best = parsed.best.unwrap();

assert_eq!(best.kind, Kind::Recurrence);
assert_eq!(best.recurrence.as_deref(), Some("FREQ=WEEKLY;BYDAY=MO"));
```

UI adapters can turn parser findings into stable severity and rank metadata:

```rust
use unravel_nl::{parse, ranked_findings, IssueSeverity};

let parsed = parse("3pm Europe/Paris", None);
let issues = ranked_findings(&parsed);

assert_eq!(issues[0].severity, IssueSeverity::Error);
assert_eq!(issues[0].rank, 90);
```

Browser-facing adapters live in `web/unravel-adapters.js`. They are dependency
free ESM helpers for DOM inputs, span-preserving `parseAllForUi()`,
field-list `canonicalizeFieldsForUi()`, canonicalizer-result normalization,
React integration by injection, and a custom element wrapper; parser functions
are injected so the same code can sit on top of a WASM bundle or a server
bridge. TypeScript definitions live in `web/unravel-adapters.d.ts`. The React
adapter is covered by an actual React server-render runtime smoke test under
`tests/react_adapter_runtime.mjs`.

## WASM

```sh
wasm-pack build --target web --out-dir pkg -- --features wasm
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm
node tests/wasm_node_smoke.mjs
```

The WASM package exports `parse_json*`, `parse_all_json*`, and
`parse_dimensions_for_editor_json*` functions, with locale-only and minimal
context variants for `expected_dimension` and `strictness`. The browser smoke
page is `tests/wasm_browser_e2e.html`; serve the repository root and open
`/tests/wasm_browser_e2e.html` after generating `pkg/`. Browser-target Method A
artifacts can be assembled from `pkg/` plus `web/unravel-adapters.*` and
checksummed before vendoring.

## Unit Registry And Strictness

```rust
use unravel_nl::{parse, unit_definitions, IssueCode, ParseCtx, Strictness};

assert!(unit_definitions().iter().any(|unit| unit.id == "ft"));

let forgiving = parse("5 meterz", None);
assert_eq!(forgiving.best.unwrap().unit.as_deref(), Some("m"));
assert_eq!(
    forgiving.findings.ambiguities[0].code,
    IssueCode::TypoCorrected
);

let confirm = parse(
    "5 meterz",
    Some(ParseCtx {
        strictness: Strictness::Confirm,
        ..ParseCtx::default()
    }),
);
assert!(confirm.best.is_none());
assert_eq!(confirm.suggestions[0].to, "m");
```

Callers can also add deterministic custom unit aliases at parse time:

```rust
use unravel_nl::{parse, CustomUnit, Dimension, ParseCtx};

let parsed = parse(
    "3 smoots",
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

assert_eq!(parsed.best.unwrap().unit.as_deref(), Some("m"));
```

Custom units can also carry an application-facing kind label:

```rust
use unravel_nl::{parse, CustomUnit, Dimension, ParseCtx};

let parsed = parse(
    "3 cases",
    Some(ParseCtx {
        custom_units: vec![CustomUnit::new(
            "case",
            "item",
            &["case", "cases"],
            Dimension::Volume,
            24.0,
        ).kind("package_count")],
        ..ParseCtx::default()
    }),
);

assert_eq!(parsed.best.unwrap().custom_kind.as_deref(), Some("package_count"));
```

## Completions

```rust
use unravel_nl::{complete, complete_readings, CompletionKind, Dimension, ParseCtx};

let completions = complete("10 met", None);

assert_eq!(completions[0].value, "meter");
assert_eq!(completions[0].canonical.as_deref(), Some("m"));
assert_eq!(completions[0].kind, CompletionKind::Unit);

let readings = complete_readings(
    "10",
    Some(ParseCtx {
        expected_dimension: Some(Dimension::Mass),
        ..ParseCtx::default()
    }),
);

assert!(readings.iter().any(|item| item.text == "10 kg"));
```

## Temperature

Temperature readings are normalized to Celsius:

```rust
use unravel_nl::{humanize, parse};

let parsed = parse("68 F", None);
let best = parsed.best.expect("temperature");

assert_eq!(best.unit.as_deref(), Some("C"));
assert_eq!(humanize(&best, None), "20 °C");
```

## Approximate And Fuzzy Input

```rust
use unravel_nl::{parse, Dimension, FuzzyProfile, FuzzyTerm, ParseCtx};

let tolerance = parse("10 ± 0.5 mm", None);
assert!(tolerance.best.unwrap().range.is_some());

let bounded = parse("10mm以下", None);
assert!(bounded.best.unwrap().range.is_some());

let hot = parse(
    "it's hot",
    Some(ParseCtx {
        expected_dimension: Some(Dimension::Temperature),
        ..ParseCtx::default()
    }),
);

assert!(hot.best.unwrap().range.is_some());

let custom = parse(
    "heavy",
    Some(ParseCtx {
        expected_dimension: Some(Dimension::Mass),
        fuzzy_profiles: vec![FuzzyProfile::new(
            "parcels",
            Dimension::Mass,
            "kg",
            &[FuzzyTerm::new("heavy", 20.0, 70.0)],
        )],
        ..ParseCtx::default()
    }),
);

assert!(custom.best.unwrap().range.is_some());
```

Callers that must reject broad grammar shapes can use `AcceptOptions`:

```rust
use unravel_nl::{parse, AcceptOptions, ParseCtx};

let parsed = parse(
    "between 5 and 10 kg",
    Some(ParseCtx {
        accept: AcceptOptions {
            ranges: false,
            ..AcceptOptions::default()
        },
        ..ParseCtx::default()
    }),
);

assert!(parsed.best.is_none());
assert_eq!(parsed.alternatives.len(), 1);
```

Numeric punctuation can be pinned with `NumberFormat` when locale inference is
too permissive:

```rust
use unravel_nl::{parse, NumberFormat, ParseCtx};

let parsed = parse(
    "1,234 kg",
    Some(ParseCtx {
        number_format: NumberFormat::CommaDecimal,
        ..ParseCtx::default()
    }),
);

assert_eq!(parsed.best.unwrap().value, Some(1.234));
```

## Currency Rates

Currency conversion only runs when the caller supplies an explicit rate:

```rust
use unravel_nl::{parse, CurrencyRate, ParseCtx};

let parsed = parse(
    "USD 10 to JPY",
    Some(ParseCtx {
        currency_rates: vec![CurrencyRate::new("USD", "JPY", 150.0)],
        ..ParseCtx::default()
    }),
);

let best = parsed.best.expect("converted amount");
assert_eq!(best.unit.as_deref(), Some("JPY"));
assert_eq!(best.value, Some(1500.0));
```

## Schemas

```rust
use unravel_nl::{
    canonicalize_values, contract_version, mcp_tool_schema_json,
    parse_input_schema_json, parsed_output_schema_json, CanonicalizeRequest,
    ParseCtx, Strictness,
};

assert_eq!(contract_version(), "unravel-nl.parse.v1");
assert!(parse_input_schema_json().contains("\"text\""));
assert!(parsed_output_schema_json().contains("\"findings\""));
assert!(mcp_tool_schema_json().contains("unravel_nl_parse"));

let values = canonicalize_values(&[CanonicalizeRequest::new(
    "weight",
    "about 20kg",
    Some(ParseCtx {
        strictness: Strictness::Strict,
        ..ParseCtx::default()
    }),
)]);

assert!(!values[0].ok);
assert!(values[0].message.as_ref().unwrap().contains("[APPROXIMATION]"));
```

## Attribution

The public API direction is inspired by `pascalorg/lingo` (MIT). This crate is
an independent Rust implementation and does not copy source code from that
project.
