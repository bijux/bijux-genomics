# DRY_RUN

## Guarantee
Dry-run emits graph + manifest only. No execution.

## Golden example
```
$ bijux dry-run --pipeline fastq.default.v1
graph.json
run_manifest.json
```

Enforced by `tests/dry_run_fastq_golden.rs`.
