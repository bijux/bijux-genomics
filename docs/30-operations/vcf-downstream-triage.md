# VCF Downstream Triage

## Purpose
Defines a deterministic triage workflow for failed VCF downstream stages.

## Scope
Applies to stage-level failures across phasing, imputation, IBD, ROH, and demography runs.

## Non-goals
- Defining scientific acceptance thresholds; use QC contracts for those rules.

## Contracts
- Triage must start from stage artifacts and provenance manifests before reruns.
- Rerun decisions must account for params/tool/panel lock identity changes.

This guide is for failures in downstream VCF stages such as `vcf.phasing`, `vcf.impute`, `vcf.ibd`, `vcf.roh`, and `vcf.demography`.

## 1. Start With Stage Artifacts

Inspect the stage run directory:

- `run_artifacts/execution_record.json`
- `run_artifacts/tool_invocation.json`
- `run_artifacts/stage_report.json`
- `run_artifacts/telemetry/events.jsonl`
- `logs/stdout.log`
- `logs/stderr.log`

If `execution_record.json.exit_code != 0`, use `stderr.log` first for tool-level failure.
For `vcf.phasing` and `vcf.impute`, also inspect `crash_provenance.json` for:
- structured `error_category`
- `actionable_hint`
- `stderr_tail`
- `tool_digest`
- captured `cleanup_policy` from `BIJUX_STAGE_CLEANUP_POLICY`

## 2. Validate Provenance + Locks

Inspect:

- `run_manifest.json`
- `run_manifest.lock.json`
- `profile_manifest.json`
- `run_artifacts/reproducibility/report.json`

Check:

- `run_manifest.run_provenance.plan_hash` matches the planned graph hash.
- `run_manifest.execution_replay_identity.tool_image_digest` is present and pinned.
- downstream panel stages include panel lock metadata in stage params/provenance.

## 3. Common Failure Classes

1. Input contract violations:
- Symptoms: early failure with contract/invariant errors.
- Check sortedness, sample consistency, contig consistency, and required index sidecars.

2. Tool/runtime mismatch:
- Symptoms: image pulls/runs but command errors immediately.
- Check `tool_invocation.json` command template and resolved tool/image.

3. Missing downstream artifacts:
- Symptoms: stage exits `0` but manifest/invariant checks fail.
- Verify expected outputs exist and are non-empty: phased/imputed VCF, IBD/ROH tables, demography outputs.

4. Panel governance issues:
- Symptoms: phasing/imputation/panel-prepare stage rejects configuration.
- Verify panel id/build/checksum/license fields are complete and policy-compatible.

## 4. Fast Checks Per Stage

- `vcf.prepare_reference_panel`: panel lock fields + index sidecars.
- `vcf.phasing`: phased VCF exists, sample IDs unchanged.
- `vcf.imputation_metrics` / `vcf.impute`: INFO/Rsq-like metrics present, output indexed when required.
- `vcf.ibd`: segment table non-empty, minimum sample constraints satisfied.
- `vcf.roh`: ROH counts/segments emitted and parseable.
- `vcf.demography`: IBD-derived inputs present and non-empty.

## 5. When To Re-run

Re-run stage after fixing inputs/pins when:

- `params_hash` changed,
- `tool_image_digest` changed,
- or panel lock changed.

If all three are unchanged and artifacts already exist, treat as suspected nondeterminism and compare `run_manifest.lock.json` and `tool_invocation.json` between runs.

Cleanup policy defaults to `keep` for HPC frontend triage. Set `BIJUX_STAGE_CLEANUP_POLICY=prune` only when you explicitly want to remove temporary/chunk intermediates on failure.
