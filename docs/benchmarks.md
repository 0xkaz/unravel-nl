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
| `cargo run --release --example bench -- 100000` | default corpus | 100,000 | 3.702 us/input, 270,100 parses/s |
| `cargo run --release --example bench -- 100000` | hostile no-match corpus | 100,000 | 5.702 us/input, 175,391 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | default corpus | 100,000 | 3.180 us/input, 314,420 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | hostile no-match corpus | 100,000 | 4.580 us/input, 218,329 parses/s |
| `cargo run --release --all-features --example bench -- 100000` | date corpus | 100,000 | 0.944 us/input, 1,058,859 parses/s |

These numbers are a local snapshot, not a universal performance promise. They
exist to catch order-of-magnitude regressions as the grammar expands.
