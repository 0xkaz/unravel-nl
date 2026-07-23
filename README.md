# unravel-nl

`unravel-nl` is a deterministic Rust library for turning informal or ambiguous
natural-language quantities into canonical readings, plus human-readable output.

Japanese documentation: [README.ja.md](README.ja.md)

## Guarantees

- **Deterministic.** The same input and context always produce the same result.
  No randomness, no models, no host clock, no locale environment.
- **No panic.** The public API is written never to panic; input it cannot read
  comes back as a finding, not as an unwind.
- **No silent loss.** Anything skipped, ambiguous, or approximate is reported in
  `findings` instead of being quietly dropped.
- **No forced choice.** When a fragment has several plausible readings, the
  competing readings are returned in `alternatives` rather than the parser
  committing to one.
- **No I/O and no runtime dependencies** on the default compute path.

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
- Approximate, tolerance, and bounded input such as `about 20C`, `約20kg`,
  `10 ± 0.5 mm`, `a few minutes`, `under 10 minutes`, `10mm以下`, and
  temperature phrases like `it's hot`
- Golden corpus and round-trip tests for every maintained canonical reading,
  including common examples from public natural-language parsing behavior
- Locale alias slices for en-GB, Spanish, French, Portuguese, Chinese, and
  Japanese inputs such as `1,5 litros`, `2 mètres carrés`, `10 quilômetros`,
  `4畳半`, and `1間半`; the date forms among them (`05/06/2026`, `明天`,
  `下周五`) need the `dates-jiff` feature
- Currency amounts such as `USD 12.34`, `12 bucks`, `99 pence`, `¥1,234`, and
  ambiguous `$12`
- Temperature input such as `20°C`, `68 F`, `293.15 K`, and `摂氏20度`
- Typed technical quantities such as `500 GB`, `20 MB/s`, `5 gpm`, `500 mAh`,
  `5 uM`, `10 Nm`, `500 lux`, `20 mSv`, `5 MBq`, `10 inH₂O`, and `1 kgf/cm²`
- Relative dates such as `next friday` and `in 3 days` with the `dates-jiff`
  feature
- Static parse input, parsed output, and MCP tool schemas for AI/tool adapters
- One configured `Parser::parse()` path, with `ParsePurpose` selecting a
  quantity, number, or date grammar without adding another public entry point
- Building-dimension extraction with byte spans via
  `Parser::parse_dimensions_for_editor()`
- Explicit `NumberFormat` and `AcceptOptions` policies for callers that need
  deterministic punctuation and grammar-shape control
- Core completion candidates for unit, date, time, currency, temperature, and
  custom-unit adapter layers
- Custom unit kind metadata, custom fuzzy vocabulary profiles, and
  `describe_*` resource views for UI/tool adapters
- Feature-gated WASM exports for browser or Node package adapters, including
  single-value JSON parsing and span-preserving dimension extraction
- Browser adapter TypeScript definitions for UI integration
- No-Silent-Loss findings for skipped, ambiguous, and approximate readings
- A normalized parser dispatch path, exact-first unit alias lookup, and a
  first-byte index over the unit registry, so typo-heavy and no-match inputs do
  not walk the whole catalog
- A configured `Parser` instance that limits grammar dispatch, registry lookup,
  typo correction, and completion to the dimensions a field actually accepts

The default compute path has no I/O and no runtime dependencies. Calendar
arithmetic is available behind the optional `dates-jiff` feature.

## Installation

```sh
cargo add unravel-nl
```

Or add it to `Cargo.toml` directly:

```toml
[dependencies]
unravel-nl = "0.1"
```

Minimum supported Rust version: **1.88** (2024 edition, let-chains).

### Feature Flags

| Feature | Default | Description |
| --- | --- | --- |
| _(none)_ | yes | Core parsing and humanizing. No I/O, no runtime dependencies. |
| `dates-jiff` | no | Calendar arithmetic and relative dates (`next friday`, `in 3 days`) via `jiff`. |
| `timezones-jiff` | no | IANA time zone handling. Implies `dates-jiff`. |
| `wasm` | no | `wasm-bindgen` exports for browser and Node adapters. See [docs/wasm.md](docs/wasm.md). |

## Example

