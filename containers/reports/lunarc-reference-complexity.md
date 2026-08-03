# LUNARC reference-complexity validation

This report records real reference preparation on the first 1,000,000 bases
of EquCab3.0 chromosome 1 (`NC_009144.3`). The source is a governed,
checksum-pinned subset of the NCBI assembly rather than a fixture sequence.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-reference-complexity.json).
Raw manifests, logs, outputs, and Slurm accounting remain under the governed
LUNARC study root.

## Passing execution

SAMtools run `run-1054` extracted the interval with `faidx` in 5 seconds and
recorded its source FASTA and index identities. DustMasker run `run-1055`
completed in 1 second, emitted 969 low-complexity intervals, and soft-masked
37,734 of the 1,000,000 input bases. Version, help, invalid-input, image,
input, output, and Slurm accounting contracts passed.

Bowtie 2 Build run `run-1100` indexed the same public reference segment in 2
seconds and emitted all six required `.bt2` shards. Every shard is non-empty
and digest-pinned, and the dependent alignment run cites each shard as an
upstream result instead of inferring their lineage from a shared prefix.

## Claim boundary

This execution proves the reference low-complexity stage and the container's
runtime behavior. It also proves that Bowtie 2 can build a complete index for
a real EquCab3.0 segment. A single chromosome interval is not a whole-assembly
complexity or indexing benchmark, a comparison of masking parameters, or
evidence that the default DUST threshold is optimal for every downstream
analysis. Those claims require a governed benchmark design and representative
assemblies.
