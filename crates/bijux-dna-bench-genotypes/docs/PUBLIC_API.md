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
