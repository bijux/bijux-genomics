# Haplotype services

`HaplotypeBenchmarkInputs::new` admits scoped stage templates and an explicit tool
binding registry. Each public capability exposes its `run_local_vcf_*_smoke`
service and persisted report types. Services retain original stage, tool, corpus,
panel, map, genotype and quality-field semantics.

The services accept an explicit repository/output root. Supported tool selection
is checked against the supplied binding registry and reference compatibility.
The CLI command adapters retain their existing report rendering and defaults.
