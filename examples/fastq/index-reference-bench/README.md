# fastq_index_reference_bench

## Purpose
Benchmark FASTQ reference indexing on local or HPC runtimes using the first-class `index-reference` bench command.

Canonical invocation:
`cargo run -q -p bijux-dna -- bench fastq index-reference --sample hg38_index --reference-fasta /abs/path/reference.fa --out artifacts/bench --tools auto`

## HPC invocation
`cargo run -q -p bijux-dna -- bench fastq index-reference --sample hg38_index --reference-fasta /abs/path/reference.fa --out /scratch/$USER/bijux-bench --tools auto --jobs 1`

## Outputs
- `bench.jsonl`
- `bench.sqlite`
- `report.json`
- per-tool `metrics.json`
- per-tool `index_reference_report.json`
