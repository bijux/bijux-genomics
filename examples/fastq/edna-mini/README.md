# fastq_edna_mini

## Purpose
Run a deterministic mini eDNA FASTQ path with primer/chimera/OTU/abundance stages enabled.

Canonical invocation: `./scripts/examples/run.sh fastq_edna_mini`

## Step 1 Containers
- Ensure image plan is resolved by the runner (`ensure-images --plan`).

## Step 2 Build/Verify
- Validate `example.toml` and `corpus-01-mini` availability.

## Step 3 Bench
- Execute ecology-oriented stages:
  - `fastq.primer_normalization`
  - `fastq.chimera_detection`
  - `fastq.otu_clustering`
  - `fastq.abundance_normalization`

## Step 4 Collect/Report
- Collect outputs under `artifacts/examples/fastq_edna_mini/`.
- Emit deterministic bundle and report artifacts.
