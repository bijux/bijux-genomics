# FASTQ + BAM Benchmark Readiness Dashboard

## Summary

- Expected pairs: 122
- Ready pairs: 118
- Blocked pairs: 4
- Exact blocker counts: corpus=1, support=3

## Surface Summary

| Surface | Status | Scope | Total | Ready | Blocked | Evidence |
| --- | --- | --- | ---: | ---: | ---: | --- |
| Matrix | attention_required | all governed fastq and bam stage-tool pairs | 122 | 118 | 4 | stages=51, tools=67, gaps=corpus=1, none=118, support=3 |
| Adapters | attention_required | all governed fastq and bam stage-tool pairs | 122 | 119 | 3 | declared_only=3, runnable=119 |
| Parsers | complete | benchmark-reporting pairs only | 116 | 116 | 0 | excluded=6, statuses=benchmark_normalized=55, comparable=12, not_normalized=3, parse_normalized=3, parser_fixture_validated=49 |
| Corpora | attention_required | all governed fastq and bam stage-tool pairs | 122 | 121 | 1 | corpora=8, assigned stages=50, statuses=asset:reference-index-assets=2, fixture:corpus-01-adna-bam-mini=7, fixture:corpus-01-adna-damage-mini=9, fixture:corpus-01-bam-mini=28, fixture:corpus-01-genotyping-mini=1, fixture:corpus-01-kinship-mini=2, fixture:corpus-01-mini=63, fixture:corpus-02-edna-mini=4, fixture:corpus-03-amplicon-mini=5, planner_only=1 |
| Assets | complete | asset-required benchmark-submission pairs | 20 | 20 | 0 | assigned=20, not_required=102 |
| Reports | complete | governed local report surfaces | 5 | 5 | 0 | expected_results=118, stage_sections=51, tool_sections=67, corpus_sections=8 |

## Report Outputs

| Report | Output | Governed items |
| --- | --- | --- |
| pair_readiness | benchmarks/readiness/pair-readiness.tsv | 122 stage_tool_pairs |
| expected_benchmark_results | benchmarks/readiness/expected-benchmark-results.tsv | 118 expected_results |
| stage_centric_report | benchmarks/readiness/stage-centric-report.md | 51 stage_sections |
| tool_centric_report | benchmarks/readiness/tool-centric-report.md | 67 tool_sections |
| corpus_centric_report | benchmarks/readiness/corpus-centric-report.md | 8 corpus_sections |

## Exact Blockers

| Domain | Stage | Tool | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.filter_low_complexity | dustmasker | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.filter_low_complexity | fastp | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.report_qc | multiqc | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.trim_reads | seqpurge | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
