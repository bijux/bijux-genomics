# Report Specs Contract

Report specs may describe render intent, templates, and release-facing report
bundles.

Author only reviewable report intent here: output narratives, packaging rules,
and template-level decisions that should remain authored even when downstream
compilers or renderers materialize generated report fragments.

## Boundaries

- [README.md](README.md) records the current authored scope for report specs.
- [../evidence/README.md](../evidence/README.md) defines the authored evidence
  inputs that report specs summarize.
- [../results/README.md](../results/README.md) defines the authored result-plane
  inputs that report specs may render or package.
- [../releases/README.md](../releases/README.md) defines the authored release
  surface that may freeze report bundles beside evidence and results.
- [../../CONTRACT.md](../../CONTRACT.md) defines the root boundary that keeps
  authored report intent separate from generated outputs and local archives.
- [../../generated/README.md](../../generated/README.md) is the downstream
  compiled surface that must remain renderer- and compiler-owned.
- [../../README.md](../../README.md) defines the wider authored, generated, and
  local-archive split for the full science control surface.
