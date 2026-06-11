# Full Benchmark Dashboard

This dashboard is generated from governed machine-readable FASTQ, BAM, and VCF readiness outputs.

| metric | count | source path | source field | detail |
| --- | ---: | --- | --- | --- |
| total_stages | 71 | `benchmarks/readiness/all-domain-stage-list.json` | `total_stage_count` | governed all-domain local stage inventory |
| total_tools | 69 | `benchmarks/readiness/expected-benchmark-results-all-domains.tsv` | `tool_count` | unique tools across canonical all-domain expected benchmark jobs |
| total_expected_jobs | 130 | `benchmarks/readiness/expected-benchmark-results-all-domains.tsv` | `row_count` | canonical FASTQ, BAM, and VCF expected benchmark bindings |
| ready_jobs | 127 | `benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.json` | `present_row_count` | expected benchmark jobs with present governed result rows |
| blocked_jobs | 3 | `benchmarks/readiness/FASTQ_BAM_VCF_BENCHMARK_REPORT.json` | `missing_result_row_count` | expected benchmark jobs still visible as missing_result rows |
| missing_parsers | 0 | `benchmarks/readiness/parser-collector-all-domains.json` | `expected_result_ids - fake_run_result_ids` | canonical result ids without governed fake-run parser evidence |
| missing_adapters | 0 | `benchmarks/readiness/rendered-commands-all-domains.sh` | `expected_result_ids - rendered_command_result_ids` | canonical result ids without governed rendered command coverage |
| missing_assets | 0 | `benchmarks/readiness/all-domain-stage-tool-table.tsv` | `expected_bindings - benchmark_ready_asset_bindings` | canonical benchmark bindings without assigned asset-profile coverage |
| failed_real_smokes | 0 | `target/local-real-smoke/core-subset/REAL_SMOKE_SUMMARY.json` | `real_smoke_rows failing success contract` | governed real-smoke executions that do not satisfy their success contract |

Unsupported pairs tracked outside the expected-job slice: 1.
