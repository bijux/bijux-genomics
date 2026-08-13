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

`schemas/active-scope-fast-validation.schema.json` defines the persisted
closure of the fast benchmark-surface validation. The benchmark model owns its
version, check identity, and exact category and outcome summaries; the command
surface renders and atomically publishes only a validated record.

`schemas/local-runtime-metrics.schema.json` defines the persisted projection of
validated local stage-result runtimes. The benchmark model owns readiness and
runtime field vocabulary, cardinality and identity validation, and schema
generation; the CLI only collects inputs and publishes the validated record.

`schemas/local-stage-manifest-completion.schema.json` and
`schemas/local-stage-output-completion.schema.json` define the independent
manifest-presence and declared-output completeness records. The benchmark model
owns their identities, rows, counters, completion invariants, and schema
generation; the CLI only observes the filesystem and publishes admitted records.

`schemas/local-stage-fake-runs.schema.json`,
`schemas/local-stage-fake-failures.schema.json`, and
`schemas/local-stage-fake-failure-record.schema.json` define disposable local
stage execution simulations and their persisted failure evidence. The
benchmark model owns record and schema identities, stage and artifact
cardinality, successful-output closure, non-zero failure semantics, governed
root containment, and semantic admission; the CLI only materializes simulated
outputs and writes admitted records below the disposable run root.

`schemas/local-essential-pipeline-fake-runs.schema.json`,
`schemas/local-essential-pipeline-fake-run-pipeline.schema.json`, and
`schemas/local-essential-pipeline-fake-run-node-metrics.schema.json` define
disposable complete-pipeline execution simulations. The benchmark model owns
their portfolio, topology, node, binding-lineage, counter, timing, output, and
governed-path invariants; the CLI only resolves declared pipeline commands and
materializes admitted simulation evidence.

`schemas/all-domain-failure-classification.schema.json` defines the exhaustive
failure taxonomy consumed by aggregate benchmark reports. The benchmark model
owns the eight required classes, exact class and trigger counts, missing-class
closure, schema identity, and admission verdict; the CLI only executes the
governed probes and supplies their evidence paths and observations.

`schemas/all-domain-expected-benchmark-results.schema.json` defines the
canonical expected-result inventory consumed by lifecycle, failure,
completion, scheduler, dashboard, and report surfaces. The benchmark model
owns row identity, aggregate closure, schema generation, and admission; the
CLI derives the inventory from governed stage, tool, corpus, output, metric,
and report-section authorities.

`schemas/stage-tool-resources.schema.json` and
`schemas/stage-tool-resources-report.schema.json` define the canonical
resource configuration and its readiness projection. The benchmark model owns
stage-tool identity, positive resource bounds, deterministic ordering,
aggregate closure, schema generation, and admission.

`schemas/full-benchmark-report.schema.json` defines the persisted aggregate
FASTQ, BAM, and VCF report, including stage, tool, corpus, pipeline, runtime,
memory, failure, missing-result, comparable-metric, and unsupported-pair
sections. The benchmark model owns its records, generated schema, source-count
closure, identity and path invariants, and admission verdict; the CLI collects
the governed inputs and materializes JSON and Markdown presentations.

`schemas/full-benchmark-result-collector.schema.json` defines the canonical
row-level union of expected, simulated, real-smoke, missing-result,
insufficient-data, and unsupported-pair evidence consumed by aggregate reports.
The benchmark model owns its records, generated schema, source/status/domain
closure, binding distinctions, safe evidence paths, and admission verdict; the
CLI only collects governed source surfaces and materializes the admitted JSON.

`schemas/scientific-acceptance-thresholds.schema.json` and
`schemas/scientific-acceptance-thresholds-report.schema.json` define the
persisted scientific comparison policy and its readiness projection. The
benchmark model owns metric identity, direction, tolerance, pass-rule,
insufficiency behavior, aggregate closure, schema generation, and admission;
the CLI only derives rows from reviewed comparable-metric contracts and writes
validated presentations.

