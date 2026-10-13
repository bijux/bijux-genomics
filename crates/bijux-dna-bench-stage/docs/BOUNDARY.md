# Local stage boundary

Owner: Benchmark local stage contracts, admission and optional repository/execution services.
Scope: Own persisted stage inventory, matrix and command schemas; validate semantic selections; optionally acquire repository matrices, project plans and publish fixture/command files.
Allowed inputs: Explicit stage records, matrix paths, normalized domain rows and caller-supplied canonical plans/materializers. The default closure uses serde, serde_json and schemars without workspace crates.
Forbidden dependencies: CLI parsing, API presentation, direct native backend engines and sibling repository checkouts. Optional execution admits core/infrastructure/stage contracts; repository admits infrastructure and TOML decoding.
Forbidden effects: Direct backend process spawning, network requests and process-global environment mutation. Optional services may read admitted matrices and publish owned files; they do not claim backend execution success.
Validation command: `cargo test --locked -p bijux-dna-bench-stage --all-targets --all-features`

Persisted schema identities and semantic reasons are canonical here. Destination admission refuses conflicting bytes, symlinks and kinds before writes; it does not provide multi-file transactions or protection against namespace races. Synthetic fixture payloads are file-shape evidence only.
