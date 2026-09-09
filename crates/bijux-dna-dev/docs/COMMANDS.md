# bijux-dna-dev Commands

This document is the crate-level command inventory for `bijux-dna-dev`.
The executable surface is `cargo run -p bijux-dna-dev -- <group> <action>`.
Command definitions live in `src/catalog`, and this document records the command ids that the crate is expected to manage.

## Command model
- `checks list` prints registered repository checks; `checks run --all` or `checks run <id>` executes check catalog entries.
- `containers list` prints container-control commands; `containers run <id> -- <args...>` executes a container command.
- `domain list` prints domain-governance commands; `domain run <id> -- <args...>` executes a domain command.
- `assets`, `docs`, `examples`, `hpc`, `lab`, `smoke`, `test`, and `tooling` each support `list` and `run <id> -- <args...>` over their own ops catalog.

## Managed command inventory

### `assets run <id>`
```text
refresh-golden
refresh-reference
refresh-toy
validate-reference
```

### `checks run <id>`
```text
check-artifact-env-contract
check-artifacts-layout
check-artifacts-tracked
check-asset-checksums
check-asset-manifests
check-assets-contracts
check-assets-drift
check-assets-large-file-allowlist
check-assets-reference-schema
check-audit-allowlist
check-automation-arg-style
check-automation-boundary
check-automation-deps
check-automation-entrypoints
check-automation-exit-codes
check-automation-help
check-automation-intent
check-automation-interface
check-automation-network-usage
check-automation-portability
check-automation-temp-discipline
check-automation-writes
check-bench-knob-discipline-downstream
check-bench-knobs
check-benchmark-integrity-policy
check-cargo-config-policy
check-certification-schema-docs
check-ci-automation-surface
check-ci-build-authority
check-ci-dependency-graph
check-cli-command-snapshot
check-clippy-allowlist-expiry
check-clippy-allowlist-growth
check-config-contract-headers
check-config-filenames
check-config-headers
check-config-index-discipline
check-config-layout
check-config-owners
check-config-schema
check-container-ssot-parity
check-coverage-regimes-schema
check-deny-policy-deviations
check-deprecations-enforcement
check-docs-build-contract
check-docs-requirements-lock
check-domain-realization
check-domain-tool-parity
check-enabled-vcf-panel-metadata
check-examples-cli-snapshot
check-examples-corpus-checksums
check-examples-corpus-layout
check-examples-corpus-manifests
check-examples-golden
check-examples-index-ssot
check-examples-notebook-policy
check-examples-policy
check-examples-runner-contract
check-examples-structure
check-executor-features-docs
check-executor-no-unwrap
check-frontend-mini-artifacts
check-frontend-mini-domain-validation
check-frontend-observability-proof
check-frontend-telemetry-sanity
check-generated-config-headers
check-generated-configs
check-gitignore-contract
check-golden-artifact-schema
check-hidden-tmp-usage
check-hpc-frontend-constraints
check-hpc-rsync-docs-parity
check-hpc-safety
check-legacy-automation-references
check-legacy-automation-removed
check-logging-contract
check-make-help-sync
check-map-locks
check-nextest-profile-contract
check-no-fake-artifacts
check-no-wait-gate
check-no-parallel-accidental
check-no-raw-cargo-in-automation
check-no-raw-cargo-in-makes
check-no-target-paths-in-tests
check-no-user-path-literals
check-output-roots
check-ownership-audit
check-panel-license-policy
check-panel-locks
check-param-registry-completeness
check-readme-links
check-reference-fetch-paths
check-reference-path-governance
check-reference-service-boundary
check-registry-required-tools-parity
check-registry-split
check-registry-ssot
check-root-layout
check-run-directory-layout
check-runtime-execution-kernel-config
check-runtime-profiles-contract
check-rustflags-consistency
check-species-aliases
check-ssot-guardrails
check-stage-domain-parity
check-stage-executor-parity
check-stage-id-symmetry
check-stage-registry-fixtures
check-stage-registry-governance
check-tool-registry-lock
check-vcf-compatibility-matrix
check-vcf-deprecation-lifecycle
check-vcf-downstream-readiness
check-vcf-reference-governance
```

