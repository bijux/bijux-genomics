# Certification Scope (Local, No HPC)

## What certification proves
- The local mini FASTQ/BAM/VCF domain stacks execute and produce expected artifact envelopes.
- Required schema keys are present for certification-critical artifacts.
- Golden drift is checked by schema and required keys, not exact metric values.
- Production-mode gating can escalate warnings to failures.

## What certification does not prove
- HPC-scale runtime behavior (scheduler, distributed IO, cluster contention).
- Throughput/latency claims at production-scale dataset sizes.
- Statistical validity beyond the defined local mini fixtures and examples.

## Modes
- `production`: warnings are treated as failures.
- `non_production`: warnings are allowed and the run is stamped as relaxed.

## Outputs
- `artifacts/certification/run_stamp.json`
- `artifacts/certification/certification_bundle.json`

