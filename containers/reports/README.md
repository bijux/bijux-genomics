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
Failed runs remain visible in the execution-site evidence and are replaced by
a new run identifier after the cause is corrected.

## Reports

- [LUNARC Apptainer image inventory](image-inventory.md)
- [Complete LUNARC run-evidence index](run-evidence.md)
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
