# FASTQ Filter Policy Matrix

This page describes the governed scientific boundary for the FASTQ cleanup stages that remove reads or trim terminal bases.

## Governed Stages

- `fastq.filter_reads`: quality and ambiguous-base filtering removes whole reads or synchronized read pairs and changes `reads_removed_by_n`, `reads_removed_by_length`, `reads_dropped`, and `mean_q_after`.
- `fastq.filter_low_complexity`: low-complexity and poly-X filtering removes whole reads or synchronized read pairs and changes `reads_removed_low_complexity`, `bases_out`, and `pairs_out`.
- `fastq.trim_polyg_tails`: poly-G cleanup trims terminal bases rather than rejecting complete reads and changes `reads_out`, `bases_out`, and `mean_q_after`.
- `fastq.trim_terminal_damage`: terminal-damage cleanup trims terminal bases rather than rejecting complete reads and changes `reads_out`, `bases_out`, and `mean_q_after`.

## Scientific Caveats

- `fastq.filter_reads` can enrich for higher-quality fragments and should remain explicit when comparing empirical error or retention rates.
- `fastq.filter_low_complexity` can remove biologically real low-complexity molecules and requires platform-aware interpretation for poly-X thresholds.
- `fastq.trim_polyg_tails` targets sequencer artifacts, but it can also shorten authentic inserts and change downstream merging behavior.
- `fastq.trim_terminal_damage` is not neutral preprocessing for ancient or damaged material because it can erase terminal damage evidence.
