# Tests

This file is the single test documentation entrypoint for `bijux-dna-bench`.
README files are intentionally not allowed under `tests/`.

## Suite Entrypoints

- `tests/boundaries.rs` loads source, docs, root-layout, and guardrail tests.
- `tests/contracts.rs` loads public API, benchmark contract, suite catalog,
  docs/fixture, workspace path, and ownership tests.
- `tests/determinism.rs` loads stable ordering, compare, and realistic snapshot
  tests.
- `tests/semantics.rs` loads gate semantic tests.
- `tests/workspace_paths.rs` provides repository path helpers.

## Boundary Tests

- `tests/boundaries/architecture_tree.rs` protects the root layout, 10-docs
  allowance, Markdown location rules, source layout, and test layout.

## Contract Tests

- `tests/contracts/api/` protects the public API snapshot, owner comments, no
  public API panics, and no raw JSON in public surfaces.
- `tests/contracts/benching/` protects benchmark contract shape, suite catalog
  rules, canonical ids, workspace paths, and dependency boundaries.
- `tests/contracts/docs/` protects docs-to-fixture alignment.
- `tests/contracts/corpus_admission.rs` and `fastq_expectations.rs` protect exact
  corpus closure and immutable scientific expectation publication.
- Unit contracts under `fastq_profile_read_lengths/` protect command expansion and
  restart-safe effective parameter evidence.
- `tests/contracts/contracts.rs` is the contract target aggregator.

## Determinism Tests

- `tests/determinism/stable_ordering.rs` protects deterministic summary ordering.
- `tests/determinism/compare.rs` protects compare output snapshots.
- `tests/determinism/bench_realistic_snapshot.rs` protects a realistic suite
  snapshot.

## Semantic Tests

- `tests/semantics/gate/gate.rs` protects gate pass/fail behavior.
- `tests/semantics/gate/rejects_unknown_metric.rs` protects unknown metric
  rejection.

## Fixtures And Snapshots

- `tests/fixtures/` contains governed benchmark input fixtures.
- `tests/snapshots/` contains public API and compare snapshots.

## Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test semantics --no-default-features
```

`repo::corpus::normalized_fastq::tests` covers deterministic mate discovery, curated accession joins, full and paired cohorts, orphan mates, missing/ambiguous accessions, count drift, and requested corpus identity. The adjacent BAM and FASTQ admission modules retain their existing owner tests.

Normalized corpus admission fixtures cover conflicting mate accessions, missing mates, repeated source/sample identities, stale raw and normalized bytes, manifest schema/duplicate-key/path/size rejection, symlinked authority, missing metadata and valid zero-sized cohorts. Tiny compressed reads exercise content identity without claiming external scientific qualification.

The micro-run acceptance case lives in `tests/contracts/micro_run.rs` under the workflow owner. A compact fixture retains eight component identities, all seven result category/status rows, six output rows, one log and two metrics. It verifies version and nested-field admission, schema roundtrip, contradictory counts, unsupported metric claims and missing current evidence, including removal of stale passing verdicts. Repeated literal construction and unused row-file copies are unnecessary.

Micro-run qualification re-observes output artifacts and file logs before declaring a current pass. The retained fixture supplies a distinct sidecar for deletion, a missing/directory log and an existing traversal path; each must clear a previous passing verdict. Directory output artifacts remain supported. Lexical path admission does not promise content identity or protection against concurrent namespace changes.

The existing micro-report producer fixture also verifies model round-trip, unsupported versions, unknown nested input fields, source-count disagreement, header/vector disagreement and overflowing status totals. These checks reuse its two existing succeeded/unavailable observations and retained publication/section assertions; no extra full fixture or workflow is introduced.

Micro-report coverage policy independently tests expected-source and current-evidence obligations, including unavailable results. Repository parser tests retain both report-level and nested-case insufficiency examples. Destination admission rejects traversal, directory roots and aliased outputs without creating directories. The original CLI producer fixture still checks eight upstream adapters, persisted report fields and all Markdown sections.

Micro-report admission compares the exact multiset of component/execution/domain/stage/tool/status identities against source results for result, runtime and memory sections. The existing two-row fixture rejects substituted identities and equal-total forged status/source distributions, clears stale passing verdicts, and accepts a harmless section permutation. No additional workflow fixture is needed.

Shared guardrail parser regressions are owned and executed by the policy crate. This boundary suite retains its crate-specific architecture checks; it does not compile another crate's test source. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

BAM campaign pure goal selection, checks, source identity and evidence projection are covered directly by `bijux-dna-bench-model` policy tests. The benchmark repository retains the admitted derive/publish/load roundtrip under `repo/bam_campaign.rs`. All existing campaign behavior cases remain registered once.

The three taxonomy judgment repository oracles are registered once under `repo/taxonomy_judgment/`: governed matrix agreement, missing-sample refusal and false-positive evidence. Pure summary/evaluation contracts belong to the model tests; the CLI retains its JSON and persisted-output compatibility case.
