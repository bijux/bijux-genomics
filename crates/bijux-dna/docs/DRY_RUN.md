# DRY_RUN

## Guarantee
Dry-run emits graph + manifest only. No execution.

## Golden example
```
$ bijux-dna run preprocess --dry-run --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE
graph.json
run_manifest.json
```

Enforced by `tests/contracts/dry_run/fastq_golden.rs`.
