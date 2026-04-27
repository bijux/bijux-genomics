# Metric Semantics (BAM)

## What
Defines the meaning, units, and interpretation constraints for BAM metrics.

## Why
Downstream decisions and reports assume shared metric semantics; ambiguity leads to inconsistent scoring.

## Non-goals
- Replacing the metric schema definitions.
- Inventing scientific BAM metrics that are not yet part of the governed BAM catalog.

## Contracts
- The governed BAM metric catalog lives in [../../../domain/bam/metrics.yaml](../../../domain/bam/metrics.yaml).
- Stage-level metric exposure lives in [STAGE_CATALOG.md](STAGE_CATALOG.md).
- The scientific interpretation boundary lives in [METHODOLOGICAL_INTENT.md](METHODOLOGICAL_INTENT.md)
  and [OPERATIONAL_CONTRACT.md](OPERATIONAL_CONTRACT.md).

## Examples
### runtime_s
- units: seconds
- meaning: wall-clock runtime for the governed stage execution
- limitation: operational telemetry only; not a biological signal

### memory_mb
- units: megabytes
- meaning: peak memory usage for the governed stage execution
- limitation: hardware- and implementation-sensitive operational telemetry

## Failure modes
- Treating runtime telemetry as biological evidence creates fake scientific comparability.
- Adding stage-specific scientific metrics here before they exist in the governed BAM metric catalog
  causes the doc surface to outrun the manifests.
