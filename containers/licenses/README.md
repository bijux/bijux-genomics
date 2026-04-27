# Container License Assertions

`containers/licenses/` stores the per-tool `*.license.toml` assertions that
support container admission and review.

## Purpose

- keep license and source assertions adjacent to the governed container surface
- give science and promotion review one stable directory for per-tool license
  records
- keep legal review inputs separate from generated locks, smoke outputs, and
  runtime artifacts

## Governing Surfaces

- [../README.md](../README.md)
  repository-level container contract entrypoint
- [../index.md](../index.md)
  authoritative container operations and lifecycle index
- [../docs/VERSION_AUTHORITY.md](../docs/VERSION_AUTHORITY.md)
  authority order for version pins and source provenance
- [../docs/SCIENCE_EVIDENCE_BOUNDARY.md](../docs/SCIENCE_EVIDENCE_BOUNDARY.md)
  review boundary between runtime proof and scientific closure

Keep the per-tool `*.license.toml` files tracked. Do not replace this directory
with ad hoc notes elsewhere under `containers/`.
