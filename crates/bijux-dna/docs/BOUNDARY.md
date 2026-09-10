# bijux-dna Boundary Contract

Owner: CLI
Scope: User-facing command adapter over stable API, registry, environment, and operator helpers
Allowed inputs: CLI arguments, current working directory, repository configs, explicit environment variables, API responses
Forbidden dependencies: engine internals, runner internals, runtime internals, and undeclared internal crates that bypass owned benchmark and fixture workflows
Forbidden effects: undeclared writes, undeclared network access, process spawning, hidden runtime mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features`

## Why this crate exists
`bijux-dna` is the operator command surface. It turns command-line input into typed API calls,
renders human/JSON output, and exits with categorized operator failures.

## What this crate owns
- CLI parser types and command names.
- Command routing and process-free CLI entrypoints for integration tests.
- Terminal/JSON rendering at the CLI boundary.
- Help text, command snapshots, and operator-facing error categorization.
- Thin adapters for environment, registry, corpus, benchmark, status, BAM, FASTQ, and VCF commands.

## What this crate must not own
- Scientific domain truth, stage catalogs, or tool semantics.
- Planner selection policy.
- Stage execution or runner backend behavior.
- Hidden network access or shell/container spawning.
- Workspace dependency decisions that belong to architecture policy.
- Benchmark planning, readiness validation, fixture truth generation, lifecycle state, evidence
  admission, aggregation, publication, or replay semantics.

## Allowed dependencies
- `bijux-dna-api` for stable planning, runtime, report, environment, and domain-facing contracts.
- `bijux-dna-infra` for declared filesystem helpers.
- CLI support libraries for parsing, serialization, rendering, and deterministic archives.

## Forbidden dependency shape
The CLI must not directly depend on benchmark, engine, runner, runtime, domain, planner,
stage-contract, or stage implementation crates to run product logic. Existing direct product edges
are migration debt: they may only be removed or routed behind `bijux-dna-api`, never expanded, and
they do not grant the CLI ownership of the behavior they expose.

## Allowed effects
- Read repository configs and governed domain/assets files through declared paths.
- Write command outputs requested by flags or documented command contracts.
- Create deterministic observability log packs from declared run artifacts.

## Notes
The family-level contract is indexed in
[docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md](../../../docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md).

Corpus fixture commands and benchmark workflows consume `bijux-dna-bench-corpus`
for local manifest and payload admission. The CLI enables its `alignment` and
`variants` features and owns argument defaults, service invocation, JSON/text
presentation and exit mapping. Corpus models and validation do not belong in
the command crate; downstream owners can use that service without compiling
this CLI or the benchmark aggregation library.

VCF benchmark commands supply admitted catalog/binding inputs and a reference
materialization operation to `bijux-dna-bench-vcf`. That owner constructs adapter
contracts and fixture observations without a normal dependency on CLI or API.

Local micro-workflow orchestration belongs to `bijux-dna-bench-workflows`. CLI wrappers
adapt arguments, supply fixture operations/tool bindings and present the returned reports.

Population benchmark services belong to `bijux-dna-bench-population`. CLI composition
acquires explicitly scoped declarations and presents the returned reports.

Reference-panel, phasing, imputation and imputation-quality benchmark services
belong to `bijux-dna-bench-haplotypes`. CLI composition supplies scoped stage
declarations and tool bindings, then renders the returned report.
