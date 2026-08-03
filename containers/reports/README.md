# Container evidence reports

This directory contains small, reviewed summaries of real container runs. Raw
images, logs, inputs, and run products remain in the governed execution site;
the repository records immutable identities and enough provenance to audit the
claim without copying large or restricted artifacts into Git.

A report must distinguish these claims:

- **runtime**: version, help, and invalid-input behavior passed;
- **stage**: the tool produced the declared output on non-fixture input;
- **science**: predefined scientific metrics passed;
- **benchmark**: comparable resource and quality measurements exist;
- **release**: all required evidence dimensions are complete.

Stage execution alone is not scientific validation or release qualification.
Failed scientific Slurm attempts remain visible in a separate execution-site
failure bundle keyed by stable run identity and Slurm job ID; they never enter
the final run-evidence index. A corrected attempt may retain the stable run
identity only after the failed submission and result records have been moved
non-destructively out of the canonical submission and result paths. The
complete-roster `run-1000` image campaign instead retains each build attempt in
one campaign record because it admits one final immutable image per tool.

## Reports

- [Canonical Apptainer storage reconciliation](image-storage.md)
- [Complete LUNARC Apptainer image rebuild](lunarc-image-rebuild.md)
- [LUNARC Apptainer image inventory](image-inventory.md)
- [Complete LUNARC run-evidence index](run-evidence.md)
- [Evidence-dimension qualification matrix](qualification.md)
- [Apptainer version and help verification](runtime-verification.md)
- [LUNARC ancient-DNA damage validation](lunarc-ancient-damage.md)
- [LUNARC ancient-DNA genotype-likelihood validation](lunarc-ancient-genotype-likelihoods.md)
- [LUNARC amplicon ASV validation](lunarc-amplicon-asv.md)
- [LUNARC BAM coverage validation](lunarc-bam-coverage.md)
- [LUNARC FASTQ-to-BAM validation](lunarc-fastq-bam.md)
- [LUNARC paired-read trimming validation](lunarc-paired-read-trimming.md)
- [LUNARC reference-complexity validation](lunarc-reference-complexity.md)
- [LUNARC VCF phasing validation](lunarc-vcf-phasing.md)
- [LUNARC VCF quality-control validation](lunarc-vcf-qc.md)
- [LUNARC VCF relatedness validation](lunarc-vcf-relatedness.md)
