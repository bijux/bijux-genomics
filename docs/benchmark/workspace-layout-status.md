# Benchmark Workspace Layout Status

- Local results root: `<LOCAL_RESULTS_ROOT>`
- Local cache mirror root: `<LOCAL_CACHE_ROOT>`
- Mirrored remote workspace root: `<LOCAL_MIRRORED_REMOTE_WORKSPACE_ROOT>`
- Authoritative remote results root: `<LOCAL_CACHE_RESULTS_ROOT>`
- Authoritative remote reference root: `<LOCAL_CACHE_REFERENCE_ROOT>`
- Authoritative local publication root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01`
- Status: `clear`
- Issues: `0`

## Root Pairs

- `remote-results`: `clear` (canonical `<LOCAL_CACHE_RESULTS_ROOT>`, legacy `<LOCAL_CACHE_ROOT>/bijux-dna-results`)
  - canonical entries: `corpus_01`
- `remote-reference`: `clear` (canonical `<LOCAL_CACHE_REFERENCE_ROOT>`, legacy `<LOCAL_CACHE_ROOT>/bijux-reference`)
  - canonical entries: `contaminants`, `rrna`, `taxonomy`

## Local Stage Layout

- Archive corpus root: `<LOCAL_RESULTS_ROOT>/corpus_01`
- Cache corpus root: `<LOCAL_CACHE_RESULTS_ROOT>/corpus_01`
- Cache-only stage ids: `fastq.correct_errors`, `fastq.deplete_host`, `fastq.deplete_host.publish`, `fastq.deplete_reference_contaminants`, `fastq.deplete_rrna`, `fastq.detect_adapters`, `fastq.extract_umis`, `fastq.filter_low_complexity`, `fastq.filter_reads`, `fastq.merge_pairs`, `fastq.normalize_primers`, `fastq.profile_overrepresented_sequences`, `fastq.profile_read_lengths`, `fastq.profile_reads`, `fastq.remove_duplicates`, `fastq.report_qc`, `fastq.screen_taxonomy`, `fastq.trim_polyg_tails`, `fastq.trim_reads`, `fastq.trim_terminal_damage`, `fastq.validate_reads`

## Issues

