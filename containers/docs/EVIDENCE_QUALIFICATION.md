# Container Evidence Qualification

Bijux Genomics separates the decision to retain a tool from the evidence that
qualifies a particular container release. This prevents a recipe, a successful
build, or a version command from being presented as proof of scientific
correctness.

Start with the [container operating guide](../README.md) for execution and the
[container product page](../index.md) for current evidence.

## Two independent decisions

- **Support disposition** is catalog policy. A tool is `retained`,
  `unsupported`, `retired`, or `excluded` in the
  [canonical catalog](../contracts/catalog.toml). A non-retained tool must
  carry a reason and, where appropriate, a replacement.
- **Qualification** is derived evidence. The
  [qualification report](../reports/qualification.md) recomputes every tool
  from the current catalog, admitted-image inventory, runtime campaign, and
  final run evidence. It cannot be edited in the catalog or assigned by a
  lifecycle command.

`retained` means Bijux intends to maintain the integration. It does not mean
that the integration is ready for scientific or release use. `unverified`
means at least one required evidence dimension is absent. `qualified` is
permitted only when every required dimension passes. Non-retained tools are
not eligible for qualification.

## Evidence dimensions

| Dimension | Evidence required |
|---|---|
| Catalog | Stable identity, purpose, ownership, contact, license, version, source, runtime, and stage contract. |
| Source | Immutable release checksum, source digest, or commit. |
| Build | Admitted image hash, recipe hash, build environment, size, provenance, and SBOM. |
| Runtime | Version, help, minimal positive, meaningful negative, network, and writable-path checks. |
| Stage | A declared Bijux stage produces its governed output contract. |
| Real data | Representative non-fixture input succeeds for every retained operational use. |
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
commit. Each tool's catalog version, support disposition, Apptainer support,
and selected image digest must also agree. Qualification recomputes the current
definition digest and every repository-owned `%files` Git object instead of
trusting the inventory's historical freshness label. A benchmark may name the
canonical image or an admitted historical run dependency only when that exact
SIF still matches the current recipe. Stale evidence remains visible for audit,
but it contributes no build, runtime, benchmark, or freshness pass. Benchmark
evidence does not imply scientific acceptance.

```console
python3 -B containers/contracts/qualification_report.py \
  --catalog containers/contracts/catalog.toml \
  --inventory containers/reports/image-inventory.json \
  --runtime containers/reports/runtime-verification.json \
  --runs containers/reports/run-evidence.json \
  --benchmark-report containers/reports/run-2001-bam-genotyping.json \
  --json-output containers/reports/qualification.json \
  --markdown-output containers/reports/qualification.md
```

The fast drift gate recomputes the same report without writing:

```console
cargo run -q -p bijux-dna-dev -- containers run check-evidence-qualification
```

The machine-readable [qualification report](../reports/qualification.json) is
the release selector. The [release checklist](RELEASE_CHECKLIST.md) must not
publish a retained-but-unverified tool or infer qualification from legacy
registry maturity fields.