### `containers run <id>`
```text
apptainer-build-all
apptainer-ensure
apptainer-ensure-stage
build-all
build-apptainer-all
build-apptainer-hpc-frontend
build-bundle
build-images
build-tool
check-apptainer-bijux-header
check-apptainer-cache-policy
check-apptainer-frontend-reproducibility
check-apptainer-frontend-security
check-apptainer-frontend-smoke-proof
check-apptainer-frontend-version-output-lock
check-apptainer-hardening
check-apptainer-post-pins
check-apptainer-rebuild-repro
check-apptainer-version-label-sync
check-bijux-apptainer-built
check-bijux-template-markers
check-build-provenance
check-contract-catalog
check-contract-lock
check-cross-runtime-representative
check-cross-runtime-smoke
check-digest-changes-on-version-change
check-digest-output-policy
check-docker-arch-policy
check-docker-arm64-completeness
check-docker-context
check-docker-hardening
check-docker-labels
check-docker-unpinned-apt
check-docker-version-sync
check-dockerfiles-built
check-evidence-qualification
check-image-inventory-publication-schema
check-image-inventory-report
check-image-inventory-scan-schema
check-ghcr-apptainer-publish-matrix
check-ghcr-publish-matrix
check-hpc-frontend-policy-enforcement
check-hpc-image-naming
check-image-size-regression
check-imputation-cross-runtime-parity
check-imputation-hardening
check-imputation-network-policy
check-imputation-release-smoke
check-imputation-runtime-constraints
check-index
check-license-index-generated
check-license-metadata
check-lock-change-discipline
check-lock-drift
check-lock-matches-built-output
check-lock-schema
check-missing-images
check-network-disclosure
check-no-secrets
check-upstream-provenance
check-owners
check-qa-matrix-generated
check-qualification-report-schema
check-qualification-schema
check-rebuild-repro
check-registry-drift
check-registry-vs-defs
check-release-checklist
check-runtime-downloads
check-runtime-tool-digest-recording
check-sbom-artifacts
check-smoke-contract
check-smoke-contract-lock
check-smoke-failure-classification
check-smoke-inputs-policy
check-time-locale-determinism
check-tool-container-coverage
check-tool-docs-generated
check-tool-id-contract
check-tool-id-manifest
check-tool-invocation-normalization
check-tool-name-collision
check-tool-name-map-generated
check-toolkit-bundle-buildable
check-toolkit-bundles
check-vcf-downstream-bundle-coverage
check-vcf-imputation-toolchain
check-version-authority
check-version-completeness
check-version-deprecations
check-version-hash-pin
check-version-immutability
check-version-lock
check-versions-index-sha
check-vuln-allowlist
check-vuln-hook
compare-frontend-local-sif-hash
container-doctor
container-runtime-check
container-smoke
containers-smoke
credential-reference
deprecate-version
docker-build-all
ensure-images
env-prep
env-smoke
external-artifact
extract-version-map
generate-contract-lock
generate-evidence-qualification
generate-execution-bindings
generate-image-inventory
generate-image-inventory-report
generate-image-inventory-publication-schema
generate-image-inventory-scan-schema
generate-ghcr-apptainer-publish-matrix
generate-ghcr-publish-matrix
generate-index
generate-license-metadata
generate-local-apptainer-digests
generate-network-usage
generate-qa-matrix
generate-qualification-report-schema
generate-qualification-schema
generate-tool-docs
generate-tool-ids
generate-tool-name-map
generate-version-lock
generate-versions-index-sha
grype-database
image-activation
image-campaign
image-qa
image-smoke-vcf
image-storage
lint
pipeline-contract
qualification-cycle
qualification-registry
qualification-slurm
qualification-specification
registry-tools
release-evidence
release-gate
repository-archive
repository-freeze
run-apptainer-frontend-reproducibility
run-apptainer-frontend-security
run-apptainer-frontend-smoke
run-evidence
runtime-verification
security-evidence
security-toolchain
smoke-apptainer
smoke-containers-apptainer
smoke-containers-apptainer-apptainer-run
smoke-containers-apptainer-bijux-run
smoke-containers-apptainer-verify
smoke-containers-docker-amd64
smoke-containers-docker-arm64
smoke-cross-runtime-verify
smoke-docker-amd64
smoke-docker-arm64
smoke-toolkit-apptainer
smoke-toolkit-docker-arm64
summary
test-images
test-images-stage
test-images-tool
vuln-scan-hook
```

