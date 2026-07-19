# WASM Adapter

The core crate keeps runtime dependencies disabled by default. WASM exports are
enabled only with the `wasm` feature:

```sh
wasm-pack build --target web --out-dir pkg -- --features wasm
wasm-pack build --target nodejs --out-dir pkg-node -- --features wasm
```

The web target emits `parse_json(text)` and `parse_json_with_locale(text,
locale)`. Both return a compact JSON summary with `ok`, `best`, and ranked
`issues` fields.

## Local Snapshot

Measured on the development machine on 2026-07-20:

| Command | Output | Size / digest |
| --- | --- | --- |
| `wasm-pack build --target web --out-dir pkg -- --features wasm` | `pkg/` | 248K |
| same | `pkg/unravel_nl_bg.wasm` | 209,677 bytes |
| same | `pkg/unravel_nl_bg.wasm` sha256 | `6161c245b7fbc131d685ac13c5dfbc13bd2ec2d86293a7195b2d673c5012e35e` |

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
Japanese length, a business-day recurrence, and an unsupported timezone case,
then writes a JSON status object into `#status`.
