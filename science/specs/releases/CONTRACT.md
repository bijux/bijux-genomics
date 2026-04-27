# Release Specs Contract

Release manifests freeze which authored claims and generated outputs belong to a
science release.

Author only reviewable release intent here: bundle composition rules,
identifiers, and cut criteria that decide which authored surfaces and generated
outputs belong together in a named science release.

## Boundaries

- [README.md](README.md) records the current authored scope for release specs.
- [manifests/README.md](manifests/README.md) inventories the governed authored
  release manifests that may be cut.
- [../evidence/README.md](../evidence/README.md) defines the authored evidence
  inputs a release may freeze.
- [../reports/README.md](../reports/README.md) defines the authored report
  surfaces a release may bundle.
- [../results/README.md](../results/README.md) defines the authored result-plane
  intent a release may freeze beside evidence and reports.
- [../../CONTRACT.md](../../CONTRACT.md) defines the root boundary that keeps
  authored release intent separate from generated outputs and local archives.
- [../../generated/current/README.md](../../generated/current/README.md)
  describes the current generated snapshot a release may freeze.
- [../../generated/current/evidence/README.md](../../generated/current/evidence/README.md)
  inventories the row-level generated evidence that may be frozen.
- [../../generated/indexes/README.md](../../generated/indexes/README.md)
  describes the rolled-up generated indexes a release may freeze beside row
  outputs.
- [../../README.md](../../README.md) defines the wider authored, generated, and
  local-archive split for the full science control surface, including release
  outputs under `artifacts/science-releases/**`.