```rust
use unravel_nl::{humanize, HumanizeCtx, Locale, Parser};

let parser = Parser::japanese_building();
let parsed = parser.parse("5尺3寸");

let best = parsed.best.expect("a canonical reading");
assert_eq!(best.unit.as_deref(), Some("m"));
assert_eq!(
    humanize(&best, Some(HumanizeCtx { locale: Some(Locale::Ja) })),
    "5尺3寸 (approx.)"
);
```

## Configured Parser

Use a `Parser` instance when the receiving field knows its measurement
domains. The instance applies the same set before grammar dispatch, registry
lookup, typo correction, completion, and final acceptance. An excluded unit is
therefore absent; it cannot win a ranking decision and cannot be "corrected"
into an enabled unit.

```rust
use unravel_nl::{Dimension, DimensionSet, Parser};

let parser = Parser::new(DimensionSet::from(Dimension::Mass));
let parsed = parser.parse("1,234 kg");

assert_eq!(parsed.best.unwrap().unit.as_deref(), Some("kg"));
```

`Parser::japanese_building()` is the small length-and-area preset.
`Parser::default()` uses the same dimensions without assuming a locale.
`Parser::unrestricted()` keeps the old full catalog for compatibility and
exploration, but it is no longer the implicit public entry point.
`Parser::parse_dimensions_for_editor()`, `Parser::complete()`, and
`Parser::complete_readings()` reuse the same configured boundary.

`ParseCtx::unit_registry` and `ParseCtx::expected_dimensions` have different
jobs. The registry decides which vocabulary exists. The expected set is an
acceptance policy: with the unrestricted registry it can still parse an
out-of-domain reading, move it to `alternatives`, and explain the refusal.
`Parser::new()` sets both to the same dimensions. `Parser::with_context()`
preserves an explicitly narrower acceptance policy.

## Dimension Extraction

There is deliberately no sentence-extraction entry point — no `parse_all()`, and
no plan for one. Sentence scanning has to guess where one value ends and the
next begins, and that guessing was the source of a defect in every round of
review it went through; the reference library this crate follows the API shape
of has no such call either. Single-value parsing, over a field the caller has
already delimited, is the supported shape. The internal note **"Sentence
extraction is out of scope"** records the five rounds of evidence behind that
decision; it is kept with the project's design notes rather than in the
published crate.

Removing the scanner did not settle the wider question, and this section should
not be read as saying it did. A sixth round found that the same class of defect
— a value reported that the input does not hold — was present in single-value
parsing too, and that the property tests meant to forbid it had three blind
spots of their own. One premise behind the five rounds has therefore been
withdrawn: the rounds establish that sentence scanning was the *worse* of the
two, not that what remains is finished. What that round changed is recorded
with the design notes, and the crate is not published while it is open.

The one scanner that remains is narrow by construction: it reads only building
dimensions, out of editor fields, against a label it can name.

For editor fields that only accept dimensions, use the dedicated scanner. It
extracts length and area values, keeps Japanese building units, and avoids
currency/date/general grammar:

```rust
use unravel_nl::{Dimension, Parser};

let parser = Parser::japanese_building();
let matches = parser.parse_dimensions_for_editor(
    "幅3m×奥行4m、予算1234、next friday、6帖、寸法3640"
);

assert_eq!(matches.len(), 4);
assert_eq!(matches[0].parsed.best.as_ref().unwrap().dimension, Some(Dimension::Length));
assert_eq!(matches[2].parsed.best.as_ref().unwrap().dimension, Some(Dimension::Area));
```

The Rust scanner preserves byte spans and uses a token-window dispatch path for
dimension-like substrings. WASM JSON includes both byte and character spans.

Browser adapters additionally try to attach JavaScript `codeUnitStart` /
`codeUnitEnd` fields for UI highlighting. These are best effort: they are
recovered by searching the source string for the matched text, so they are
omitted entirely when that search fails, and callers must treat them as
optional. The byte and character spans from the core are always present.

```rust
use unravel_nl::{Dimension, DimensionSet, Locale, ParseCtx, Parser};

let dimensions = DimensionSet::from(Dimension::Length);
let parser = Parser::with_context(
    dimensions,
    ParseCtx {
        locale: Some(Locale::Ja),
        expected_dimensions: dimensions,
        ..ParseCtx::default()
    },
);
let matches = parser.parse_dimensions_for_editor("3m×4m のLDK");

assert_eq!(matches[0].text, "3m");
assert_eq!(matches[0].start, 0);
assert_eq!(matches[1].text, "4m");
assert_eq!(matches[1].start, 4);
```

