# `corpus-01` published result mirror status

- Published stages audited: `10`
- Complete mirrored stages: `3`
- Incomplete mirrored stages: `7`
- Mirror issues: `15`

## Stage status

- `fastq.validate_reads`: `incomplete` (`2` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.validate_reads/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.validate_reads/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.validate_reads/lunarc/run_manifest.json
- `fastq.detect_adapters`: `incomplete` (`2` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.detect_adapters/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.detect_adapters/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.detect_adapters/lunarc/run_manifest.json
- `fastq.profile_reads`: `incomplete` (`2` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_reads/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_reads/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_reads/lunarc/run_manifest.json
- `fastq.profile_read_lengths`: `incomplete` (`2` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_read_lengths/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_read_lengths/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_read_lengths/lunarc/run_manifest.json
- `fastq.profile_overrepresented_sequences`: `incomplete` (`3` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_overrepresented_sequences/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc
  - `missing-local-run-root`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc/run_manifest.json
- `fastq.trim_polyg_tails`: `incomplete` (`3` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.trim_polyg_tails/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc
  - `missing-local-run-root`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc/run_manifest.json
- `fastq.trim_reads`: `complete` (`0` issues)
- `fastq.merge_pairs`: `incomplete` (`1` issues)
  - `missing-localized-report-json`: 10 run rows do not resolve to a local report.json
- `fastq.trim_terminal_damage`: `complete` (`0` issues)
- `fastq.report_qc`: `complete` (`0` issues)
