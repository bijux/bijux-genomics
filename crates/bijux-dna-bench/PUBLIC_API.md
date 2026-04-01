# bijux-dna-bench Public API

The crate root stays intentionally small:
- `pub mod public_api;`
- `pub use public_api::*;`

`src/public_api/mod.rs` re-exports the durable surface:
- benchmark model contracts from `bijux-dna-bench-model`
- workflow entrypoints: `compare`, `gate`, `load_suite`, `summarize`, `BenchRunOptions`
- repository workspace helpers: `bench_data_dir`, `bench_suites_dir`

New stable exports should be added to `src/public_api/mod.rs`, not spread directly through
`src/lib.rs`.
