# Scientific benchmark contracts

This directory owns the reviewed scientific policy that exists before a
benchmark is executed. It is separate from container contracts: container
contracts establish image identity and runtime confinement, while these files
define which real data, tools, measurements, truth strategies, and downstream
substitution probes make a stage comparison admissible.

`portfolio.toml` reserves one durable run identity for every retained FASTQ,
BAM, and VCF stage and binds each stage to its corpus and scientific profile
requirements. `stage-campaigns.toml` contains only executable campaigns whose
complete retained-tool closure, inputs, parameters, resource measurements,
quality metrics, interpretation, and limitations have been reviewed.

`run-registry.json` is the generated permanent namespace authority for
`run-2001` through `run-2071`. It records lifecycle, retained-tool closure,
scientific profile and corpus roles, interface contracts, supersession fields,
the immutable introduction commit, and the monotonic allocation frontier. Its
reviewed bytes are bound by `run-registry.sha256`; regenerate or check all
registry surfaces with `bijux-dna-dev tooling run benchmark-run-registry`.

`stage-universe.json` is generated from the authoritative BAM, FASTQ, and VCF
stage YAML contracts. It records an explicit benchmark disposition and rationale
for every stage and binds every `benchmark_required` stage to exactly one
permanent reservation in `portfolio.toml`; `benchmarks/reports/stage-universe.md`
is its generated review index.

`stage-interfaces.toml` closes an executable stage definition over its accepted
input contracts, canonical output, normalized benchmark result, complete tool
adapter set, crate-owned default provider, and downstream substitution probe.
A campaign without that interface remains blocked. The single
`config-pipeline-parameters.yml` records only explicit operator overrides;
fields such as input layout remain input-derived and schema identity remains
immutable. The interface declares those ownership boundaries and rejects an
override that crosses them. Every run must archive the override file and the
resolved `effective-parameters.json` separately so replay does not depend on
whichever defaults happen to exist later.

Run specifications and raw evidence do not belong here. Specifications are
published under the governed study result root, large execution evidence stays
under `STUDY/results/benchmarks/`, and small reviewed reports belong in
`benchmarks/reports/`.

`schemas/stage-coverage.schema.json` is generated from the
`bijux-dna-bench-model` coverage type. The crate-owned coverage workflow derives
retained-stage tool closure from the environment catalog and preserves
historical report identities as `stale`; only a passing report from the required
repository commit is counted as admitted.
`schemas/stage-universe.schema.json` is generated from the typed canonical-stage
registry and must drift whenever a stage is added, removed, or reclassified.

The checked `suite`, `corpus`, `bundle`, `observation`, `summary`, and `decision`
schemas are also generated from `bijux-dna-bench-model`. `GateDecision` is the
only persisted decision record; schema changes must originate in the model
crate and pass its generated-schema drift tests.

All benchmark model schemas are generated into `schemas/`. The separate
`benchmarks/schemas/` tree owns only domain-normalized metric contracts and must
not contain snapshots of model-owned records.
