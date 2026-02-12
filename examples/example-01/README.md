# Example 01

`example-01` is the first reproducible ENA-to-benchmark workflow example.

- Source project: `PRJEB44430`
- Selection policy: exactly 10 single-end (SE) + 10 paired-end (PE) runs
- Stage-1 benchmark: `fastq.validate_pre` only
- Runtime: `apptainer`

Produced artifacts:
- ENA snapshot with accept/reject reasons: `ENA_METADATA.snapshot.json`
- Downloaded raw FASTQ files under `raw/`
- Normalized corpus + checksum manifest (`MANIFEST.json`)
- Benchmark run manifest + telemetry + metrics
- Analysis report (`json` + `html`)

The stage-1 scope is intentionally narrow so we can validate policy, corpus contracts, and HPC wiring before expanding to more stages.
