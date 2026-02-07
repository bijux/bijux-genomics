# PATHS

Infra path helpers must defer to core canonicalization rules.
Do not implement separate path normalization here.

Any path canonicalization must call into `bijux-core`.
