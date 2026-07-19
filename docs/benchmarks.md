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
| `cargo run --release --all-features --example bench -- 200000` | default corpus | 200,000 | 9.566 us/input, 104,532 parses/s |
| `cargo run --release --all-features --example bench -- 200000` | hostile no-match corpus | 200,000 | 11.142 us/input, 89,752 parses/s |
| `cargo run --release --all-features --example bench -- 200000` | date corpus | 200,000 | 1.377 us/input, 726,054 parses/s |

These numbers are a local snapshot, not a universal performance promise. They
exist to catch order-of-magnitude regressions as the grammar expands.
