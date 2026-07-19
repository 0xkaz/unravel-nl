# unravel-nl

`unravel-nl` is a deterministic Rust library for turning informal or ambiguous
natural-language quantities into canonical readings, plus human-readable output.

The first slice focuses on:

- Japanese customary length and area input such as `5尺3寸`, `6帖`, and `1坪`
- Square-meter input such as `延床100㎡`
- Ranges such as `100-120㎡`, `2〜3日`, and `between 5 and 10 kg`
- Grouped plain numbers such as `1,234`
- Metric and mass examples such as `180cm`, `1m80`, and `1,5 kg`
- Mixed imperial height input such as `5ft 11`
- Locale-sensitive cup volumes with explicit alternatives
- An expanding unit registry for length, mass, area, duration, volume, speed,
  data, data-rate, flow-rate, pressure, power, electrical, lighting, and
  radiation aliases
- Registry-backed unit typo correction such as `5 meterz`
- Forgiving, confirm, and strict parse modes for correction policy
- Compact and ISO-style durations such as `1h30`, `2d4h`, and `PT1H30M`
- Clock times and slots such as `3pm`, `14:30`, and `3pm-4pm`
- Currency amounts such as `USD 12.34`, `12 bucks`, `99 pence`, `¥1,234`, and
  ambiguous `$12`
- Temperature input such as `20°C`, `68 F`, `293.15 K`, and `摂氏20度`
- Typed technical quantities such as `500 GB`, `20 MB/s`, `5 gpm`, `500 mAh`,
  `5 uM`, `10 Nm`, `500 lux`, `20 mSv`, `5 MBq`, `10 inH₂O`, and `1 kgf/cm²`
- Relative dates such as `next friday` and `in 3 days` with the `dates-jiff`
  feature
- Static parse input, parsed output, and MCP tool schemas for AI/tool adapters
- Core completion candidates for unit, date, time, currency, temperature, and
  custom-unit adapter layers
- No-Silent-Loss findings for skipped, ambiguous, and approximate readings

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

## Completions

```rust
use unravel_nl::{complete, CompletionKind};

let completions = complete("10 met", None);

assert_eq!(completions[0].value, "meter");
assert_eq!(completions[0].canonical.as_deref(), Some("m"));
assert_eq!(completions[0].kind, CompletionKind::Unit);
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
    contract_version, mcp_tool_schema_json, parse_input_schema_json,
    parsed_output_schema_json,
};

assert_eq!(contract_version(), "unravel-nl.parse.v1");
assert!(parse_input_schema_json().contains("\"text\""));
assert!(parsed_output_schema_json().contains("\"findings\""));
assert!(mcp_tool_schema_json().contains("unravel_nl_parse"));
```

## Benchmark

Run the local parser benchmark with:

```sh
cargo run --release --example bench
cargo run --release --all-features --example bench
```

Pass an iteration count as the first argument, for example
`cargo run --release --example bench -- 1000000`.

## Attribution

The public API direction is inspired by `pascalorg/lingo` (MIT). This crate is
an independent Rust implementation and does not copy source code from that
project.
