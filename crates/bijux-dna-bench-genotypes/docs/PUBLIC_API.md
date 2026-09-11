# Public genotype benchmark services

`GenotypeBenchmarkInputs::new` admits scoped `VcfStageMatrixRow` declarations. Each
family exports `run_local_vcf_*_smoke`, receiving admitted inputs, an explicit
repository root and requested tool identity, and returning its compatible typed report.

The owner exports no CLI argument types and performs no rendering. Existing report
schema versions, field names, default paths, metrics and stage-result output IDs
remain compatible. Native evidence resides in the destination's `executions` store;
the stable publication inventory is `genotype-artifact-ownership.json` with schema
`bijux.bench.genotype_artifact_ownership.v1`.

Admission, execution, semantic validation and publication errors propagate through
`anyhow::Result`. Unknown declarations and missing native inputs fail visibly.
Success is not proof of tool substitution, remote qualification or CI performance.

## Calling and likelihood readiness

`readiness::{call, call_gl, call_diploid, call_pseudohaploid, gl_propagation}` owns
stage readiness assessment and publication beside each native smoke service. The
caller supplies admitted genotype inputs, an immutable `VcfEvidenceSnapshot`,
command publication paths, repository root and output path. Snapshot agreement with
the stage's corpus, asset, adapter, parser and schema is checked before native work.
Command readiness comes from snapshot observations; `ReadinessCommandPaths` supplies
only their publication locations.

Each service preserves its stage-specific retained/active selection, required
metrics, smoke predicates and serialized report fields. Coverage policy, flat wire
projection and filesystem publication are separate modules. Missing smoke evidence
remains an incomplete surface, and incomplete reports are published before the
service returns failure. These local checks do not establish remote readiness or
scientific tool interchangeability.
