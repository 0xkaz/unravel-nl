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

Measured on the development machine on 2026-07-19:

| Command | Corpus | Iterations | Result |
| --- | --- | ---: | --- |
| `cargo run --release --example bench -- 100000` | default corpus | 100,000 | 7.111 us/input, 140,623 parses/s |
| `cargo run --release --example bench -- 100000` | hostile no-match corpus | 100,000 | 18.233 us/input, 54,847 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | default corpus | 100,000 | 5.652 us/input, 176,921 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | hostile no-match corpus | 100,000 | 14.200 us/input, 70,422 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | date corpus | 100,000 | 1.070 us/input, 934,885 parses/s |

These numbers are a local snapshot, not a universal performance promise. They
exist to catch order-of-magnitude regressions as the grammar expands.