### `docs run <id>`
```text
check-doc-assets
check-doc-depth
check-doc-links
check-doc-major-depth
check-doc-root-layout
check-docs-graph
check-domain-doc-references
check-generated-docs
check-no-placeholder-language
check-root-pollution
```

### `domain run <id>`
```text
check-default-settings-docs
check-doc-links
check-domain-index
check-domain-layout
check-domain-schema
check-domain-tool-metadata
check-external-tool-policy
check-fixture-contracts
check-inventory
check-orphan-files
check-planner-fixture-coverage
check-planner-stage-coverage
check-reference-bundle-lock
check-rust-stage-catalog-parity
check-shared-tools
check-ssot-authority
check-tool-container-parity
generate-index
generate-inventory
inventory-drift
lock-registry
validate
```

### `examples run <id>`
```text
check-drift
check-index
generate-index
run
```

### `hpc run <id>`
```text
benchmark-attempt-migration
benchmark-sync-pull
benchmark-sync-push
managed-slurm-job
run-frontend-mini-e2e
site-cycle-preflight
validate-frontend-constraints
```

### `lab run <id>`
```text
run-bench
run-pipelines
```

### `smoke run <id>`
```text
run
smoke-bam
smoke-fastq
```

### `test run <id>`
```text
fastq-gold-repro
reproduce-failure
test-control-plane-smoke
test-triage
toy-runs
```

### `tooling run <id>`
```text
acquire-maps
acquire-panels
acquire-reference
architecture-report
artifact-contracts
benchmark-campaign-cycle
benchmark-data-catalog-snapshots
benchmark-data-portfolio-audit
benchmark-integrity-mini
benchmark-progress-ledger
benchmark-protocol-portfolio
benchmark-publication-registry
benchmark-run-index
benchmark-run-registry
benchmark-scientific-readiness
benchmark-smoke-level1
benchmark-stage-coverage
benchmark-stage-tool-closure
benchmark-stage-universe
bijux
cargo-targets
certification-gate
certify-all
certify-bam
certify-domains
certify-fastq
certify-level1
certify-vcf
check-config-paths
check-config-snapshot
ci-audit
ci-clippy
ci-clippy-executors
ci-coverage
ci-fast
ci-fmt
ci-install-tools
ci-slow
ci-test
ci-test-all-sharded
ci-test-slow
clean-docs
config-inventory
coverage-summary
crash-triage
deprecate-vcf-knob
deprecate-vcf-panel
docs-build
flake-hunt
generate-compatibility-matrix
generate-config-tree-snapshot
generate-configs
generate-docs
generate-docs-graph
generate-domain-coverage-doc
generate-panel-compatibility-matrix
generate-policy-index
generate-repo-root-map
generate-tool-index
image-qa
inventory
lint-fast
make-help
operator-workflow-maturity
reference-external-data
repo-doctor
scientific-caveat-propagation
setup-docs-venv
simulate-coverage-regime
validate-frontend-mini-domain-stacks
```

`ci-test-all-sharded` executes every shard in the checked-in coverage contract when no arguments
are supplied. A classified failure may be recovered with repeated `--shard <id>` pairs; the
timing receipt records `affected_recovery` scope and cannot claim complete contract coverage.
Unknown, duplicate, empty, or malformed selections fail before the archive build starts.
The exclusive `--preflight-only` option validates the same build authority, pinned executor and
runtime dependency bindings, writes the normal dependency report, and returns before preparing
an archive or launching shards. A preflight pass is not a test or coverage pass.

`benchmark-data-catalog-snapshots publish` preserves each flat v1 governed catalog inside a
digest-bound v2 authority, retains current v2 records from an explicit prior registry, and
publishes the complete source-addressed registry. Existing identical publications are replayed;
conflicting snapshot or registry bytes are rejected without replacement.

