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
| `cargo run --release --all-features --example bench -- 200000` | default corpus | 200,000 | 5.299 us/input, 188,726 parses/s |
| `cargo run --release --all-features --example bench -- 200000` | hostile no-match corpus | 200,000 | 7.291 us/input, 137,150 parses/s |
| `cargo run --release --all-features --example bench -- 200000` | date corpus | 200,000 | 0.883 us/input, 1,131,913 parses/s |

These numbers are a local snapshot, not a universal performance promise. They
exist to catch order-of-magnitude regressions as the grammar expands.
