# Benchmark Workspace Layout Status

- Local results root: `/Users/bijan/bijux/bijux-dna-results`
- Local cache mirror root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache`
- Mirrored remote workspace root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24`
- Authoritative remote results root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results`
- Authoritative remote reference root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/reference`
- Authoritative local publication root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01`
- Status: `incomplete`
- Issues: `1`

## Root Pairs

- `remote-results`: `duplicate` (canonical `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results`, legacy `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/bijux-dna-results`)
  - canonical entries: `corpus_01`
  - legacy entries: `corpus_01`
  - shared entries: `corpus_01`
- `remote-reference`: `clear` (canonical `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/reference`, legacy `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/bijux-reference`)

## Local Stage Layout

- Archive corpus root: `/Users/bijan/bijux/bijux-dna-results/corpus_01`
- Cache corpus root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01`
- Shared stage ids: `fastq.trim_reads`
- Archive-only stage ids: `fastq.deplete_host`, `fastq.deplete_host.publish`, `fastq.deplete_reference_contaminants`, `fastq.deplete_rrna`, `fastq.detect_adapters`, `fastq.extract_umis`, `fastq.filter_low_complexity`, `fastq.filter_reads`, `fastq.merge_pairs`, `fastq.normalize_primers`, `fastq.profile_overrepresented_sequences`, `fastq.profile_read_lengths`, `fastq.profile_reads`, `fastq.remove_duplicates`, `fastq.report_qc`, `fastq.trim_polyg_tails`, `fastq.trim_terminal_damage`, `fastq.validate_reads`
- Cache-only stage ids: `fastq.correct_errors`, `fastq.screen_taxonomy`

## Issues

- `duplicate-remote-results-root`: both /Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results and /Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/bijux-dna-results exist
