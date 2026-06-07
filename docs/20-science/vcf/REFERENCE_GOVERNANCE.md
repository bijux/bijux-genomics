# VCF Reference Governance

## Purpose
Define the real reference-governance boundary for planned VCF downstream analysis without pretending the current stage manifests already declare stage-level reference banks.

## Scope
This document governs the panel-bound downstream family:
- `vcf.prepare_reference_panel`
- `vcf.phasing`
- `vcf.imputation_metrics`
- `vcf.impute`
- `vcf.postprocess`
- `vcf.qc`

## Non-goals
- Claiming that the VCF stage manifests already expose non-empty `bank_hooks`.
- Replacing the lower-level planner/runtime contract in `domain/vcf/docs/IMPUTATION_CONTRACT.md`.

## Contracts
- Current `domain/vcf/stages/*.yaml` entries still declare `bank_hooks: ["none"]`; stage-level bank governance is not yet promoted in the VCF manifest layer.
- Reference, panel, and map enforcement for the stages above currently lives at planner/runtime admission, not as best-effort tool flags.
- Use db-ref APIs (`resolve_reference_bundle`, `resolve_reference_bank`, `resolve_genetic_map_bank`) instead of direct path literals.
- Required references and maps must have lock-backed checksums and explicit `{species_id, build_id}` compatibility.
- Runs in this stage family must emit provenance that identifies the admitted bundle, panel, and map choices.

## Runtime Rules
- `vcf.prepare_reference_panel` refuses panel/build mismatches before any downstream method is admitted.
- `vcf.phasing` and `vcf.impute` must consume the same governed panel and map identity recorded at admission time.
- `vcf.imputation_metrics` keeps multi-tool admission explicit, but tool choice does not weaken bundle compatibility checks.
- `vcf.postprocess` and `vcf.qc` inherit the same governed build and panel provenance rather than inventing fresh reference identity.

## Failure modes
- Treating planner-level reference governance as optional would make panel/build mismatches look operationally valid.
- Hiding that `bank_hooks` are still unpromoted in stage manifests would create false confidence about where the refusal boundary actually lives.
