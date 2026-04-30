# Explain Output

`explain_vcf_plan` returns `PlannerExplainV1`, the planner-level explanation for a resolved VCF plan.

## Purpose
Explain output should make a plan reviewable without executing it. It records why stages and tools were selected and which reference context shaped the plan.

## Contents
- Planner version.
- Coverage regime.
- Coverage resolution reasons and damage-aware policy.
- Stage list with tool, reason, contract-shaped params, artifact classes, and calling-mode
  overlays.
- Selected panel plus panel-boundary and phasing/imputation boundary contracts when the stage set
  requires them.
- Cohort validation, population guardrail, cohort-analysis, report-coverage, production-corpus,
  and scientific-drift contracts so downstream review has one governed surface.
- Reference bundle, panel, and map identifiers and checksums.
- Decision traces that explain backend, panel, map, chunking, coverage, and iteration-specific
  contract surfacing choices.

## Stability
Explain payload shape is part of the public review contract. Changes require snapshot review in `tests/contracts.rs` and documentation updates here.

## Non-Goals
- Runtime metrics.
- Tool stderr/stdout.
- Product-level scientific interpretation beyond the explicitly surfaced caveats and contracts.
- Environment discovery results.
