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
| `target/release/examples/bench 500000` | default corpus | 500,000 | 6.031 us/input, 165,798 parses/s |
| `target/release/examples/bench 500000` | hostile no-match corpus | 500,000 | 7.628 us/input, 131,104 parses/s |
| `target/release/examples/bench 500000` | date corpus | 500,000 | 2.456 us/input, 407,203 parses/s |
| `target/release/examples/entry_bench 500000` | broad `parse()` quantity corpus | 500,000 | 5.253 us/input, 190,378 parses/s |
| `target/release/examples/entry_bench 500000` | `parse_quantity_fast()` corpus | 500,000 | 1.750 us/input, 571,565 parses/s |
| `target/release/examples/entry_bench 500000` | broad `parse()` date corpus | 500,000 | 1.519 us/input, 658,466 parses/s |
| `target/release/examples/entry_bench 500000` | `parse_date_fast()` corpus | 500,000 | 0.210 us/input, 4,765,863 parses/s |
| `target/release/examples/entry_bench 500000` | `parse_all()` sentence corpus | 500,000 | 28.884 us/input, 34,622 parses/s |

The default corpus now includes Unicode normalization, locale number formats,
Japanese large-number notation, business-day recurrence, and technical unit
aliases. These numbers are a local snapshot, not a universal performance
promise. They exist to catch order-of-magnitude regressions as the grammar
expands.

The split entry-point benchmark shows why callers should use narrower APIs
when a UI field already knows the expected shape. Broad `parse()` remains the
compatibility entry; `parse_quantity_fast()` and `parse_date_fast()` avoid
unrelated grammar families and rich ambiguity checks.

The sentence scanner uses token-window dispatch for numeric and dimension-like
substrings before falling back to broad clause parsing. This keeps byte spans
stable while avoiding broad grammar checks for every long sentence fragment.