`benchmark-data-catalog-snapshots build-bam-diversity` revalidates one exact materialization plan
against the structured sample selection in `ancient-dna-bam-diversity.toml`, source-addressed
FASTQ and reference catalogs, container-image digests, and all ten BAM/index result pairs. It
publishes a new derived-catalog candidate under the governed data root; it does not replace the
flat authority. `admit-derived` separately performs compare-and-replace admission and republishes
the complete source-addressed registry.

`benchmark-data-portfolio-audit` publishes an immutable typed join between the benchmark portfolio
and one exact governed STUDY data index. It records catalog identity, access and redistribution
classes, access terms, observed sample-count authority, verified storage, source projections,
unbound holdings, and suspected toy, fixture, synthetic, or smoke contamination without promoting
index eligibility to scientific admission.

`benchmark-progress-ledger validate` performs bounded loading and Rust-owned semantic validation
of task, execution-unit, attempt, check, commit, external-wait, and complete frontier references.

`benchmark-protocol-portfolio` generates or checks the complete permanent campaign, stage
interface, parameter, and run-directory declaration closure from the Rust benchmark authority.

`benchmark-publication-registry` generates or checks the crate-owned classification of current
governance products, active and historical stage reports, and quarantined legacy readiness files.
It fails when stale stage evidence remains in the active namespace or historical report paths lose
their source-commit identity.

`benchmark-campaign-cycle prepare` composes benchmark-owned corpus and execution-spec preparation
with runner-owned Slurm materialization. It selects crate-owned material by the source-locked run
identity, initializes or recovers every attempt, and records `ready` lifecycle evidence without
submitting or polling scheduler jobs. The benchmark crate owns executable material for every
campaign in the current registry: run-2001 through run-2004, run-2011, run-2030, run-2042,
run-2043, and run-2051.
Registry closure tests fail when a declared campaign lacks crate-owned material.

Every campaign operation that creates or consumes a source-locked plan requires an explicit
`--repository-root`. The operator binary may come from a newer clean worktree, but source identity,
contracts, interfaces, and generated execution material are always resolved from that declared
repository root.

`benchmark-campaign-cycle submit` performs the same complete authority and material preflight
before the first scheduler effect, then submits every governed tool through the runner's
intent-first API and records each returned Slurm job identity. It returns after submission without
observing or polling jobs; repeating it with the same timestamps recovers persisted handles without
duplicate submission.

`benchmark-campaign-cycle reconcile` performs at most one scheduler observation for each submitted
or running attempt. Every actual query is first appended to the attempt's typed
`scheduler-observations/` journal, including repeated running states, before lifecycle reduction
records running or terminal accounting transitions. Non-runnable lifecycle states are reported
without polling, and the command never loops or waits for scheduler state to change.

`benchmark-campaign-cycle admit` admits only terminally collected attempts. It uses benchmark-owned
validation to rehash exact output closure, evaluate declared metrics and the downstream probe,
publish immutable acceptance evidence, and record `passed` lifecycle transitions.

`benchmark-campaign-cycle publish` selects the report implementation from the source-locked plan,
replays the admitted attempt closure, and no-clobber publishes JSON and Markdown identities. Each
campaign implementation retains its own normalized output, metric, science, comparison, and
downstream-probe validation; the operator command does not branch on scientific report semantics.
Publication also requires `--presentation-repository-root`. Current source catalogs use their own
presentation metadata. Historical v1 catalogs that predate that metadata receive only absent
`report_presentation` tables from the explicit presentation repository; all execution, scientific,
parameter, adapter, and downstream-probe fields remain validated against `--repository-root`. The
operator reports the presentation commit, catalog SHA-256, and exact overlaid stage IDs.

Fast policy lanes (`policy-fast`, `ssot-policy-fast`, `registry-lint`, and `release-readiness`) list their required nextest selections before execution. Every module group or exact named case must resolve to a non-ignored test. Missing, renamed, unexpected, or ignored selections fail the lane. The inventory is retained under `artifacts/ci/<lane>/inventory.json`; each lane then executes its combined selection in one nextest run.