`schemas/local-stage-inventory.schema.json` and
`schemas/local-all-domain-stage-inventory.schema.json` define the domain-local
and aggregate local benchmark stage inventories. The benchmark model owns
domain membership, stage identity, cardinality, and exact domain-count closure;
the CLI only loads reviewed matrices and publishes admitted records.

`schemas/local-corpus-skip-report.schema.json` defines corpus incompatibility,
replacement lineage, asset-backed stages, and planner-only dispositions. The
benchmark model owns corpus references, distinct stage coverage, exact counters,
and schema generation; the CLI only derives and publishes a validated report.

`schemas/local-corpus-stage-compatibility.schema.json` defines validated fixture
lineage and the fixture-backed, asset-backed, and planner-only disposition of
each local stage. The benchmark model owns the persisted rows, counters,
lineage validation, and schema generation; the CLI only loads the reviewed
matrix and publishes an admitted report.

`schemas/local-hpc-job-graph.schema.json` defines the persisted local HPC
benchmark topology. The benchmark model owns job-kind identity, reciprocal edge
closure, root derivation, topological ordering, and exact counters; the CLI only
constructs the graph from selected jobs and admits it before use.

`schemas/local-hpc-asset-staging.schema.json` defines persisted source and
staged asset identity for each selected local HPC job. The benchmark model owns
source kinds, checksums, shared-source consistency, consumer lineage,
repository containment, canonical ordering, and exact counters; the CLI only
discovers repository inputs and assembles the validated manifest.

`schemas/local-hpc-candidate-run-manifest.schema.json` defines the persisted
bounded candidate subset chosen for an initial HPC execution. The benchmark
model owns representative identity, resource and input ceilings, stop
conditions, behavior proofs, canonical ordering, exact counters, and schema
generation; the CLI only derives candidates from admitted planning inputs and
publishes the validated manifest.

`schemas/local-hpc-execution-resolver.schema.json` defines persisted selected
tool scope and its admitted host, Apptainer, or unavailable execution
resolution. The benchmark model owns ordering, scope closure, availability
reasoning, exact counters, and schema generation; the CLI only observes runtime
probes and assembles the admitted report.

`schemas/local-hpc-resume-simulation.schema.json` defines the persisted decision
to skip a valid completed job or rerun an incomplete job and its dependent
closure. The benchmark model owns completion evidence, topological dependency
propagation, seeded behavior proofs, exact counters, and schema generation; the
CLI only constructs the isolated simulation tree and publishes the admitted
report.

`schemas/local-hpc-dependency-simulation.schema.json` defines persisted
failure-isolation cases over the governed HPC job graph. The benchmark model
owns reciprocal topology, simulated status and timing, descendant scope,
continued independent work, seeded behavior proofs, exact counters, and schema
generation; the CLI only simulates the graph and publishes the admitted report.

`schemas/local-hpc-stage-benchmark-array.schema.json` defines the persisted
homogeneous Slurm array for independent stage benchmark results. The benchmark
model owns row identity, execution availability, paths, commands, resource
envelopes, exact counters, and schema generation; the CLI only joins admitted
planning inputs and renders the script plus manifest.

`schemas/local-hpc-pipeline-node-array.schema.json` defines the persisted local
HPC pipeline-node array. The benchmark model owns ordered dependency lineage,
projected upstream results, expected outputs, paths, commands, exact counters,
resource-envelope closure, and schema generation; the CLI only joins admitted
planning inputs and renders the script plus manifest.

`schemas/local-hpc-dry-run-readiness.schema.json` defines the persisted closure
of the governed local HPC dry-run surfaces. The benchmark model owns the exact
goal inventory, pass/fail counters, behavior proof, readiness verdict, summary
cardinality, safe evidence paths, and schema generation; the CLI only joins the
admitted component reports and publishes the validated closure.

`schemas/local-hpc-submission-readiness.schema.json` defines the persisted
closure of all 99 local HPC submission-readiness goals. The benchmark model
owns goal and category inventory, pass/fail counters, verdict closure, safe
evidence paths, and schema generation; the CLI only evaluates the governed
surfaces and publishes the admitted report.

