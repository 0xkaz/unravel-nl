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
Single-value exports return a compact JSON summary with `ok`, `best`, and
ranked `issues` fields. Multi-value exports return an array of matches with
byte spans, original text, and a compact parsed summary.

## Local Snapshot

Measured on the development machine on 2026-07-20:

| Command | Output | Size / digest |
| --- | --- | --- |
| `wasm-pack build --target web --out-dir pkg -- --features wasm` | `pkg/` | 312K |
| same | `pkg/unravel_nl_bg.wasm` | 269,025 bytes |
| same | `pkg/unravel_nl_bg.wasm` sha256 | `120b37adbc748f31b73f1be5882ad6fb13bb1b31ac29e35a7b6cc2b82cf558f0` |

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
