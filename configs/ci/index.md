# configs/ci

Purpose: CI contract configuration root.

Layout:
- `configs/ci/registry/`: tool/domain registries and lockfile.
- `configs/ci/stages/`: stage catalogs.
- `configs/ci/tools/`: required tools and image catalog.
- `configs/ci/params/`: parameter schema registries.

Rule: no config files are allowed directly under `configs/ci/` except this index.
