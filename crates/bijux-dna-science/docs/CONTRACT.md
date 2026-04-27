# bijux-dna-science Contract

## Role
`bijux-dna-science` validates authored science inputs, resolves typed references, compiles
traceability outputs, and cuts immutable science release bundles.

## Inputs and Outputs

- Authored input: [science/specs/evidence/README.md](../../../science/specs/evidence/README.md)
- Governed upstream evidence input: [science/docs/upstream/README.md](../../../science/docs/upstream/README.md)
- Generated output: [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md)
- Generated index: [science/generated/indexes/README.md](../../../science/generated/indexes/README.md)
- Machine-readable index entrypoint: [science/generated/indexes/science_index.json](../../../science/generated/indexes/science_index.json)
- Release output: `artifacts/science-releases/**`

## Generated Evidence Contract

`build` must keep committed generated outputs byte-for-byte aligned with
`compile::compile_workspace`. Generated TSV files must stay rectangular, sorted by
the compiler's deterministic ordering, and free of comment rows. JSON outputs must
use stable pretty rendering.

The generated FASTQ evidence set includes the environment matrix, container
reference matrix, download backlog, paper archive matrix, closure gate, truth
delta, missing closure prerequisites, and default binding risk ledger.

[science/generated/indexes/science_index.json](../../../science/generated/indexes/science_index.json)
is not just an inventory counter. The wider generated output surface is
documented by [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md)
and [science/generated/indexes/README.md](../../../science/generated/indexes/README.md).
The index must also summarize the source-archive and FASTQ closure surfaces through:

- `source_archive_summary` for source kind, access mode, archive status, and missing-tool rollups

- `fastq_closure_summary` for rolled-up closure state and blocker/warning counts
- `fastq_evidence_summary` for backlog, paper archive, prerequisite, risk, and truth-delta counts

## Boundaries
- No pipeline runtime orchestration.
- No stage execution.
- No runner dependency.
- No graph compilation logic inside rendering code.
- No filesystem concerns inside pure domain types.

## Owns

- Authored science spec loading and validation.
- Typed science identifier data structures.
- Deterministic evidence-row compilation.
- Stable TSV and JSON rendering for science outputs.
- Science release bundle writing under `artifacts/science-releases/`.

## Does Not Own

- Pipeline planning.
- Stage execution.
- Container runtime resolution.
- Benchmark policy decisions outside compiled science evidence.
- Tool invocation or filesystem writes outside governed science outputs.
