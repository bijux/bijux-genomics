# bijux-dna-science Commands

This file is the single source of truth for commands owned by the
`bijux-dna-science` crate.

## Command Owner

The binary is `bijux-dna-science`. Every command accepts `--workspace-root <path>`,
which defaults to the current directory.

## Managed Commands

| Command | Writes files | Purpose |
| --- | --- | --- |
| `validate` | No | Load authored science specs and fail on parse, schema, or cross-reference errors. |
| `build` | Yes | Compile authored specs and refresh governed generated science outputs. |
| `trace [--stage <stage_id>] [--tool <tool_id>]` | No | Print FASTQ stage-tool environment evidence rows, optionally filtered. |
| `closure [--stage <stage_id>] [--tool <tool_id>]` | No | Print FASTQ closure-gate rows, optionally filtered. |
| `release --release-id <release_id>` | Yes | Write an immutable science release bundle for an authored release manifest. |

## Inputs

Commands resolve workspace-relative authored specs and governed upstream evidence
from these roots:

- `science/specs/evidence/**`
- `science/specs/releases/manifests/**`
- `science/docs/upstream/**`

## Outputs

`build` writes deterministic governed outputs under:

- `science/generated/current/evidence/**`
- `science/generated/indexes/science_index.json`

`release` writes immutable bundles under:

- `artifacts/science-releases/<release-id>/**`

## Non-Owned Commands

This crate does not own workflow execution, pipeline planning, stage execution,
container launching, benchmarking, or runtime replay commands. Those commands must
remain in planner, runtime, engine, runner, or environment crates.
