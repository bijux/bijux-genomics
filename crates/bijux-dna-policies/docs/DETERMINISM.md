# Determinism

Policy checks must produce stable results for identical repository state.

## Deterministic Inputs
- Repository paths are traversed in stable order before observable comparisons.
- Cargo manifests and governed config files are parsed as structured data.
- Snapshots and fixtures are read from repository-owned paths.
- Policy diagnostics are rendered through the same WHAT/WHY/HOW/MORE contract.

## Stable Outputs
- Dependency and docs inventories compare sorted sets.
- Snapshots are updated only when an intentional policy or documentation contract changes.
- Test names are stable search handles for failing rules.

## Forbidden Nondeterminism
- Wall-clock decisions.
- Random identifiers.
- Environment-specific path assumptions.
- Network lookup.
- Process execution from production source.
- Iteration over unordered maps when output order is visible.
