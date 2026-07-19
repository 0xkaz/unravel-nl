# Benchmarks

`unravel-nl` ships a dependency-free benchmark example:

```sh
cargo run --release --example bench -- 100000
cargo run --release --all-features --example bench -- 100000
cargo run --release --all-features --example entry_bench -- 100000
```

The argument is the number of parse iterations. The benchmark runs a fixed
corpus through `parse()` and reports matched parses, microseconds per input,
and parses per second.

## Local Snapshot

Measured on the development machine on 2026-07-20:

| Command | Corpus | Iterations | Result |
| --- | --- | ---: | --- |
| `target/release/examples/bench 500000` | default corpus | 500,000 | 6.489 us/input, 154,110 parses/s |
| `target/release/examples/bench 500000` | hostile no-match corpus | 500,000 | 9.749 us/input, 102,572 parses/s |
| `target/release/examples/bench 500000` | date corpus | 500,000 | 3.253 us/input, 307,390 parses/s |
| `target/release/examples/entry_bench 500000` | broad `parse()` quantity corpus | 500,000 | 4.684 us/input, 213,478 parses/s |
| `target/release/examples/entry_bench 500000` | `parse_quantity_fast()` corpus | 500,000 | 1.786 us/input, 560,030 parses/s |
| `target/release/examples/entry_bench 500000` | broad `parse()` date corpus | 500,000 | 1.854 us/input, 539,380 parses/s |
| `target/release/examples/entry_bench 500000` | `parse_date_fast()` corpus | 500,000 | 0.309 us/input, 3,238,781 parses/s |
| `target/release/examples/entry_bench 500000` | `parse_all()` sentence corpus | 500,000 | 107.016 us/input, 9,344 parses/s |

The default corpus now includes Unicode normalization, locale number formats,
Japanese large-number notation, business-day recurrence, and technical unit
aliases. These numbers are a local snapshot, not a universal performance
promise. They exist to catch order-of-magnitude regressions as the grammar
expands.

The split entry-point benchmark shows why callers should use narrower APIs
when a UI field already knows the expected shape. Broad `parse()` remains the
compatibility entry; `parse_quantity_fast()` and `parse_date_fast()` avoid
unrelated grammar families and rich ambiguity checks.
