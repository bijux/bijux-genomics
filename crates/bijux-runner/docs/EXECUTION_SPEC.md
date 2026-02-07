# EXECUTION_SPEC

## CommandSpec hashing
Invocation hashing includes:
- Command binary and argv.
- Environment variables (after allowlist/filter).
- Working directory.
- Container image reference (if applicable).

Redactions apply to secrets before hashing.
