# `corpus-01` published result mirror status

- Published stages audited: `10`
- Complete mirrored stages: `7`
- Incomplete mirrored stages: `3`
- Mirror issues: `7`

## Stage status

- `fastq.validate_reads`: `incomplete` (`1` issues)
  - `report-tool-roster-drift`: sample_0001 observed ['fastqvalidator']; sample_0002 observed ['fastqvalidator']; sample_0003 observed ['fastqvalidator'] (+17 more)
- `fastq.detect_adapters`: `complete` (`0` issues)
- `fastq.profile_reads`: `complete` (`0` issues)
- `fastq.profile_read_lengths`: `complete` (`0` issues)
- `fastq.profile_overrepresented_sequences`: `incomplete` (`3` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_overrepresented_sequences/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc
  - `missing-local-run-root`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc/run_manifest.json
- `fastq.trim_polyg_tails`: `incomplete` (`3` issues)
  - `summary-run-root-drift`: summary run_root=/home/bijan/bijux/corpus_01/benchmarks/fastq.trim_polyg_tails/lunarc expected /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc
  - `missing-local-run-root`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc
  - `missing-stage-run-manifest`: missing /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc/run_manifest.json
- `fastq.trim_reads`: `complete` (`0` issues)
- `fastq.merge_pairs`: `complete` (`0` issues)
- `fastq.trim_terminal_damage`: `complete` (`0` issues)
- `fastq.report_qc`: `complete` (`0` issues)
