# Bijux Genomics

Bijux Genomics is a reproducible DNA pipeline workspace built around explicit stage contracts, deterministic planning, audited execution, and governed benchmark evidence.

It is designed for operator-visible genomics work rather than opaque workflow orchestration: FASTQ, BAM, and VCF behavior is modeled in owned domain crates, planned through explicit adapters, and checked through tracked readiness and fixture proofs.

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

## What Lives Here

- `crates/`: the Rust workspace, including the `bijux-dna` CLI, runtime, planner, engine, API, domain, policy, and benchmark crates.
- `domain/`: governed FASTQ, BAM, and VCF stage catalogs, metrics, and stage-level contract inputs.
- `benchmarks/`: tracked readiness proofs, schemas, benchmark configs, and benchmark-owned fixtures.
- `configs/`: runtime, CI, coverage, benchmark, HPC, and domain configuration surfaces.
- `containers/`: Docker and Apptainer build surfaces and packaging support.
- `docs/`: architecture, operations, science, policy, and reference documentation.
- `science/`: scientific specs, generated science-facing material, and review-oriented reference artifacts.
- `examples/`: example FASTQ, BAM, and VCF flows and failure-oriented usage slices.

## Core Domains

- FASTQ: read QC, trimming, taxonomy screening, decontamination, preprocessing, and amplicon-oriented stages.
- BAM: alignment-adjacent processing, contamination, kinship, sex inference, coverage, and post-alignment QC.
- VCF: calling, normalization, stats, imputation metrics, panel workflows, and cohort-oriented reporting.

## Quick Start

Prerequisites:

- Rust `1.88`
- `make`
- Optional local container tooling for environment and smoke commands

Common entrypoints:

```bash
cargo run -p bijux-dna -- --help
cargo run -p bijux-dna -- registry list-stages
cargo run -p bijux-dna -- fixtures validate --root benchmarks/tests/fixtures --all
cargo run -p bijux-dna -- bench active-scope validate --fast
```

Repository verification:

```bash
make fmt
make lint
make test
make audit
make ci-fast
```

## Common Operator Surfaces

- Environment inspection and preparation: `bijux-dna env ...`
- Tool and stage registry queries: `bijux-dna registry ...`
- Pipeline profile validation and explanation: `bijux-dna plan ...`
- FASTQ execution surface: `bijux-dna run ...`
- Analysis and comparison: `bijux-dna analyze ...`
- Benchmark readiness, fixture, and result proofs: `bijux-dna bench ...`

The canonical CLI command inventory is maintained in [`crates/bijux-dna/docs/COMMANDS.md`](crates/bijux-dna/docs/COMMANDS.md).

## Documentation

- Project intro: [`docs/00-intro/WHAT_IS_BIJUX.md`](docs/00-intro/WHAT_IS_BIJUX.md)
- Architecture overview: [`docs/10-architecture/ARCHITECTURE_OVERVIEW.md`](docs/10-architecture/ARCHITECTURE_OVERVIEW.md)
- CI and repository operations: [`docs/30-operations/CI.md`](docs/30-operations/CI.md)
- HPC campaign dry-run guidance: [`docs/30-operations/HPC_CAMPAIGN_DRYRUN.md`](docs/30-operations/HPC_CAMPAIGN_DRYRUN.md)
- Licensing reference: [`docs/50-reference/LICENSING.md`](docs/50-reference/LICENSING.md)

## Development Notes

When working from a shared Bijux workspace checkout, start from the workspace-wide contribution policy at the checkout root before editing this repository.

Generated run products, temporary reports, and local proof artifacts belong under `artifacts/` or `runs/` unless a command explicitly governs another checked-in output location.
