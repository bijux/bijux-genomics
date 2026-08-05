# bijux-dna-dev Contracts

This crate owns developer automation contracts, not production pipeline contracts.

## Documentation contract
- The crate root may contain only `README.md` as narrative documentation.
- All other crate documentation must live under `docs/`.
- `docs/COMMANDS.md` is the crate-level command inventory and must be updated with command catalog changes.

## Command contract
- Every executable command id must be registered under `src/catalog`.
- Command execution must route through `src/application` and `src/commands`; `cli/` remains parsing and reporting only.
- Command ids must be deterministic, unique within their command group, and stable unless a migration is documented.
- `containers run generate-image-inventory -- <repository-root> <study-root> <image-root> <build-evidence-root> <run-evidence-root> <scan-output> <publication-output>`
  delegates all image admission and selection to `bijux-dna-environment`, preserves every source
  root explicitly, and atomically refuses to replace either evidence document.
- Image-inventory scan and publication schemas are generated and checked through native container
  commands backed by the persisted Rust environment types.
- `containers run generate-image-inventory-report -- [<publication> <output>]` loads a validated
  publication and atomically renders Markdown; `check-image-inventory-report` rejects governed
  report drift.
- `containers run runtime-verification -- execute <repository-root> ...` requires the exact clean
  source checkout explicitly; the automation checkout never substitutes its own repository identity.

## Artifact contract
- Generated reports, logs, run products, and local command outputs default to the repository root `artifacts/` tree.
- Commands that intentionally update governed outputs outside `artifacts/` must name the owned output path in docs or command help.
- Temporary directories used by automation must be explicit and must not depend on user-specific home paths.

## Failure contract
- Validation commands fail with clear nonzero outcomes when drift is found.
- External command failures must keep stdout and stderr available in the typed outcome.
- Network-capable workflows must disclose the command surface that owns network access and must keep offline failures explicit.