## Date Parsing

Date parsing needs the `dates-jiff` feature, which is off by default. Without
it the examples in this section return no reading and report a finding instead —
the parser refuses to guess rather than resolving against an implicit "today".

```toml
unravel-nl = { version = "0.1", features = ["dates-jiff"] }
```

```rust
use unravel_nl::{Date, DimensionSet, Locale, ParseCtx, ParsePurpose, Parser};

let parser = Parser::with_context(
    DimensionSet::new(),
    ParseCtx {
        locale: Some(Locale::En),
        reference_date: Date::new(2026, 7, 19),
        purpose: ParsePurpose::Date,
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("next friday");

assert_eq!(parsed.best.unwrap().date.as_deref(), Some("2026-07-24"));
```

Enable date arithmetic with:

```toml
unravel-nl = { version = "0.1", features = ["dates-jiff"] }
```

Japanese relative dates are supported with the same feature:

```rust
use unravel_nl::{Date, DimensionSet, Locale, ParseCtx, ParsePurpose, Parser};

let parser = Parser::with_context(
    DimensionSet::new(),
    ParseCtx {
        locale: Some(Locale::Ja),
        reference_date: Date::new(2026, 7, 19),
        purpose: ParsePurpose::Date,
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("来週金曜日");

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

UI adapters can turn parser findings into stable severity and rank metadata:

```rust
use unravel_nl::{ranked_findings, Dimension, IssueSeverity, Parser};

let parsed = Parser::new(Dimension::Time.into()).parse("3pm Europe/Paris");
let issues = ranked_findings(&parsed);

assert_eq!(issues[0].severity, IssueSeverity::Error);
assert_eq!(issues[0].rank, 90);
```

Browser-facing adapters live in `web/unravel-adapters.js`. They are dependency
free ESM helpers for DOM inputs, span-preserving `parseAllForUi()`, field-list
`canonicalizeFieldsForUi()`, canonicalizer-result normalization, and React
integration by injection; parser functions are injected so the same code can sit
on top of a WASM bundle or a server bridge. An injected parser must return the
core summary envelope with `ok` and fully ranked `issues`; the adapter does not
guess acceptance or duplicate the Rust issue catalog for hand-built objects.
TypeScript definitions live in
`web/unravel-adapters.d.ts`, and a test compares the two export lists so the
declarations cannot fall behind the module. The React adapter is covered by an
actual React server-render runtime smoke test under
`tests/react_adapter_runtime.mjs`.

There is no custom element — no `defineUnravelElement()`, no `<unravel-input>`,
and no plan for either. It was the one export in this file that nothing in the
repository ever called: no test, no example, no other adapter, only the line
that defined it. An untested wrapper around `createUnravelFieldController()` is
not a feature, it is a second way to reach one, and the controller is the way
that is exercised. Callers who want a custom element can write one over
`createUnravelFieldController()` in a dozen lines and own the registration
policy — the tag name, the registry, and the shadow-DOM question — themselves,
rather than inheriting three defaults this crate never tested.

## Recurrence

There is no recurrence entry point — no `parse_recurrence_fast()`, no
`Kind::Recurrence`, no `Reading::recurrence`, no `ParsePurpose::Recurrence`, and
no plan for any of them. `every monday`, `毎週月曜`, `every third business day`
and raw `FREQ=…` strings are not read; they come back with `best: None` and an
`IssueCode::NoValue` finding that says so, on the same channel every other
unreadable input uses.

The removal has two reasons, and both are about the surface rather than the
code. The reference library this crate follows the API shape of —
`pascalorg/lingo` — documents no recurrence API at all: neither its root README
nor `packages/lingo` mentions `recurrence`, `RRULE`, or `every …` (measured at
commit `8507914c476026afbbc2f4f9fe84b31f2713c6a2`). So the entry point was an
extension of this crate's own invention, held to no external contract. And its
own surface could not say what it refused: it shipped an
`IssueCode::RecurrenceUnsupported` — a code whose whole job was to admit that
the grammar recognized a phrase it could not express — which is the shape of a
feature that never settled where its boundary was. A repeating schedule is a
calendar concern with a real specification (RFC 5545) behind it; a partial
RRULE subset bolted onto a value parser is not that, and callers who need one
are better served by a library that implements the specification.

Dates, times, durations and clock slots are a different question and are
unaffected: `3pm-4pm`, `1h30`, `PT1H30M`, `明日`, and `来週金曜日` read exactly
as they did.

## WASM

```sh
wasm-pack build --target web --out-dir pkg -- --features wasm
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm
node tests/wasm_node_smoke.mjs
```

The WASM package exports `parse_json*` and
`parse_dimensions_for_editor_json*` functions. Their no-context default is the
same length-and-area registry as `Parser::default()`; context variants replace
it with the dimensions named by `expected_dimension` and also apply
`strictness`. The browser smoke
page is `tests/wasm_browser_e2e.html`; serve the repository root and open
`/tests/wasm_browser_e2e.html` after generating `pkg/`. Browser-target Method A
artifacts can be assembled from `pkg/` plus `web/unravel-adapters.*` and
checksummed before vendoring.

## Unit Registry And Strictness

```rust
use unravel_nl::{
    unit_definitions, Dimension, IssueCode, ParseCtx, Parser, Strictness,
};

