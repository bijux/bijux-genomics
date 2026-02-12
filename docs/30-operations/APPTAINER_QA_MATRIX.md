# Apptainer QA Matrix

This page tracks the per-tool Apptainer readiness status:

- build ok
- smoke ok
- run ok

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
