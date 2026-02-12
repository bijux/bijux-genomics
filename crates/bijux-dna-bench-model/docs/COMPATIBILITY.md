# COMPATIBILITY

## Expected metrics
The model expects benchmark observations derived from analyze/runtime metrics:
- runtime and memory metrics must be present for gating and summary statistics.
- stage and tool identifiers must be canonical and stable.

## Unknown metrics
Unknown metrics are preserved in observations but ignored by strict contract checks unless
explicitly referenced in policies. Gate policies may reject unknown metric ids.
