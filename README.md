# Bijux Genomics

Bijux Genomics is a governed DNA workflow workspace for FASTQ, BAM, and VCF processing.
It emphasizes explicit stage contracts, deterministic planning, reproducible execution, and benchmark evidence that stays visible to operators and reviewers.

<!-- bijux-genomics-badges:generated:start -->
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-0F766E)](https://github.com/bijux/bijux-genomics/blob/main/LICENSE)
[![CI](https://github.com/bijux/bijux-genomics/actions/workflows/ci.yml/badge.svg)](https://github.com/bijux/bijux-genomics/actions/workflows/ci.yml)
[![deploy-docs](https://img.shields.io/badge/deploy--docs-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-crates](https://img.shields.io/badge/release--crates-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-pypi](https://img.shields.io/badge/release--pypi-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-ghcr](https://img.shields.io/badge/release--ghcr-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release-github](https://img.shields.io/badge/release--github-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/actions)
[![release](https://img.shields.io/badge/release-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics/releases)
[![ghcr](https://img.shields.io/badge/ghcr-no%20status-9CA3AF)](https://github.com/bijux?tab=packages&repo_name=bijux-genomics)
[![published packages](https://img.shields.io/badge/published%20packages-no%20status-9CA3AF)](https://github.com/bijux/bijux-genomics)

[![Repository docs](https://img.shields.io/badge/docs-no%20status-9CA3AF?logo=materialformkdocs&logoColor=white)](https://github.com/bijux/bijux-genomics/tree/main/docs)
<!-- bijux-genomics-badges:generated:end -->

## What This Repository Owns

- Governed FASTQ, BAM, and VCF stage catalogs under `domain/`
- The `bijux-dna` CLI and supporting Rust crates under `crates/`
- Deterministic benchmark fixtures, configs, schemas, and readiness proof under `benchmarks/`
- Runtime, CI, and registry configuration under `configs/`
- Container build surfaces under `containers/`
- Architecture, operations, science, and reference docs under `docs/` and `science/`

## Working Model

The repository is organized around a contract-first workflow:

1. Domain manifests define stages, tools, artifacts, metrics, and defaults.
2. Planner crates derive deterministic stage and tool selections from those manifests.
3. Runtime and API crates execute or materialize the governed surfaces.
4. Readiness and benchmark commands verify whether declared support is actually executable, parseable, and reportable.

This keeps stage claims tied to owned code and checked fixtures instead of prose-only declarations.

## Supported Domains

- FASTQ: validation, QC, trimming, filtering, taxonomy screening, depletion, and amplicon-oriented interpretation
- BAM: alignment follow-up, QC, contamination, authenticity, damage, kinship, sex inference, and downstream preparation
- VCF: calling, normalization, QC, phasing, imputation, population reporting, and cohort-oriented analytics

## Repository Layout

- `benchmarks/`: benchmark configs, schemas, fixtures, readiness snapshots, and deterministic local-ready proof
- `configs/`: generated registries, CI inputs, runtime settings, and benchmark parameter catalogs
- `crates/`: CLI, planners, runtime, API, domain crates, policies, and benchmark/reporting crates
- `domain/`: source-of-truth stage and tool manifests for FASTQ, BAM, and VCF
- `docs/`: human-facing architecture, operations, and reference documentation
- `examples/`: small usage examples and focused walkthrough inputs
- `runs/`: local smoke and execution output roots

## Quick Start

Prerequisites:

- Rust `1.88`
- `make`
- Optional Docker or Apptainer for container-backed smoke and readiness commands

Common commands:

```bash
cargo run -p bijux-dna -- --help
cargo run -p bijux-dna -- registry list-stages
cargo run -p bijux-dna -- fixtures validate --root benchmarks/tests/fixtures --all
cargo run -p bijux-dna -- bench active-scope validate --fast
```

Common verification:

```bash
make fmt
make lint
make test
make audit
make ci-fast
```

## Operator Entry Points

- `bijux-dna registry ...`: inspect governed stages, tools, and contracts
- `bijux-dna plan ...`: validate and explain planner decisions
- `bijux-dna run ...`: execute owned workflow surfaces
- `bijux-dna analyze ...`: summarize, compare, and inspect results
- `bijux-dna bench ...`: validate fixtures, readiness, generated commands, and benchmark proof

The full CLI inventory is maintained in [`crates/bijux-dna/docs/COMMANDS.md`](crates/bijux-dna/docs/COMMANDS.md).

## Documentation

- Project introduction: [`docs/00-intro/WHAT_IS_BIJUX.md`](docs/00-intro/WHAT_IS_BIJUX.md)
- Architecture overview: [`docs/10-architecture/ARCHITECTURE_OVERVIEW.md`](docs/10-architecture/ARCHITECTURE_OVERVIEW.md)
- CI operations: [`docs/30-operations/CI.md`](docs/30-operations/CI.md)
- HPC dry-run guidance: [`docs/30-operations/HPC_CAMPAIGN_DRYRUN.md`](docs/30-operations/HPC_CAMPAIGN_DRYRUN.md)
- Licensing: [`docs/50-reference/LICENSING.md`](docs/50-reference/LICENSING.md)

## Development Notes

- Generated benchmark proof belongs under `benchmarks/readiness/` when it is deterministic and worth checking in.
- Disposable local execution output belongs under `artifacts/` or `runs/`.
- Source-of-truth domain changes should be followed by the relevant index, registry, and readiness regeneration commands before commit.

