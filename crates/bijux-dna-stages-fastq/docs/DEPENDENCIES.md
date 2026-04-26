# Dependencies

`bijux-dna-stages-fastq` is a library crate for FASTQ stage contracts,
observer parsing, and metrics normalization. Its dependency graph must stay
below planner, runner, engine, API, CLI, pipeline, and environment layers.

## Normal Dependencies

| Dependency | Reason |
| --- | --- |
| `anyhow` | Fallible parser, plugin, and metrics-envelope contracts. |
| `bijux-dna-core` | Shared IDs, artifact contracts, metrics envelopes, canonicalization, and measurement types. |
| `bijux-dna-domain-fastq` | FASTQ stage IDs, contract registry, domain metrics, parser models, and execution-support truth. |
| `bijux-dna-infra` | Stable file hashing and governed JSON writes for observer artifacts. |
| `bijux-dna-stage-contract` | Shared stage plan, invocation, artifact, and plugin output contracts. |
| `flate2` | Read gzip-compressed FASTQ inputs while deriving metrics. |
| `serde` | Serialization derives for public and internal metric types. |
| `serde_json` | JSON metrics, reports, and observer output parsing. |
| `tracing` | Local instrumentation hooks around observer and metrics work. |

## Dev Dependencies

| Dependency | Reason |
| --- | --- |
| `anyhow` | Fallible integration tests. |
| `bijux-dna-policies` | Shared guardrail tests. |
| `bijux-dna-testkit` | Shared fixture and deterministic JSON helpers. |
| `serde_json` | Test fixture assertions. |
| `sha2` | Snapshot and fixture hash assertions. |
| `tempfile` | Isolated test directories. |
| `walkdir` | Source, fixture, and docs tree scans. |

## Forbidden Normal Dependencies

- `bijux-dna`
- `bijux-dna-analyze`
- `bijux-dna-api`
- `bijux-dna-engine`
- `bijux-dna-environment`
- `bijux-dna-environment-qa`
- `bijux-dna-pipelines`
- `bijux-dna-planner-*`
- `bijux-dna-runner`
- `bijux-dna-runtime`
- CLI frameworks such as `clap`

## Verification

Internal `bijux-dna-*` dependencies must come from the workspace catalog. Do not
add local `path = "../..."` declarations for core, domain, infra, or contract
crates; the boundary tests reject that shape.

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo tree -p bijux-dna-stages-fastq --no-default-features --edges normal,dev
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --test boundaries --no-default-features
```
