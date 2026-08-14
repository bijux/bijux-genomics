# Target Architecture

Owner: Architecture
Scope: Container and benchmark product ownership, lifecycle boundaries, and evidence flow
Contract version: v1
Status: Normative

## Purpose

Define the exclusive Rust ownership boundaries for the container and benchmark products. This
contract governs where product semantics live, how generic stage execution is composed, and which
repository surfaces may contain declarations or publications without becoming alternate lifecycle
engines.

Dependency edges remain governed by
[the boundary map](docs/10-architecture/BOUNDARY_MAP.md). Authored domain truth and generated
configuration remain governed by [the SSOT contract](docs/10-architecture/SSOT.md).
Production corpus, truth, materialization, and scientific claim eligibility remain governed by
[the data portfolio and truth contract](DATA_PORTFOLIO_AND_TRUTH_CONTRACT.md).

## Product Boundary

Containers and benchmarks are application products implemented by workspace crates. The
[`containers/`](containers/) and [`benchmarks/`](benchmarks/) trees are declarative and
publication surfaces. They may contain reviewed recipes, catalogs, contracts, generated schemas,
protocol definitions, citations, small reports, documentation, and narrow external-tool workers.

Those root trees must not own lifecycle state, scheduler control, run allocation, attempt
transitions, evidence admission, scientific or policy verdicts, aggregation, evidence books,
publication selection, or replay. Raw data, images, attempts, logs, and large outputs belong in
identity-addressed governed STUDY paths, not in Git.

## Exclusive Product Owners

| Crate | Exclusive responsibility |
| --- | --- |
| `bijux-dna-bench-model` | Persisted benchmark run, campaign, case, attempt, accounting, evidence, report, publication, and citation records; validation; generated JSON Schemas. |
| `bijux-dna-bench` | Ready-frontier planning, registry-driven case generation, data and evidence admission, aggregation, report production, evidence books, publication selection, and replay. |
| `bijux-dna-runtime` | Immutable attempt transitions, retry and supersession lineage, recovery, journals, and governed runtime recording. |
| `bijux-dna-runner` | One-shot local, Apptainer, and Slurm submission; one-shot observation and terminal collection; cancellation; event-driven local process supervision. |
| `bijux-dna-containers` | Container catalog, recipe/source/artifact identity, image build and build publication, image inventory and selection, and checksum-bound external-artifact admission. |
| `bijux-dna-containers-qa` | Runtime, stage, output, security, and release probes that produce observations for admission or policy evaluation. |
| `bijux-dna-stage-contract` | `ArtifactContract`, `StageContract`, `ToolBinding`, `ParameterProjection`, `ResultNormalizer`, `DownstreamProbe`, and the generic execution-plan vocabulary. |
| `bijux-dna-science` | Scientific metrics, truth comparison, uncertainty, claim eligibility, and downstream scientific acceptance. |
| `bijux-dna-policies` | Licensing, product support, artifact delivery, freshness, security, release, and publication decisions. |
| `bijux-dna` | Thin user CLI: argument parsing, service invocation, rendering, exit behavior, and no durable product semantics. |
| `bijux-dna-dev` | Thin operator composition: command routing and repository checks over crate-owned services, with no duplicate lifecycle. |

## Supporting Crate Boundaries

- `bijux-dna-domain-*` owns authored domain identifiers, vocabularies, constraints, and typed
  scientific stage declarations.
- `bijux-dna-domain-compiler` compiles authored domain truth into generated configuration views.
- `bijux-dna-stages-*` owns domain-specific invocation and parsing implementations expressed
  through the generic stage contracts.
- `bijux-dna-planner-*` owns deterministic domain plan assembly and selection explanations, not
  benchmark lifecycle or backend execution.
- `bijux-dna-pipelines` owns default pipeline composition and its defaults ledger.
- `bijux-dna-engine` executes an explicit plan without selecting tools or implementing backends.
- `bijux-dna-api` is the stable application composition surface over lower-level crate contracts.
- `bijux-dna-analyze` consumes produced artifacts for analysis without admitting benchmark
  evidence or deciding publication state.
- `bijux-dna-core`, `bijux-dna-infra`, database crates, and `bijux-dna-testkit` remain shared
  foundations within their crate-local boundary contracts.

## Generic Stage Lifecycle

One lifecycle handles every admitted stage/tool pair:

1. Domain and catalog authorities identify the stage, supported tools, artifact contracts, and
   declared product-support dimensions.
2. `bijux-dna-stage-contract` validates the `StageContract`, `ArtifactContract`, `ToolBinding`,
   parameter projection, result normalizer, and downstream probe before execution is allocated. It
   constructs the immutable `StagePlanV1` from those admitted authorities; lifecycle owners do not
   reconstruct tool commands or artifact mappings independently.
