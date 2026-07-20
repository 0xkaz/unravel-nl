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

## Artifact Size

`wasm-pack build --target web --out-dir pkg -- --features wasm` produces a
`pkg/` directory of roughly 380 KB, of which `pkg/unravel_nl_bg.wasm` is about
330 KB. Treat both as ballpark figures: they move with the toolchain version.

No reference digest is published here, because the build is not reproducible
across machines or toolchain versions — the same source produces a different
`unravel_nl_bg.wasm` hash on different setups. Compute the digest for the
artifact you actually ship, and pin that:

```sh
shasum -a 256 pkg/unravel_nl_bg.wasm
```

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
