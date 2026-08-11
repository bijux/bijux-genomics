# Scientific benchmark contracts

This directory owns the reviewed scientific policy that exists before a
benchmark is executed. It is separate from container contracts: container
contracts establish image identity and runtime confinement, while these files
define which real data, tools, measurements, truth strategies, and downstream
substitution probes make a stage comparison admissible.

`portfolio.toml` reserves one durable run identity for every runnable FASTQ,
BAM, and VCF stage and binds each stage to its corpus and scientific profile
requirements. Runnable closure includes retained packages and licensed tools
available through governed external-artifact admission. `stage-campaigns.toml`
contains only executable campaigns whose complete tool closure, inputs,
parameters, resource measurements, quality metrics, interpretation, and
limitations have been reviewed.

`data-fitness.toml` is the machine-evaluable corpus fitness policy for every
scientific profile in `portfolio.toml`. It sets minimum real-sample, case,
study, and input-scale requirements; requires representative layouts, truth,
controls, contamination or damage treatment, ancestry treatment, access
identity, bias assessment, and limitations; and permits a reduced minimum only
through an explicit, claim-restricted scientific exception. Checked-in
planning and governed data audits reject drift between the rubric and the
portfolio profile universe.

`run-registry.json` is the generated permanent namespace authority for
`run-2001` through `run-2071`. It records lifecycle, retained-tool closure,
scientific profile and corpus roles, interface contracts, supersession fields,
the immutable introduction commit, and the monotonic allocation frontier. Its
reviewed bytes are bound by `run-registry.sha256`; regenerate or check all
registry surfaces with `bijux-dna-dev tooling run benchmark-run-registry`.
`schemas/run-index.schema.json` is generated from the benchmark-model run-index
authority. The corresponding `benchmarks/runs/RUN_INDEX.json` publication adds
stable citation keys and selected-publication closure without treating a
registered protocol as executed evidence.

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
runnable-stage tool closure from the environment catalog and preserves
historical report identities as `stale`; only a passing report from the required
repository commit is counted as admitted.
`schemas/stage-universe.schema.json` is generated from the typed canonical-stage
registry and must drift whenever a stage is added, removed, or reclassified.

`schemas/fastq-corpus-source.schema.json` is generated from the model-owned
production FASTQ source contract. A source declaration records its production
class, archive authority, access and redistribution terms, attribution,
scientific truth limits, and bounded storage and compute projection before
live metadata resolution or materialization.

`schemas/corpus-acquisition.schema.json` and
`schemas/governed-data-catalog.schema.json` preserve that same source policy in
v2 acquisition and admission records. Historical v1 catalogs remain readable,
but new materialization cannot reconstruct or override policy with local
hardcoded values.

`schemas/governed-data-catalog-registry.schema.json` selects current immutable
catalog snapshots without replacing historical authorities. Snapshot paths are
source-addressed, exact hashes are mandatory, and only a v2 snapshot can be
selected for a newly derived production data index.

The checked `suite`, `corpus`, `bundle`, `observation`, `summary`, and `decision`
schemas are also generated from `bijux-dna-bench-model`. `GateDecision` is the
only persisted decision record; schema changes must originate in the model
crate and pass its generated-schema drift tests.

`schemas/cycle-preparation.schema.json` and
`schemas/cycle-admission.schema.json` define the persisted results of one
non-barrier lifecycle pass. They preserve every independent attempt outcome
without making an aggregate campaign result a scheduling barrier.

`schemas/benchmark-matrix-report.schema.json` defines the persisted readiness
matrix consumed by campaign planning and appraisal. The model crate owns its
version, row and summary invariants; command surfaces only adapt inputs and
render the validated record.

`schemas/appraisal-report.schema.json` and
`schemas/hardening-queue-report.schema.json` define the admitted findings and
deterministic remediation queue derived from that matrix. The benchmark model
owns their versions, vocabulary, ordering, identity, and exact aggregate
closure; the benchmark workflow owns derivation and atomic publication.

`schemas/cross-benchmark-campaign-report.schema.json` defines the persisted
cross-domain goal selection, scenario and matrix-row references, appraisal and
hardening subsets, and exact campaign summary. The benchmark model owns its
version and admission rules; command surfaces only adapt arguments and render
validated records.

`schemas/governed-data-catalog-snapshot-publication.schema.json`,
`schemas/governed-data-catalog-registry-publication.schema.json`, and
`schemas/campaign-report-publication.schema.json` define the versioned outcomes
of catalog preservation, catalog admission, and campaign report publication.
The benchmark workflow owns filesystem effects; the model crate owns their
persisted identities and admission rules.

All benchmark model schemas are generated into `schemas/`. The separate
`benchmarks/schemas/` tree owns only domain-normalized metric contracts and must
not contain snapshots of model-owned records.
