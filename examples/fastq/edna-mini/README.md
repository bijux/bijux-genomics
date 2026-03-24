# fastq_edna_mini

## Purpose
Run a deterministic mini eDNA FASTQ path with primer/chimera/OTU/abundance stages enabled.

Canonical invocation: `cargo run -q -p bijux-dev-dna -- examples run run fastq_edna_mini`

## Step 1 Containers
- Ensure image plan is resolved by the runner (`ensure-images --plan`).

## Step 2 Build/Verify
- Validate `example.toml` and `corpus-01-mini` availability.
- Validate reference DB governance contract:
  - pinned DB identifier and version
  - checksum-locked provenance
  - marker compatibility declared before chimera/reference steps

## Step 3 Bench
- Execute ecology-oriented stages:
  - `fastq.normalize_primers`
  - `fastq.remove_chimeras`
  - `fastq.cluster_otus`
  - `fastq.normalize_abundance`

## Step 4 Collect/Report
- Collect outputs under artifacts/examples/fastq_edna_mini/.
- Emit deterministic bundle and report artifacts.
- Emit warnings in report for compositionality caveats and database-bias risk.
