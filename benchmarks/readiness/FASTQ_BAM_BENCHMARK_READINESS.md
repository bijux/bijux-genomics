# FASTQ + BAM Benchmark Readiness Dashboard

## Summary

- Expected pairs: 123
- Ready pairs: 115
- Blocked pairs: 8
- Exact blocker counts: corpus=3, support=5

## Surface Summary

| Surface | Status | Scope | Total | Ready | Blocked | Evidence |
| --- | --- | --- | ---: | ---: | ---: | --- |
| Matrix | attention_required | all governed fastq and bam stage-tool pairs | 123 | 115 | 8 | stages=51, tools=67, gaps=corpus=3, none=115, support=5 |
| Adapters | attention_required | all governed fastq and bam stage-tool pairs | 123 | 118 | 5 | declared_only=5, runnable=118 |
| Parsers | complete | benchmark-reporting pairs only | 115 | 115 | 0 | excluded=8, statuses=benchmark_normalized=55, comparable=12, not_normalized=5, parse_normalized=2, parser_fixture_validated=49 |
| Corpora | attention_required | all governed fastq and bam stage-tool pairs | 123 | 120 | 3 | corpora=7, assigned stages=49, statuses=fixture:corpus-01-adna-bam-mini=7, fixture:corpus-01-adna-damage-mini=9, fixture:corpus-01-bam-mini=28, fixture:corpus-01-genotyping-mini=1, fixture:corpus-01-kinship-mini=2, fixture:corpus-01-mini=63, fixture:corpus-02-edna-mini=4, fixture:corpus-03-amplicon-mini=6, planner_only=3 |
| Assets | complete | asset-required benchmark-submission pairs | 18 | 18 | 0 | assigned=19, not_required=104 |
| Reports | complete | governed local report surfaces | 5 | 5 | 0 | expected_results=115, stage_sections=51, tool_sections=67, corpus_sections=7 |

## Report Outputs

| Report | Output | Governed items |
| --- | --- | --- |
| pair_readiness | benchmarks/readiness/pair-readiness.tsv | 123 stage_tool_pairs |
| expected_benchmark_results | benchmarks/readiness/expected-benchmark-results.tsv | 115 expected_results |
| stage_centric_report | benchmarks/readiness/stage-centric-report.md | 51 stage_sections |
| tool_centric_report | benchmarks/readiness/tool-centric-report.md | 67 tool_sections |
| corpus_centric_report | benchmarks/readiness/corpus-centric-report.md | 7 corpus_sections |

## Exact Blockers

| Domain | Stage | Tool | Gap | Support | Adapter | Parser | Corpus | Asset |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| fastq | fastq.estimate_library_complexity_prealign | bijux_dna | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.filter_low_complexity | dustmasker | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.filter_low_complexity | fastp | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
| fastq | fastq.index_reference | bowtie2_build | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | assigned |
| fastq | fastq.index_reference | star | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.normalize_abundance | seqfu | support | planned_contract | declared_only | not_normalized | fixture:corpus-03-amplicon-mini | not_required |
| fastq | fastq.report_qc | multiqc | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.trim_reads | seqpurge | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
