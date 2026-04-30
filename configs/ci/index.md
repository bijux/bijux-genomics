# configs/ci

Purpose: CI contract configuration root.

Layout:
- `artifacts/planning/`: backlog cards, scoreboard, and issue-routing labels.
- `configs/ci/compatibility/`: compatibility release inputs and deprecation dashboard sources.
- `configs/ci/registry/`: tool/domain registries and lockfile.
- `configs/ci/stages/`: stage catalogs.
- `configs/ci/tools/`: required tools and image catalog.
- `configs/ci/params/`: parameter schema registries.
- `configs/ci/lints/`: clippy allowlist contracts.

Rule:
- Keep governed config collections in named subdirectories with their own `index.md`.
- The root-level `*.toml` files in `configs/ci/` remain legacy governed surfaces and must not grow workflow truth that belongs in the subdirectories.
