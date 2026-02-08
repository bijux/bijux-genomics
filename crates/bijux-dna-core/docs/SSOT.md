# SSOT

## Ownership Table
| Item | Owner Module | Allowed References |
| --- | --- | --- |
| Identifier catalog constants | `id_catalog.rs` | All crates via `bijux-dna-core::id_catalog` |
| StageId/StepId/ToolId/ArtifactId/ProfileId | `ids.rs` | All crates via `bijux-dna-core` |
| ContractVersion | `contract/version.rs` | Core + runtime + engine + api |
| ExecutionGraph | `contract/execution/graph.rs` | planners, engine, api |
| ExecutionManifest | `contract/execution/manifest.rs` | runtime, engine, analyze |
| ToolInvocation | `contract/tooling/mod.rs` | runtime, engine, analyze |
| MetricsEnvelope | `metrics/types.rs` | stages, analyze, benchmark |

## Forbidden Elsewhere
- Defining new ID types.
- Re-implementing hashing or canonical JSON.
- Adding contract fields without versioning.

## Public surface note
Consumers should import via `prelude` unless a narrower module (`contract`, `ids`, `metrics`) is preferred.
