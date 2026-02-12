# Tool Binding Decisions

This record captures tools whose binding shape spans multiple domains or differs between
`stage_ids` and `bindings`, and documents why the binding was moved/expanded.

- bowtie2:
  - decision: keep canonical role as `aligner` and allow use in FASTQ host depletion and BAM alignment.
  - reason: host depletion in FASTQ is alignment-driven; contaminant screening remains screen-tool based.
  - affected bindings: `bam.align`, `fastq.host_depletion`.
  - date: 2026-02-11.
- angsd:
  - decision: keep binding in BAM authenticity/contamination analysis only.
  - reason: estimator semantics are specific to BAM-level damage/authenticity workflows.
- bamtools:
  - decision: keep BAM-only binding for BAM transform/metrics stages.
  - reason: utility is BAM-structural and not FASTQ-domain compatible.
- bbduk:
  - decision: allow FASTQ trim/filter roles through explicit stage bindings.
  - reason: same tool serves distinct semantics by stage contract.
- bedtools:
  - decision: keep BAM/coverage analytics binding only.
  - reason: interval operations are downstream of alignment outputs.
- fastp:
  - decision: keep FASTQ trim/filter/QC bindings and disallow BAM roles.
  - reason: algorithm and outputs are read-level preprocessing semantics.
- pmdtools:
  - decision: keep BAM authenticity/damage role binding.
  - reason: PMD signal interpretation is BAM-domain specific.
- samtools:
  - decision: use explicit multi-binding by stage role (prepare_reference, qc, metrics, transform).
  - reason: one binary legitimately spans multiple BAM/FASTQ support stages.
