# `corpus-01` published result mirror status

- Governed publication stages: `20`
- Published stages audited: `19`
- Complete mirrored stages: `18`
- Incomplete mirrored stages: `2`
- Mirror issues: `4`

## Stage status

- `fastq.validate_reads`: `complete` (`0` issues)
- `fastq.detect_adapters`: `complete` (`0` issues)
- `fastq.profile_reads`: `complete` (`0` issues)
- `fastq.profile_read_lengths`: `complete` (`0` issues)
- `fastq.profile_overrepresented_sequences`: `complete` (`0` issues)
- `fastq.normalize_primers`: `complete` (`0` issues)
- `fastq.trim_polyg_tails`: `complete` (`0` issues)
- `fastq.trim_reads`: `incomplete` (`3` issues)
  - `duplicate-result-root-ambiguity`: both /Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01/fastq.trim_reads/lunarc and /Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_reads/lunarc exist
  - `run-manifest-tool-roster-drift`: run_manifest tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `report-tool-roster-drift`: sample_0001 observed ['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic']; sample_0002 observed ['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic']; sample_0003 observed ['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] (+17 more)
- `fastq.filter_reads`: `complete` (`0` issues)
- `fastq.filter_low_complexity`: `complete` (`0` issues)
- `fastq.deplete_rrna`: `complete` (`0` issues)
- `fastq.merge_pairs`: `complete` (`0` issues)
- `fastq.remove_duplicates`: `complete` (`0` issues)
- `fastq.deplete_host`: `complete` (`0` issues)
- `fastq.deplete_reference_contaminants`: `complete` (`0` issues)
- `fastq.correct_errors`: `incomplete` (`1` issues)
  - `missing-published-summary`: missing /Users/bijan/bijux/bijux-dna/docs/benchmark/fastq.correct_errors/corpus-01/summary.json
- `fastq.extract_umis`: `complete` (`0` issues)
- `fastq.screen_taxonomy`: `complete` (`0` issues)
- `fastq.trim_terminal_damage`: `complete` (`0` issues)
- `fastq.report_qc`: `complete` (`0` issues)
