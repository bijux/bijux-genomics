# Haplotype services

`HaplotypeBenchmarkInputs::new` admits scoped stage templates and an explicit tool
binding registry. Each public capability exposes its `run_local_vcf_*_smoke`
service and persisted report types. Services retain original stage, tool, corpus,
panel, map, genotype and quality-field semantics.

The services accept an explicit repository/output root. Supported tool selection
is checked against the supplied binding registry and reference compatibility.
The CLI command adapters retain their existing report rendering and defaults.

## Reference-panel and imputation-metrics readiness

`readiness::{reference_panel, imputation_metrics}` owns observed readiness,
scientific assessment, smoke orchestration and compatible report publication.
Services receive `HaplotypeBenchmarkInputs`, an immutable `VcfEvidenceSnapshot`,
command publication addresses and explicit repository/report paths. Stage templates,
requested tool bindings, corpus, asset, adapter, parser and normalized schema must
agree before native execution. CLI code acquires declarations and observations.

Each stage separates wire models, field projection, scientific coverage predicates,
report contracts and effectful orchestration. Reference-panel readiness requires
complete governed coverage before replacing a report. Imputation metrics preserves
incomplete diagnostic publication followed by an error. Addresses remain declared
provenance; they do not by themselves prove command execution or scientific validity.
