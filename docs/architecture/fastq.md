# FASTQ Architecture

## Ownership

The FASTQ domain owns:

- Stage semantics (validate/trim/filter/stats/merge/correct/umi/screen).
- FASTQ-specific invariants and contracts.
- FASTQ metrics definitions and deltas.
- Canonical pipelines and tool defaults.

The FASTQ domain does **not** own:

- Container/runtime concerns (Docker/Apptainer).
- Runner selection or execution policy.
- Benchmark ranking/selection logic.
- Cross-domain orchestration.

## Boundaries

- **Engine** executes plans and enforces contracts.
- **Domain** defines FASTQ semantics and compatibility.
- **Bench/Analyze** collect metrics and decide meaning (ranking/reporting).

## Execution Model

Runs are materialized under `runs/<run_id>/` with a fixed layout:

- `input_assessment.json` (immutable)
- `execution_manifest.json`
- `environment.json`
- `run_metadata.json`
- `events.jsonl`
- `stages/<stage>/...`
- `summary/`

## Guarantees

- Stage contracts are explicit and enforced.
- Input layout (SE/PE) is never guessed.
- Domain metrics are deterministic given inputs and tools.

## Non-Goals

- Implicit tool switching.
- Hidden heuristics in execution.
- Domain-specific runner logic.
