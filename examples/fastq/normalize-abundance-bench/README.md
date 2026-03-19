# fastq_normalize_abundance_bench

## Purpose
Benchmark FASTQ abundance normalization on local or HPC runtimes using the first-class `normalize-abundance` bench command.

Canonical invocation:
`cargo run -q -p bijux-dna -- bench fastq normalize-abundance --sample-id abundance_norm --table /abs/path/feature-table.tsv --out artifacts/bench --tools auto`

## HPC invocation
`cargo run -q -p bijux-dna -- bench fastq normalize-abundance --sample-id abundance_norm --table /abs/path/feature-table.tsv --out /scratch/$USER/bijux-bench --tools auto --replicates 3 --jobs 4 --explain`

## Outputs
- `bench.jsonl`
- `bench.sqlite`
- `report.json`
- per-tool `metrics.json`
- per-tool `normalize_abundance_report.json`

## Note
This benchmark is table-backed rather than read-backed, so it uses the first-class bench command directly instead of the corpus-oriented example runner.