assert!(unit_definitions().iter().any(|unit| unit.id == "ft"));

let forgiving = Parser::new(Dimension::Length.into()).parse("5 meterz");
assert_eq!(forgiving.best.unwrap().unit.as_deref(), Some("m"));
assert_eq!(
    forgiving.findings.ambiguities[0].code,
    IssueCode::TypoCorrected
);

let confirm_parser = Parser::with_context(
    Dimension::Length.into(),
    ParseCtx {
        strictness: Strictness::Confirm,
        ..ParseCtx::default()
    },
);
let confirm = confirm_parser.parse("5 meterz");
assert!(confirm.best.is_none());
assert_eq!(confirm.suggestions[0].to, "m");
```

Callers can also add deterministic custom unit aliases at parse time:

```rust
use unravel_nl::{CustomUnit, Dimension, ParseCtx, Parser};

let parser = Parser::with_context(
    Dimension::Length.into(),
    ParseCtx {
        custom_units: vec![CustomUnit::new(
            "smoot",
            "m",
            &["smoot", "smoots"],
            Dimension::Length,
            1.7018,
        )],
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("3 smoots");

assert_eq!(parsed.best.unwrap().unit.as_deref(), Some("m"));
```

Custom units can also carry an application-facing kind label:

```rust
use unravel_nl::{CustomUnit, Dimension, ParseCtx, Parser};

let parser = Parser::with_context(
    Dimension::Volume.into(),
    ParseCtx {
        custom_units: vec![CustomUnit::new(
            "case",
            "item",
            &["case", "cases"],
            Dimension::Volume,
            24.0,
        ).kind("package_count")],
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("3 cases");

assert_eq!(parsed.best.unwrap().custom_kind.as_deref(), Some("package_count"));
```

## Completions

```rust
use unravel_nl::{CompletionKind, Dimension, Parser};

let completions = Parser::new(Dimension::Length.into()).complete("10 met");

assert_eq!(completions[0].value, "meter");
assert_eq!(completions[0].canonical.as_deref(), Some("m"));
assert_eq!(completions[0].kind, CompletionKind::Unit);

let readings = Parser::new(Dimension::Mass.into()).complete_readings("10");

assert!(readings.iter().any(|item| item.text == "10 kg"));
```

## Temperature

Temperature readings are normalized to Celsius:

```rust
use unravel_nl::{humanize, Dimension, Parser};

let parsed = Parser::new(Dimension::Temperature.into()).parse("68 F");
let best = parsed.best.expect("temperature");

assert_eq!(best.unit.as_deref(), Some("C"));
assert_eq!(humanize(&best, None), "20 °C");
```

## Approximate And Fuzzy Input

```rust
use unravel_nl::{Dimension, FuzzyProfile, FuzzyTerm, ParseCtx, Parser};

let tolerance = Parser::new(Dimension::Length.into()).parse("10 ± 0.5 mm");
assert!(tolerance.best.unwrap().range.is_some());

let bounded = Parser::new(Dimension::Length.into()).parse("10mm以下");
assert!(bounded.best.unwrap().range.is_some());

let hot_parser = Parser::with_context(
    Dimension::Temperature.into(),
    ParseCtx {
        expected_dimensions: Dimension::Temperature.into(),
        ..ParseCtx::default()
    },
);
let hot = hot_parser.parse("it's hot");

assert!(hot.best.unwrap().range.is_some());

let custom_parser = Parser::with_context(
    Dimension::Mass.into(),
    ParseCtx {
        expected_dimensions: Dimension::Mass.into(),
        fuzzy_profiles: vec![FuzzyProfile::new(
            "parcels",
            Dimension::Mass,
            "kg",
            &[FuzzyTerm::new("heavy", 20.0, 70.0)],
        )],
        ..ParseCtx::default()
    },
);
let custom = custom_parser.parse("heavy");

assert!(custom.best.unwrap().range.is_some());
```

Callers that must reject broad grammar shapes can use `AcceptOptions`:

```rust
use unravel_nl::{AcceptOptions, Dimension, ParseCtx, Parser};

let parser = Parser::with_context(
    Dimension::Mass.into(),
    ParseCtx {
        accept: AcceptOptions {
            ranges: false,
            ..AcceptOptions::default()
        },
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("between 5 and 10 kg");

assert!(parsed.best.is_none());
assert_eq!(parsed.alternatives.len(), 1);
```

Numeric punctuation can be pinned with `NumberFormat` when locale inference is
too permissive:

```rust
use unravel_nl::{Dimension, NumberFormat, ParseCtx, Parser};

let parser = Parser::with_context(
    Dimension::Mass.into(),
    ParseCtx {
        number_format: NumberFormat::CommaDecimal,
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("1,234 kg");

assert_eq!(parsed.best.unwrap().value, Some(1.234));
```

## Currency Rates

Currency conversion only runs when the caller supplies an explicit rate:

```rust
use unravel_nl::{CurrencyRate, Dimension, ParseCtx, Parser};

let parser = Parser::with_context(
    Dimension::Currency.into(),
    ParseCtx {
        currency_rates: vec![CurrencyRate::new("USD", "JPY", 150.0)],
        ..ParseCtx::default()
    },
);
let parsed = parser.parse("USD 10 to JPY");

let best = parsed.best.expect("converted amount");
assert_eq!(best.unit.as_deref(), Some("JPY"));
assert_eq!(best.value, Some(1500.0));
```

## Schemas

```rust
use unravel_nl::{
    canonicalize_values, contract_version, mcp_tool_schema_json,
    parse_input_schema_json, parsed_output_schema_json, CanonicalizeRequest,
    Dimension, ParseCtx, Parser, Strictness,
};

assert_eq!(contract_version(), "unravel-nl.parse.v1");
assert!(parse_input_schema_json().contains("\"text\""));
assert!(parsed_output_schema_json().contains("\"findings\""));
assert!(mcp_tool_schema_json().contains("unravel_nl_parse"));

let values = canonicalize_values(&[CanonicalizeRequest::new(
    "weight",
    "about 20kg",
    Parser::with_context(
        Dimension::Mass.into(),
        ParseCtx {
            strictness: Strictness::Strict,
            ..ParseCtx::default()
        },
    ),
)]);

assert!(!values[0].ok);
assert!(values[0].message.as_ref().unwrap().contains("[APPROXIMATION]"));
```

## Development

```sh
make lint           # cargo fmt --check + clippy -D warnings
make test           # cargo test --all-features
make test-default   # cargo test          (the build most users get)
make test-dates     # cargo test --features dates-jiff
make test-timezones # cargo test --features timezones-jiff
make test-wasm-lib  # cargo test --features wasm  (the shipped WASM feature set)
make test-wasm      # wasm-pack builds + Node/browser adapter smoke tests
make web-test       # TypeScript definition type-check
make check          # lint test test-default test-dates test-timezones test-wasm-lib
```

`make check` runs each feature configuration separately rather than relying on
`--all-features` alone, because code reachable only under one of them has
shipped bugs before. It is `lint` plus the five pure-cargo test lanes and
nothing else: `make test-wasm` and `make web-test` need `wasm-pack`, Node.js,
and `npm install`, so run those yourself.

`make test-wasm` requires [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)
and Node.js. `make test-wasm` and `make web-test` both require `npm install`
inside `web/` first — the React adapter smoke test imports React from
`web/node_modules`.

## Attribution

The public API direction is inspired by `pascalorg/lingo` (MIT). This crate is
an independent Rust implementation and does not copy source code from that
project.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
