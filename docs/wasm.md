# WASM Adapter

The core crate keeps runtime dependencies disabled by default. WASM exports are
enabled only with the `wasm` feature:

```sh
wasm-pack build --target web --out-dir pkg -- --features wasm
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm
```

The web target emits `parse_json(text)`, `parse_json_with_locale(text, locale)`,
`parse_json_with_context(text, locale, expected_dimension, strictness)`,
`parse_all_json(text)`, `parse_all_json_with_locale(text, locale)`, and
`parse_all_json_with_context(text, locale, expected_dimension, strictness)`.
It also emits `parse_dimensions_for_editor_json(text)` and
`parse_dimensions_for_editor_json_with_context(text, locale, expected_dimension,
strictness)` for building-dimension-only editor scans.
Single-value exports return a compact JSON summary with `ok`, `best`, and
ranked `issues` fields. Multi-value exports return an array of matches with
byte spans, character spans, original text, and a compact parsed summary. The
Rust core uses byte spans; browser adapters should use `codeUnitStart` /
`codeUnitEnd` from `parseAllForUi()` when slicing JavaScript strings.

The browser adapter files are `web/unravel-adapters.js` and
`web/unravel-adapters.d.ts`. A Method A browser artifact should include those
adapter files plus the generated `pkg/` web target and a checksum manifest.

## Local Snapshot

Measured on the development machine on 2026-07-20:

| Command | Output | Size / digest |
| --- | --- | --- |
| `wasm-pack build --target web --out-dir pkg -- --features wasm` | `pkg/` | 328K |
| same | `pkg/unravel_nl_bg.wasm` | 279,448 bytes |
| same | `pkg/unravel_nl_bg.wasm` sha256 | `5a3b2a7ba3910d7060cf98558c33b83fe414d96845921fdf31b66912666aafc4` |

Smoke checks:

```sh
node tests/wasm_node_smoke.mjs
```

Browser E2E page:

```sh
python3 -m http.server 8765
open http://127.0.0.1:8765/tests/wasm_browser_e2e.html
```

The page loads `pkg/unravel_nl.js`, initializes the `.wasm` module, parses a
Japanese length, room dimension extraction, a business-day recurrence, and an
unsupported timezone case, then writes a JSON status object into `#status`.
