# bijux-dna-science Contract

## Role
`bijux-dna-science` validates authored science inputs, resolves typed references, compiles
traceability outputs, and cuts immutable science release bundles.

## Inputs and Outputs

- Authored input: `science/specs/**`
- Governed upstream evidence input: `science/docs/upstream/**`
- Generated output: `science/generated/current/evidence/**`
- Generated index: `science/generated/indexes/science_index.json`
- Release output: `artifacts/science-releases/**`

## Generated Evidence Contract

`build` must keep committed generated outputs byte-for-byte aligned with
`compile::compile_workspace`. Generated TSV files must stay rectangular, sorted by
the compiler's deterministic ordering, and free of comment rows. JSON outputs must
use stable pretty rendering.

The generated FASTQ evidence set includes the environment matrix, container
reference matrix, download backlog, paper archive matrix, closure gate, truth
delta, missing closure prerequisites, and default binding risk ledger.

`science/generated/indexes/science_index.json` is not just an inventory counter.
It must also summarize the FASTQ closure surface through:

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
