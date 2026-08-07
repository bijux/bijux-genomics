# bijux-dna-science Dependencies

## Dependency Role

`bijux-dna-science` is a science compiler and command surface. Dependencies must
support deterministic parsing, rendering, file IO, command parsing, and policy
guardrails without pulling in runtime execution or planning responsibilities.

## Normal Dependencies

| Dependency | Role |
| --- | --- |
| `anyhow` | Error context for command and compiler flows. |
| `bijux-dna-bench-model` | Persisted benchmark gate policy and decision records consumed by pure science evaluation. |
| `bijux-dna-core` | Canonical metric direction semantics used by scientific gates. |
| `bijux-dna-infra` with `yaml` | Shared deterministic file IO and governed YAML parsing. |
| `clap` | CLI parser for the `bijux-dna-science` binary. |
| `flate2` | Streaming multi-member gzip decoding for independent FASTQ truth. |
| `serde` | Authored spec and compiled model serialization. |
| `serde_json` | Science index and release JSON rendering. |
| `sha2` | Streaming exact per-record digest derivation without retaining all reads. |
| `toml` | Governed TOML evidence-table parsing. |
| `walkdir` | Deterministic recursive YAML discovery. |

## Development Dependencies

| Dependency | Role |
| --- | --- |
| `bijux-dna-policies` | Workspace guardrail configuration used by crate tests. |

## Forbidden Dependency Edges

The crate must not depend on planner, runtime, engine, runner, environment, or
workflow execution crates. Science evidence can inform those surfaces, but this
crate must not import them or couple command behavior to execution backends.

## Dependency Review Notes

Internal `bijux-dna-*` dependencies must be declared through the workspace catalog. The science
crate's model, metric-semantics, and infrastructure edges are boundary-tested and must stay
cataloged.

The crate previously declared `thiserror` without using it. That direct dependency
is intentionally absent; error handling currently uses `anyhow` because failures
are reported as command/compiler diagnostics rather than a public custom error enum.
