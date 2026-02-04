# Partial Run Policy

Owner: bijux-engine

## Policy

- If a stage exits with a non-zero exit code, partial outputs are **purged** by default.
- To keep partial outputs for debugging, set `BIJUX_KEEP_PARTIAL=1` (or `true`).

## Rationale

Partial outputs can silently corrupt downstream metrics or be mistaken for valid artifacts. The
default behavior favors scientific correctness and reproducibility over convenience.

## Enforcement

- Engine purges stage outputs on non-zero exit by default.
- Telemetry records whether partial outputs were purged or kept.

## Tests

- `bijux-engine` tests verify that failed stages do not leave partial outputs unless explicitly
  allowed via `BIJUX_KEEP_PARTIAL`.
