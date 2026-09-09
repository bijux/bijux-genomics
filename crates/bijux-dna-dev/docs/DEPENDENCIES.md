# bijux-dna-dev Dependencies

This crate is allowed to depend on stable workspace primitives and developer-policy crates, but it must not become a production runtime dependency.

## Current dependency shape
- `bijux-dna-core` is used for shared identifiers and catalog helpers needed by developer automation.
- `bijux-dna-infra` is used for governed filesystem primitives such as directory creation and file writes.
- `bijux-dna-domain-compiler` is invoked as a library for artifact-scoped generated-config drift
  checks; the compiler retains domain projection semantics and the developer surface only reports
  the comparison verdict.
- `bijux-dna-runtime` is used to read and compose runtime-owned immutable lifecycle evidence.
- `bijux-dna-runner` is used only through its stable facade to materialize or invoke typed,
  non-waiting scheduler effects; runner retains backend execution ownership.
- `bijux-dna-policies` owns verdict derivation consumed by thin developer composition commands.
- `clap`, `anyhow`, `serde`, `serde_json`, `toml`, `regex`, `aho-corasick`, `sha2`, `chrono`,
  `walkdir`, and `reqwest` support CLI parsing, error context, structured data, checksums,
  bounded multi-pattern caller analysis, deterministic traversal, and explicit automation workflows.

- `twox-hash` supplies only xxHash64 for the qualified Nextest hash partition protocol. It is already present in the workspace lockfile; developer planning reuses the implementation without enabling its random or allocation features.

## Allowed direction
- `bijux-dna-dev` may depend on low-level shared crates that do not depend back on it.
- `bijux-dna-dev` may compose stable runtime transitions and runner effects, but must not implement
  their persistence, scheduler, or container-execution internals.
- Production runtime, planner, engine, and domain crates must not depend on `bijux-dna-dev`.
- Developer-policy tests may call the `bijux-dna-dev` binary surface, but production command execution must remain outside this crate.

## Boundary checks
- Use `cargo tree -p bijux-dna-dev --no-default-features --depth 1` to inspect direct dependencies.
- Dependency changes must be reviewed against [BOUNDARY.md](BOUNDARY.md) before commit.
- External network-capable dependencies are allowed only when a documented command owns the network behavior and keeps offline failure modes explicit.

## API documentation capability

The default `api-docs` feature retains API reference generation. Operator builds
with `--no-default-features` omit the API facade dependency while retaining
container qualification, policy/schema checks and CI execution. Qualification
composition comes directly from benchmark services; schemas and report rendering
come from their policy owner. `generate-docs` in an operator build reports the
missing feature before creating any output. Rebuild with `--features api-docs`
when generating the full documentation set.

Full workspace verification still builds all features. A minimal operator build
is a separate supported build scope, not evidence of all-feature test coverage.
