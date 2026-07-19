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
| `cargo run --release --example bench -- 100000` | default corpus | 100,000 | 4.789 us/input, 208,796 parses/s |
| `cargo run --release --example bench -- 100000` | hostile no-match corpus | 100,000 | 10.316 us/input, 96,932 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | default corpus | 100,000 | 4.754 us/input, 210,363 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | hostile no-match corpus | 100,000 | 10.074 us/input, 99,267 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | date corpus | 100,000 | 0.677 us/input, 1,476,810 parses/s |

These numbers are a local snapshot, not a universal performance promise. They
exist to catch order-of-magnitude regressions as the grammar expands.
