# Benchmarks

`unravel-nl` ships a dependency-free benchmark example:

```sh
cargo run --release --example bench -- 100000
cargo run --release --all-features --example bench -- 100000
```

The argument is the number of parse iterations. The benchmark runs a fixed
corpus through `parse()` and reports matched parses, microseconds per input,
and parses per second.

## Local Snapshot

Measured on the development machine on 2026-07-20:

| Command | Corpus | Iterations | Result |
| --- | --- | ---: | --- |
| `target/release/examples/bench 500000` | default corpus | 500,000 | 13.059 us/input, 76,575 parses/s |
| `target/release/examples/bench 500000` | hostile no-match corpus | 500,000 | 12.613 us/input, 79,282 parses/s |
| `target/release/examples/bench 500000` | date corpus | 500,000 | 3.649 us/input, 274,028 parses/s |

The default corpus now includes Unicode normalization, locale number formats,
Japanese large-number notation, business-day recurrence, and technical unit
aliases. These numbers are a local snapshot, not a universal performance
promise. They exist to catch order-of-magnitude regressions as the grammar
expands.
