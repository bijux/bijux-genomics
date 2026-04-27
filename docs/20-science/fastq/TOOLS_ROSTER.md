# FASTQ Tools Roster

## What
Supported FASTQ-stage backends from the current domain manifests.

## Why
Makes the supported roster explicit so review decisions do not have to be reverse-engineered from YAML and runtime allowlists.

## Non-goals
- Exhaustive survey of all possible external tools.
- Tool recommendations outside the currently supported contracts.

## Contracts
- Every tool listed here must appear in the corresponding FASTQ stage manifest.
- Tools omitted here are not currently supported at that stage, even if they exist elsewhere in the workspace.

| Stage | Supported tools | Rationale |
| --- | --- | --- |
| fastq.validate_reads | fastqvalidator, fastqc, fastq_scan, seqtk, fqtools | Structural validation before any mutating read transform |
| fastq.detect_adapters | fastqc | Report-only adapter evidence without mutating FASTQ |
| fastq.trim_polyg_tails | fastp, bbduk | PolyG/polyX artifact trimming for sequencer-tail cleanup |
| fastq.trim_reads | fastp, cutadapt, atropos, bbduk, adapterremoval, alientrimmer, fastx_clipper, leehom, trimmomatic, trim_galore, prinseq, seqkit, skewer | Adapter and quality trimming backends with governed runtime coverage and normalized stage outputs |
| fastq.filter_reads | fastp, seqkit, prinseq, bbduk | Quality/length/content filtering without stage overloading |
| fastq.filter_low_complexity | prinseq, bbduk | Dedicated low-complexity-capable backends admitted in the current runtime set |
| fastq.profile_read_lengths | seqkit_stats | Neutral read-length summaries |
| fastq.profile_reads | seqkit_stats | Deterministic baseline read statistics |
| fastq.profile_overrepresented_sequences | fastqc, seqkit | Overrepresented-sequence reporting |
| fastq.merge_pairs | pear, vsearch, bbmerge, flash2, leehom | Overlap-aware paired-end merging backends still supported by the runtime contract |
| fastq.remove_duplicates | fastuniq, clumpify | FASTQ-space duplicate removal without BAM-level duplicate marking |
| fastq.deplete_host | bowtie2 | Explicit mapping-based host depletion contract |
| fastq.deplete_reference_contaminants | bowtie2 | Reference-driven contaminant depletion |
| fastq.deplete_rrna | sortmerna | Read-level rRNA depletion |
| fastq.correct_errors | rcorrector, musket, lighter, bayeshammer | Intentionally supported error-correction backends after roster cleanup |
| fastq.extract_umis | umi_tools | UMI extraction with barcode-pattern-aware header propagation |
| fastq.screen_taxonomy | kraken2, krakenuniq, centrifuge, kaiju | Read-level screening and profiling backends only |
| fastq.report_qc | multiqc | Aggregated QC reporting |
| fastq.trim_terminal_damage | adapterremoval, cutadapt, seqkit | Terminal-damage-aware trimming/masking |
| fastq.normalize_primers | cutadapt | Primer normalization with explicit sequence handling |
| fastq.remove_chimeras | vsearch | Chimera removal in amplicon workflows |
| fastq.cluster_otus | vsearch | OTU clustering |
| fastq.infer_asvs | no admitted backend yet | Stage contract is defined, but governed runtime admission for ASV inference is still pending |
| fastq.normalize_abundance | seqkit | Post-inference abundance normalization helpers |