`schemas/local-hpc-result-collection-simulation.schema.json` defines the
persisted complete, failed, missing, insufficient, and unavailable collection
outcomes. The benchmark model owns status distinctions, evidence lineage,
behavior proofs, exact counters, and schema generation; the CLI only assembles
the governed simulation inputs and publishes the admitted report.

`schemas/local-hpc-scratch-layout.schema.json` defines persisted job-scoped
scratch isolation, input staging and link lineage, bounded resources, and
success/failure cleanup behavior. The benchmark model owns job lineage, exact
counters, containment, resource, and cleanup admission; the CLI only resolves
selected jobs and source assets into the validated layout.

`schemas/local-pipeline-dag-validation.schema.json` defines persisted local
pipeline identity, execution and reference context, project and asset lineage,
node topology, and validation-profile evidence. The benchmark model owns the
record types, schema identity, aggregate closure, and topological admission;
the CLI only loads declarations, derives the report, validates it, and writes
the admitted record.

`schemas/local-pipeline-dag-validation-set.schema.json` defines the canonical
aggregate of governed local pipeline DAG validation records. The benchmark
model owns its schema identity, deterministic ordering, child admission, and
aggregate closure; the CLI only discovers declarations and writes the admitted
set.

`schemas/local-slurm-dependency-check.schema.json` defines persisted scheduler
dependency-source evidence. The benchmark model owns its record types, schema
identity, source classification, per-job verdicts, uniqueness, and aggregate
closure; the CLI only reads rendered scripts and manifests, validates the
derived record, and writes it.

`schemas/local-slurm-shell-syntax.schema.json` defines persisted one-shot
`bash -n` evidence for rendered scheduler scripts. The benchmark model owns its
record types, schema identity, per-script verdicts, uniqueness, and aggregate
closure; the CLI only discovers scripts, executes the bounded syntax check,
validates the derived record, and writes it.

`schemas/local-slurm-dry-run.schema.json` defines the persisted one-shot Slurm
script-rendering record. The benchmark model owns domain identity, readiness
vocabulary, resource and path invariants, uniqueness, aggregate closure, and
the generated schema; the CLI only derives run paths and writes script bodies.

`schemas/local-slurm-submit-manifest.schema.json` defines persisted local Slurm
jobs, resources, logs, artifact outputs, sample scopes, and dependency edges.
The benchmark model owns these records, their generated schema, and admission
closure; the CLI only projects admitted plans into one-shot submission inputs.

`schemas/local-all-domain-slurm-submit-manifest.schema.json` extends that
authority across benchmark-result and essential-pipeline jobs. The model owns
job-kind identity, resource, output, count, and dependency closure; the CLI
only resolves governed benchmark and pipeline inputs into the shared record.

`schemas/local-stage-commands.schema.json` defines persisted stage commands,
argument vectors, resource ceilings, and input/output artifact projections.
The benchmark model owns these records, their schema, identity uniqueness, and
aggregate admission; the CLI only derives commands from governed stage plans
and writes the shell, JSON, and JSONL projections.

`schemas/local-benchmark-summary.schema.json` defines the persisted local
readiness rollup. The benchmark model owns its stage and status vocabulary,
source identities, counters, output cardinality validation, and generated
schema; the CLI only joins validated source reports and renders JSON/Markdown.

`schemas/local-corpus-run.schema.json` defines the persisted FASTQ corpus
benchmark run manifest. The benchmark model owns its identity, sample records,
counter invariants, status semantics, and generated schema; the CLI only
executes samples and writes the validated record.

`schemas/corpus-dossier-summary.schema.json` defines the persisted per-stage
corpus benchmark dossier summary. The benchmark model owns corpus, stage, tool,
sample and cohort coverage, metric validity, headline consistency, CSV sample
row types, and the generated schema; the CLI only assembles observed evidence
and renders the validated dossier.

