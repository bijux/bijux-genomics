# Explain Output

`explain_vcf_plan` returns `PlannerExplainV1`, the planner-level explanation for a resolved VCF plan.

## Purpose
Explain output should make a plan reviewable without executing it. It records why stages and tools were selected and which reference context shaped the plan.

## Contents
- Planner version.
- Coverage regime.
- Pipeline domain.
- Stage list with tool, reason, command, inputs, outputs, params, and chunk count.
- Requested stages and stage tool overrides.
- Panel lock, panel selection, and species context.
- Reference bundle, panel, and map identifiers and checksums.

## Stability
Explain payload shape is part of the public review contract. Changes require snapshot review in `tests/contracts.rs` and documentation updates here.

## Non-Goals
- Runtime metrics.
- Tool stderr/stdout.
- Product-level QC interpretation.
- Environment discovery results.
