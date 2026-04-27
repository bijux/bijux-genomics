# bijux-dna-science Commands

This file is the single source of truth for commands owned by the
`bijux-dna-science` crate.

## Command Owner

The binary is `bijux-dna-science`. Every command accepts `--workspace-root <path>`,
which defaults to the current directory.

`src/main.rs` is the only Cargo binary entrypoint owned by this crate.

## Managed Commands

| Command | Writes files | Purpose |
| --- | --- | --- |
| `validate` | No | Load authored science specs and fail on parse, schema, or cross-reference errors. |
| `build` | Yes | Compile authored specs, refresh governed generated science outputs, and print the rolled-up source-archive and FASTQ closure summaries. |
| `trace [--stage <stage_id>] [--tool <tool_id>]` | No | Print FASTQ stage-tool environment evidence rows, optionally filtered. |
| `closure [--stage <stage_id>] [--tool <tool_id>]` | No | Print FASTQ closure-gate rows, optionally filtered. |
| `release --release-id <release_id>` | Yes | Write an immutable science release bundle for an authored release manifest. |

## Inputs

Commands resolve workspace-relative authored specs and governed upstream evidence
from these roots:

- [science/specs/evidence/README.md](../../../science/specs/evidence/README.md)
- [science/specs/releases/README.md](../../../science/specs/releases/README.md)
- [science/docs/upstream/README.md](../../../science/docs/upstream/README.md)

## Outputs

`build` writes deterministic governed outputs under:

- [science/generated/README.md](../../../science/generated/README.md)
- [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md)
- [science/generated/indexes/README.md](../../../science/generated/indexes/README.md)
- [science/generated/indexes/science_index.json](../../../science/generated/indexes/science_index.json)

The generated index now includes row counts plus rolled-up source-archive, FASTQ closure,
and FASTQ evidence summaries so operators can spot closure debt before opening the detailed
TSV outputs.

`release` writes immutable bundles under:

- `artifacts/science-releases/<release-id>/**`

## Non-Owned Commands

This crate does not own workflow execution, pipeline planning, stage execution,
container launching, benchmarking, or runtime replay commands. Those commands must
remain in planner, runtime, engine, runner, or environment crates.

## Forbidden Command Surfaces

- No bioinformatics tool execution.
- No container, scheduler, runtime, or runner orchestration.
- No network clients.
- No writes outside `science/generated/**` or `artifacts/science-releases/**`.
