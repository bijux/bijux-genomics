# STYLE

This workspace uses explicit, intention-revealing module names and directory
boundaries. Avoid generic buckets like helpers/utils/misc. Prefer domain- or
responsibility-specific modules.

Rules of thumb:
- Prefer shallow, purposeful trees over flat sprawl.
- Do not hide public surface area; document it in SCOPE.md.
- Keep core contracts boring and stable.
- Tests should live under `tests/` with fixtures in `tests/fixtures/`.
