# Dependencies

`bijux-dna-pipelines` is a pure library crate. Runtime dependencies must support typed profile construction, deterministic defaults, registry lookups, and serialization only.

## Runtime Dependencies
- `anyhow` — fallible lookup and merge APIs.
- `serde`, `serde_json`, `toml` — stable profile, manifest, defaults-ledger, and owner metadata encoding.
- `sha2` — deterministic profile and contract hashes.
- `bijux-dna-core` — typed identifiers, shared vocabulary, hashing, and contract primitives.
- `bijux-dna-domain-fastq`, `bijux-dna-domain-bam`, `bijux-dna-domain-vcf` — domain contract vocabulary consumed by canonical profiles.

## Test-Only Dependencies
- `insta` — snapshot contracts.
- `walkdir` — policy and source-tree guardrails.
- `bijux-dna-policies` — repository policy tests.
- `bijux-dna-testkit` — deterministic test fixtures and helpers.
- `bijux-dna-runtime` — downstream runtime model contract checks only.

## Forbidden Dependency Direction
This crate must not depend on command, planner, engine, runner, stage implementation, database, environment, analysis application, or science orchestration crates. Those crates may consume pipeline contracts; pipeline contracts must not consume them.

## Review Rules
- Add runtime dependencies only when profile construction, defaults ledgers, or registry lookup cannot stay simpler without them.
- Prefer workspace dependency declarations when the dependency is already listed in the workspace root.
- Keep downstream runtime and policy crates in `dev-dependencies` unless a public contract genuinely requires them.
