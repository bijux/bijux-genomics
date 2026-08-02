# LUNARC amplicon ASV validation

This report records DADA2 1.30.0 inference on two public, non-fixture amplicon
samples: [single-end ITS run DRR037766 from ENA](https://www.ebi.ac.uk/ena/browser/view/DRR037766)
and an overlapping paired-end 16S V4 sample from the
[Mothur MiSeq SOP](https://mothur.org/wiki/miseq_sop/). The same immutable
Apptainer image executed version, help, invalid-input, and real-data checks
without runtime network access.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-amplicon-asv.json).
Raw manifests, logs, inputs, outputs, failed-run evidence, and Slurm accounting
remain under the governed LUNARC study root.

## Passing execution

Single-end run `run-1058` processed ENA run `DRR037766` in 21 seconds. It
retained 16,327 of 16,337 reads and emitted 54 non-chimeric ASVs representing
15,594 reads.

Paired-end run `run-1060` processed sample `F3D0` from the public MiSeq SOP in
23 seconds. The 2×250 reads target the approximately 253-base 16S V4 region and
therefore have the overlap required by the merge contract. DADA2 retained
5,592 of 7,793 input pairs, merged 4,741 pairs, and emitted 85 non-chimeric
ASVs.

Both runs emitted the governed feature table, representative-sequence FASTA,
taxonomy-ready FASTA and FASTQ, and metrics report. The table row counts and
FASTA record counts agree with the reported ASV counts.

## Preserved failures

Runs `run-1056` and `run-1057` exposed a missing packaged runtime dependency:
RcppParallel requires `dpkg-architecture` when its namespace loads. The image
was rebuilt from the frozen Ubuntu snapshot with the exact `dpkg-dev` package,
and the image test now loads DADA2 rather than checking its version alone.

Run `run-1059` then rejected the first paired ENA candidate because none of its
reads overlapped. That is a corpus-to-method incompatibility, not evidence for
concatenating reads or weakening the merge rule. The failed run remains in the
evidence tree, and the replacement corpus records why overlap is scientifically
expected.

## Claim boundary

These executions prove the container runtime and `fastq.infer_asvs` output
contract for one single-end and one overlapping paired-end sample. They do not
establish taxonomy accuracy, cross-sample pooling behavior, ecological
interpretation, sensitivity across primer sets, or benchmark superiority.
Those claims require reference databases, mock-community truth, predefined
accuracy measures, and representative multi-sample campaigns.
