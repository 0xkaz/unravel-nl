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
| `cargo run --release --example bench -- 100000` | default corpus | 100,000 | 4.014 us/input, 249,131 parses/s |
| `cargo run --release --example bench -- 100000` | hostile no-match corpus | 100,000 | 5.783 us/input, 172,910 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | default corpus | 100,000 | 4.565 us/input, 219,068 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | hostile no-match corpus | 100,000 | 5.933 us/input, 168,547 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | date corpus | 100,000 | 2.207 us/input, 453,063 parses/s |

These numbers are a local snapshot, not a universal performance promise. They
exist to catch order-of-magnitude regressions as the grammar expands.
