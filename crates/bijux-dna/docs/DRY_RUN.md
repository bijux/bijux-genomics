# DRY_RUN

## Guarantee
Dry-run emits graph + manifest only. No execution.

## Golden example
```
$ bijux-dna dry-run --pipeline fastq.default.v1
graph.json
run_manifest.json
```

Enforced by `tests/contracts/dry_run/fastq_golden.rs`.
