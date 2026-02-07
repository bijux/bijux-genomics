# APPTAINER_PLAN

## Scope
Enable Apptainer execution for QA images.

## Constraints
- No network pulls by default
- Must support digest pinning

## Test strategy
- parity tests against docker outputs
- fixture-based validation

## Parity definition
Same manifest/report shapes and key metrics.