`schemas/fastq-comparable-metrics.schema.json`,
`schemas/bam-comparable-metrics.schema.json`, and
`schemas/vcf-comparable-metrics.schema.json` define the domain-specific
comparable-metric readiness reports. The benchmark model owns schema identity,
stage and tool coverage, shared-metric status, aggregate closure, and VCF metric
direction semantics; the CLI only derives domain rows and writes them after
validation.

`schemas/stage-centric-report.schema.json`,
`schemas/tool-centric-report.schema.json`, and
`schemas/corpus-centric-report.schema.json` define the complementary readiness
views over the same stage-tool closure. The benchmark model owns nested identity,
tool and stage rosters, blocker lineage, metric applicability, and every aggregate;
the CLI derives and renders only reports admitted by those contracts.

`schemas/all-domain-active-stage-tool-matrix.schema.json`,
`schemas/all-domain-retained-tools.schema.json`, and
`schemas/stage-registry-extra-pairs.schema.json` define the benchmark scope
closure, retained product tool roster, and registry drift view. The benchmark
model requires all eight supported licensed bindings and validates identity,
status, exception lineage, and aggregate closure before thin CLI writers persist
TSV output.

`schemas/local-vcf-smoke-suite-ready.schema.json` defines the complete Goal
211-229 local VCF smoke-suite readiness record. The benchmark model owns goal
identity, failure preservation, output evidence, aggregate closure, and the
generated schema; the CLI only executes each governed smoke and persists the
validated result.

`schemas/all-domain-parser-collector.schema.json` defines parser and normalized
evidence across fake-run and real-smoke sources. The benchmark model owns row
identity, aggregate closure, canonical paths, and the required real-smoke set,
including independent `vcf.stats` and `vcf.prepare_reference_panel` evidence.

`schemas/published-results-status.schema.json` defines the persisted audit of
published benchmark result mirrors. The benchmark model owns stage identity,
completion and issue counters, result-root consistency, issue lineage, and the
generated schema; the CLI only discovers result roots and renders the validated
record.

`schemas/dossier-index.schema.json` defines the persisted index of benchmark
publication dossiers. The benchmark model owns stage identity, publication
state, metadata presence, run-root source semantics, path uniqueness, counters,
and the generated schema; the CLI only discovers dossier inputs and renders the
validated record.

`schemas/publication-status.schema.json` defines the persisted benchmark
publication audit. The benchmark model owns coverage, exclusions, stage and
tool identity, issue lineage, result-root consistency, and generated schema;
the CLI only inspects publication inputs and renders the validated audit.

`schemas/publication-findings.schema.json` defines supplemental findings used
by publication audit and remediation. The benchmark model owns corpus identity,
freshness, issue identity and severity, uniqueness, and the generated schema;
the CLI only loads and groups the admitted findings.

`schemas/remediation-queue.schema.json` defines the persisted benchmark
remediation queue. The benchmark model owns stage ownership, open/clear state,
issue grouping, publication lineage, recommended actions, counters, and the
generated schema; the CLI only gathers evidence and renders the validated queue.

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

`schemas/fastq-benchmark-campaign-report.schema.json`,
`schemas/bam-benchmark-campaign-report.schema.json`,
`schemas/vcf-benchmark-campaign-report.schema.json`, and
`schemas/hardening-benchmark-campaign-report.schema.json` define the persisted
domain and portfolio-hardening campaign records. The benchmark model owns their
versions, selected-goal closure, nested evidence references, status semantics,
and exact summaries; command surfaces only derive and render admitted records.

`schemas/governed-data-catalog-snapshot-publication.schema.json`,
`schemas/governed-data-catalog-registry-publication.schema.json`, and
`schemas/campaign-report-publication.schema.json` define the versioned outcomes
of catalog preservation, catalog admission, and campaign report publication.
The benchmark workflow owns filesystem effects; the model crate owns their
persisted identities and admission rules.

All benchmark model schemas are generated into `schemas/`. The separate
`benchmarks/schemas/` tree owns only domain-normalized metric contracts and must
not contain snapshots of model-owned records.
