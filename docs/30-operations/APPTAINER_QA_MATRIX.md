# Apptainer QA Matrix

## What
This page tracks the per-tool Apptainer readiness status:

- build ok
- smoke ok
- run ok

## Why
Keeps image readiness verifiable and reproducible before benchmark/production runs.

## Non-goals
- Defining tool scientific validity.
- Replacing per-tool smoke manifests.

## Contracts
- Matrix rows must reflect generated SIF inventory + smoke manifests.
- Missing smoke/version metadata is a contract violation.

## Regenerate

```bash
bijux environment apptainer-qa-matrix \
  --hpc-root /home/bijan/bijux \
  --out docs/30-operations/APPTAINER_QA_MATRIX.md
```

## Inventory Source

The matrix is generated from:

- `bijux-dna-containers/*/*.sif`
- sibling smoke manifests (`<digest>.smoke_manifest.json`)

Use JSON inventory directly when needed:

```bash
bijux environment sif-inventory --hpc-root /home/bijan/bijux --json
```

## Examples
- Use this matrix to gate `supported` tools before profile promotion.

## Failure modes
- Outdated matrix if inventory is stale.
- False green if smoke manifests are missing required schema fields.
