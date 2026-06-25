# FASTQ + BAM Benchmark Readiness Dashboard

## Summary

- Expected pairs: 120
- Ready pairs: 120
- Blocked pairs: 0
- Exact blocker counts: 

## Surface Summary

| Surface | Status | Scope | Total | Ready | Blocked | Evidence |
| --- | --- | --- | ---: | ---: | ---: | --- |
| Matrix | complete | all governed fastq and bam stage-tool pairs | 120 | 120 | 0 | stages=51, tools=65, gaps=none=120 |
| Adapters | complete | all governed fastq and bam stage-tool pairs | 120 | 120 | 0 | runnable=120 |
| Parsers | complete | benchmark-reporting pairs only | 118 | 118 | 0 | excluded=2, statuses=benchmark_normalized=56, comparable=12, parse_normalized=3, parser_fixture_validated=49 |
| Corpora | complete | all governed fastq and bam stage-tool pairs | 120 | 120 | 0 | corpora=8, assigned stages=51, statuses=asset:reference-index-assets=2, fixture:corpus-01-adna-bam-mini=7, fixture:corpus-01-adna-damage-mini=9, fixture:corpus-01-bam-mini=28, fixture:corpus-01-genotyping-mini=1, fixture:corpus-01-kinship-mini=2, fixture:corpus-01-mini=62, fixture:corpus-02-edna-mini=4, fixture:corpus-03-amplicon-mini=5 |
| Assets | complete | asset-required benchmark-submission pairs | 20 | 20 | 0 | assigned=20, not_required=100 |
| Reports | complete | governed local report surfaces | 5 | 5 | 0 | expected_results=120, stage_sections=51, tool_sections=65, corpus_sections=8 |

## Report Outputs

| Report | Output | Governed items |
| --- | --- | --- |
| pair_readiness | benchmarks/readiness/pair-readiness.tsv | 120 stage_tool_pairs |
| expected_benchmark_results | benchmarks/readiness/expected-benchmark-results.tsv | 120 expected_results |
| stage_centric_report | benchmarks/readiness/stage-centric-report.md | 51 stage_sections |
| tool_centric_report | benchmarks/readiness/tool-centric-report.md | 65 tool_sections |
| corpus_centric_report | benchmarks/readiness/corpus-centric-report.md | 8 corpus_sections |

## Exact Blockers

| Domain | Stage | Tool | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