3. `bijux-dna-bench` derives every dependency-ready case and execution unit without waiting for
   unrelated tools, stages, images, or campaigns. Preparation projects each unit's admitted paths
   and parameter provenance through the canonical binding to build the full measured-repetition
   plan matrix; catalog executable/version identity and the binding's thread allocation must match
   the immutable execution specification before the attempt can become ready.
4. `bijux-dna-runner` submits the declared execution once and returns its durable backend identity.
5. `bijux-dna-runtime` appends the attempt transition and retry or supersession lineage.
6. A later cycle observes scheduler state once, collects terminal/accounting-ready attempts once,
   and passes immutable outputs through the declared normalizer and probes.
7. `bijux-dna-science` and `bijux-dna-policies` derive independent scientific and governance
   decisions from admitted observations.
8. `bijux-dna-bench` admits eligible evidence, closes ready cases, publishes each independently
   complete run, and updates evidence books or replay receipts.

Adding a tool may add a catalog entry, binding, adapter, normalizer, probe, or governed artifact. It
must not add stage- or tool-specific branches to the generic scheduler, lifecycle, evidence,
admission, aggregation, or report engines. Invalid combinations fail during planning.

## Non-Waiting Control Flow

A lifecycle cycle is one-shot and idempotent:

1. Validate the ledger.
2. Observe active backends once.
3. Collect every terminal and accounting-ready attempt once.
4. Compute the complete ready task and execution-unit frontier.
5. Prepare and submit every independent ready unit within declared resource bounds.
6. Persist identities, dependencies, evidence roots, backend handles, and the next frontier.
7. Return immediately.

Foreground sleep/status polling is forbidden. True scheduler dependencies use `afterok`; terminal
collection and failure classification may use `afterany`; arrays are restricted to homogeneous
resource and output contracts. Local concurrency uses event-driven child supervision. A running
attempt blocks only its own execution unit.

## Identity and Publication

- A run ID is a permanent semantic protocol identity.
- An attempt is one immutable execution of a run case.
- Every measured execution unit has a pre-execution `StagePlanV1`; the attempt stores the complete
  repetition-by-unit plan matrix and a checksum-bound manifest beside its immutable execution
  specification.
- Retry and supersession create new attempts and explicit lineage; they never rewrite history.
- A publication selects admitted attempts without changing their evidence.
- Publication receipts derive their selected attempts from the source-locked plan's passing
  lineage tips and retain the immutable plan, case matrix, and admitted attempt evidence; callers
  cannot nominate unrelated attempts or omit supersession authority.
- Evidence books may admit an explicit immutable receipt before the final tracked run pointer
  exists; the receipt must match the permanent protocol, and the checked-in pointer is selected
  only after book, replay, and citation evidence close without a circular dependency.
- Replay accepts that receipt only together with the immutable candidate evidence book that admits
  it; mixed candidate/indexed authority is rejected before semantic comparison. The benchmark
  engine validates the source-locked case matrix, selects exactly one case for every admitted
  attempt, publishes those case definitions immutably beside the replay receipt, and retains the
  matrix as an input lock. Callers do not assemble or approximate the selected case closure.
- A citation is the stable paper-facing record for a run protocol and its selected publication.
- Scheduler job IDs are provenance, not run identities.
- Current reports require exact source, catalog, stage/interface, data/truth, artifact, campaign,
  and report hash closure. A stale report remains historical and is never overwritten into
  currency.

## Licensed External Artifacts

Product support and artifact distribution are independent. A supported restricted tool remains in
planning, stage coverage, and comparison matrices even when its artifact is not currently
available. Container and policy records carry these independent dimensions:

- `product_support`
- `artifact_delivery`
- `redistribution_permitted`
- `availability`

An external artifact becomes executable only after checksum-, version-, architecture-, source-,
and license-scope admission. Restricted bytes never enter Git, public reports, or replay bundles.
Blocked access is an honest availability result, not completed execution evidence.

## Repository Publication Surfaces

The benchmark product root owns declarations and reviewed publications such as contracts, corpus
source definitions, permanent run protocols, citations, case indexes, current-publication
pointers, report indexes, and thin workers. The container product root owns recipes, catalogs,
licenses, permanent run protocols, report indexes, and reviewed evidence-book publications.

Neither root may contain a copied checkout, an embedded absolute-workspace mirror, raw attempt
logs, SIF bytes, private external artifacts, or scheduler state. The exact direct-root shape and
absolute-prefix prohibition are governed by
[the root tree contract](docs/10-architecture/ROOT_TREE_CONTRACT.md).

## Enforcement

Architecture changes must keep this contract, crate-local architecture documents, the
[contract index](docs/10-architecture/CONTRACT_INDEX.md), and executable boundary checks aligned.
At minimum, validate:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-dev -- checks run check-ci-dependency-graph
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-dev -- checks run check-no-wait-gate
```

Migration is complete only when executable ownership checks show that root scripts and the CLI
contain no non-adapter product semantics and every required lifecycle surface is implemented by
its exclusive owner.
