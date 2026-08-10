# Container Evidence Qualification

Bijux Genomics separates product support, artifact delivery, redistribution
permission, artifact availability, and qualification evidence. This prevents a
license restriction, recipe, successful build, or version command from being
presented as a product-support or scientific verdict.

Start with the [container operating guide](../README.md) for execution and the
[container product page](../index.md) for current evidence.

## Independent support and artifact dimensions

- **Product support** is catalog policy: `supported`, `retired`, or `excluded`.
- **Artifact delivery** is `redistributable_image`, `external_artifact`,
  `private_site_build`, or `site_module`.
- **Redistribution permission** records whether Bijux may publish the artifact
  bytes. It does not determine product support.
- **Availability** is evidence-derived: `admitted`, `blocked_external_access`,
  or `unavailable`.
- **Qualification** is derived evidence. The
  [qualification report](../reports/qualification.md) recomputes every tool
  from the current catalog, admitted-image inventory, runtime campaign, and
  final run evidence. It cannot be edited in the catalog or assigned by a
  lifecycle command.

`supported` means Bijux maintains the integration. It does not mean that an
artifact is presently available or qualified for scientific or release use.
Supported external tools remain in the report. Without an admitted artifact
they carry `unavailable` evidence; a governed access audit can refine that
state to `blocked_external_access`. `unverified` means at least one required
evidence dimension is absent. `qualified` is permitted only when every required
dimension passes. Retired and excluded tools are not eligible for qualification.

Every dimension decision records the exact rule version, measured values,
thresholds, outcome, and reason. Passing decisions additionally carry the
immutable SHA-256 of their admitted evidence. Missing observations, empty
thresholds, and disagreement between the recorded outcome and verdict fail
closed.

## Evidence dimensions

| Dimension | Evidence required |
|---|---|
| Catalog | Stable identity, purpose, ownership, contact, license, version, source, runtime, and stage contract. |
| Source | Immutable release checksum, source digest, or commit. |
| Build | Admitted image hash, recipe hash, build environment, size, provenance, and SBOM. |
| Runtime | Version, help, minimal positive, meaningful negative, network, and writable-path checks. |
| Stage | A declared Bijux stage produces its governed output contract. |
| Real data | Representative non-fixture input succeeds for every supported operational use. |
| Science | Predeclared domain metrics and acceptance thresholds pass. |
| Benchmark | Comparable runtime, memory, CPU, I/O, input scale, and output-quality measurements exist. |
| Security | Vulnerabilities, privileges, network behavior, writable paths, and secret exposure are assessed. |
| Release | Immutable artifact, evidence bundle, documentation, and release metadata agree. |
| Freshness | Evidence records the recipe, source, repository commit, site, runtime, and execution time. |

The [canonical image storage reconciliation](../reports/image-storage.md),
[complete isolated rebuild](../reports/lunarc-image-rebuild.md),
[image inventory](../reports/image-inventory.md),
[runtime verification](../reports/runtime-verification.md), and
[run evidence](../reports/run-evidence.md) deliberately prove different
dimensions. The [science boundary](SCIENCE_EVIDENCE_BOUNDARY.md) explains why
container execution and scientific acceptance require independent review.

## Validation and release use

Regenerate the reports only from these governed inputs:

- the canonical catalog;
- the portable published image inventory;
- the portable published runtime execution;
- the final run-evidence index; and
- each reviewed stage-benchmark report admitted by the release gate.

The inventory and runtime inputs must carry publication schemas with
`study://` and `repository://` paths plus the raw evidence SHA-256. Raw LUNARC
execution JSON is intentionally rejected because tracked qualification must
not depend on one operator's absolute paths.

Build and runtime evidence is current only when the inventory and runtime
campaign both name the present catalog SHA-256, the runtime campaign names the
published inventory's raw SHA-256, and both campaigns name the same repository
commit. Each tool's product support, artifact delivery, redistribution policy,
availability, catalog version, and selected artifact identity must also agree.
Qualification recomputes the current
definition digest and every repository-owned `%files` Git object instead of
trusting the inventory's historical freshness label. A benchmark may name the
canonical image or an admitted historical run dependency only when that exact
SIF still matches the current recipe. Stale evidence remains visible for audit,
but it contributes no build, runtime, benchmark, or freshness pass. Benchmark
evidence does not imply scientific acceptance.

```console
cargo run -q -p bijux-dna-dev -- containers run generate-evidence-qualification
```

The generator fails before writing when catalog, recipe, image, runtime, run, or benchmark evidence
cannot be admitted by its owning Rust crate. It never refreshes stale evidence or converts a failed
campaign into a passing claim.

The report contract is generated from the policy type and checked independently:

```console
cargo run -q -p bijux-dna-dev -- containers run generate-qualification-report-schema
cargo run -q -p bijux-dna-dev -- containers run check-qualification-report-schema
```

The fast drift gate recomputes the same report without writing:

```console
cargo run -q -p bijux-dna-dev -- containers run check-evidence-qualification
```

The machine-readable [qualification report](../reports/qualification.json) is
the release selector. The [release checklist](RELEASE_CHECKLIST.md) must not
publish a supported-but-unverified tool or infer qualification from legacy
support-disposition or registry maturity fields.
