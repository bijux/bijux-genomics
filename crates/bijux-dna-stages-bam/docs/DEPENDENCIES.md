# Dependencies

`bijux-dna-stages-bam` is a library crate for BAM stage contracts. Its
dependency graph must stay below planner, runtime, runner, API, and CLI layers.

## Normal Dependencies
| Dependency | Reason |
| --- | --- |
| `anyhow` | Fallible plugin and parser contract errors. |
| `bijux-dna-core` | Shared contract versions, canonical fingerprints, and metrics envelopes. |
| `bijux-dna-domain-bam` | BAM stage IDs, stage registry, BAM metric contracts, and invariant evaluation. |
| `bijux-dna-infra` | Stable file hashing for input fingerprints only. |
| `bijux-dna-stage-contract` | Shared stage plan, invocation, artifact, and plugin output contracts. |
| `serde` | Contract serialization derives. |
| `serde_json` | JSON metrics envelopes and parser output payloads. |

## Dev Dependencies
| Dependency | Reason |
| --- | --- |
| `bijux-dna-policies` | Repository guardrail checks. |
| `bijux-dna-testkit` | Shared fixture and test helpers. |
| `walkdir` | Source and fixture tree scans in boundary tests. |

## Forbidden Normal Dependencies
- `bijux-dna-api`
- `bijux-dna-engine`
- `bijux-dna-environment`
- `bijux-dna-pipelines`
- `bijux-dna-planner-*`
- `bijux-dna-runner`
- `bijux-dna-runtime`
- CLI frameworks such as `clap`

## Review Rule
Any new normal dependency must support BAM stage contracts directly. If it owns
tool choice, command execution, orchestration, storage, network access, or user
command surfaces, it belongs outside this crate.
