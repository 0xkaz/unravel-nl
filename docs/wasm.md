# WASM Adapter

The core crate keeps runtime dependencies disabled by default. WASM exports are
enabled only with the `wasm` feature:

```sh
wasm-pack build --target web --out-dir pkg -- --features wasm
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm
```

The web target emits `parse_json(text)`, `parse_json_with_locale(text, locale)`,
and `parse_json_with_context(text, locale, expected_dimension, strictness)`.
It also emits `parse_dimensions_for_editor_json(text)` and
`parse_dimensions_for_editor_json_with_context(text, locale, expected_dimension,
strictness)` for building-dimension-only editor scans.

The `expected_dimension` argument is a hard filter, not a hint: a reading from
any other measurement domain is refused with `REJECTED_BY_POLICY` instead of
being returned. Readings that carry no dimension at all — a bare number, a
date, a recurrence — are not refused by it. Several domains are written as a
comma-separated list (`"length,area"`); an empty string accepts every domain,
and an unrecognized name is dropped without taking the rest of the list with
it. If *every* name in the tag is unrecognized — a bare `"lenght"` — the call
is refused with `REJECTED_BY_POLICY` rather than run: dropping the last member
would leave the empty set, which is no restriction at all, so a typo would
silently turn a hard filter into none. The array-returning exports report that
refusal as one match spanning the whole input, since an empty array would be
the same silence.
Single-value exports return a compact JSON summary object with exactly the keys
`ok`, `input`, `best`, and ranked `issues` — `input` is always present, echoing
the string that was parsed. When `best` is a range reading it also carries a
`range` object with `from` and `to`, each a nested reading with the same fields
a top-level reading has. The editor extractor exports return an array of
matches with byte spans, character spans, original text, and a compact parsed
summary. The Rust core uses byte spans; browser adapters should use `codeUnitStart` /
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
