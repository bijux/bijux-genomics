# FASTQ + BAM Benchmark Readiness Dashboard

## Summary

- Expected pairs: 123
- Ready pairs: 112
- Blocked pairs: 11
- Exact blocker counts: corpus=6, support=5

## Surface Summary

| Surface | Status | Scope | Total | Ready | Blocked | Evidence |
| --- | --- | --- | ---: | ---: | ---: | --- |
| Matrix | attention_required | all governed fastq and bam stage-tool pairs | 123 | 112 | 11 | stages=51, tools=67, gaps=corpus=6, none=112, support=5 |
| Adapters | attention_required | all governed fastq and bam stage-tool pairs | 123 | 118 | 5 | declared_only=5, runnable=118 |
| Parsers | complete | benchmark-reporting pairs only | 112 | 112 | 0 | excluded=11, statuses=benchmark_normalized=55, comparable=12, not_normalized=5, parse_normalized=2, parser_fixture_validated=49 |
| Corpora | attention_required | all governed fastq and bam stage-tool pairs | 123 | 117 | 6 | corpora=7, assigned stages=48, statuses=fixture:corpus-01-adna-bam-mini=7, fixture:corpus-01-adna-damage-mini=9, fixture:corpus-01-bam-mini=28, fixture:corpus-01-genotyping-mini=1, fixture:corpus-01-kinship-mini=2, fixture:corpus-01-mini=60, fixture:corpus-02-edna-mini=4, fixture:corpus-03-amplicon-mini=6, planner_only=6 |
| Assets | complete | asset-required benchmark-submission pairs | 18 | 18 | 0 | assigned=19, not_required=104 |
| Reports | complete | governed local report surfaces | 5 | 5 | 0 | expected_results=112, stage_sections=51, tool_sections=67, corpus_sections=7 |

## Report Outputs

| Report | Output | Governed items |
| --- | --- | --- |
| pair_readiness | benchmarks/readiness/pair-readiness.tsv | 123 stage_tool_pairs |
| expected_benchmark_results | benchmarks/readiness/expected-benchmark-results.tsv | 112 expected_results |
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
| fastq | fastq.profile_overrepresented_sequences | fastq_scan | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.profile_overrepresented_sequences | fastqc | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.profile_overrepresented_sequences | seqkit | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.report_qc | multiqc | corpus | observer_specialized_benchmark | runnable | comparable | planner_only | not_required |
| fastq | fastq.trim_reads | seqpurge | support | planned_contract | declared_only | not_normalized | fixture:corpus-01-mini | not_required |
