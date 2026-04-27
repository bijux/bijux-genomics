# Certification Scope (Local, No HPC)

## Purpose
Define what the local certification gate proves before any HPC-specific execution claims are made.

## Scope
Applies to local mini-stack certification of FASTQ, BAM, and VCF example workflows.

## Non-goals
- HPC-scale runtime behavior, scheduler behavior, or shared-filesystem contention.
- Scientific validity claims beyond the committed local mini fixtures and examples.

## Contracts
- Local certification depends on the mini-stack validator defined in
  [FRONTEND_MINI_STACK_VALIDATION.md](FRONTEND_MINI_STACK_VALIDATION.md).
- Certified runs must emit the governed artifact envelope defined in
  [RUN_ARTIFACTS.md](RUN_ARTIFACTS.md).
- Example-driven certification must keep the example output bundle within the governed surface in
  [EXAMPLE_RUNNER_CONTRACT.md](../50-reference/EXAMPLE_RUNNER_CONTRACT.md).

## What Certification Proves
- The local mini FASTQ, BAM, and VCF domain stacks execute and produce expected artifact
  envelopes.
- Required schema keys are present for certification-critical artifacts.
- Golden drift is checked by schema and required keys, not exact metric values.
- Production-mode gating can escalate warnings to failures.

## What Certification Does Not Prove
- HPC-scale runtime behavior.
- Throughput or latency claims at production-scale dataset sizes.
- Statistical validity beyond the defined local mini fixtures and examples.

## Modes
- `production`: warnings are treated as failures.
- `non_production`: warnings are allowed and the run is stamped as relaxed.

## Outputs
- `artifacts/certification/run_stamp.json`
- `artifacts/certification/certification_bundle.json`

## Failure modes
- Certification runs that skip mini-stack validation provide no governed readiness signal.
- Missing artifact-envelope outputs invalidate the certification bundle.
