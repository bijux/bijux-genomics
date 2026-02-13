# Config Schema Rules

## Version Bump Rules
- Bump `schema_version` when a config contract changes semantics, required keys, or validation logic.
- Patch-like content updates that do not change schema rules keep the same `schema_version`.
- New optional fields do not require a bump if old readers remain valid.

## Backward Compatibility Policy
- `N` readers must accept `N` and may accept `N-1` during migration windows.
- Removing a field or changing type/meaning is a breaking change and requires `N+1`.
- Writers must emit a single canonical version per file and avoid dual-format output.

## Deprecation Timeline
- Mark deprecated schema versions in release notes immediately after introducing `N+1`.
- Keep `N-1` readable for at most two minor releases.
- Remove `N-1` read support after the migration window and update contract tests in the same change.
