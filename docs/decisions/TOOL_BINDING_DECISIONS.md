# Tool Binding Decisions

This record captures tools whose binding shape spans multiple domains or differs between
`stage_ids` and `bindings`, and documents why the binding was moved/expanded.

- bowtie2:
  - decision: keep canonical role as `aligner` and allow use in FASTQ host depletion and BAM alignment.
  - reason: host depletion in FASTQ is alignment-driven; contaminant screening remains screen-tool based.
  - affected bindings: `bam.align`, `fastq.host_depletion`.
  - date: 2026-02-11.
