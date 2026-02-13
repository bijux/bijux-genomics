# Bench Suites Index

## What
This directory contains benchmark suite definitions owned by the `bijux-dna-bench` crate.

## Layout
- `suites/*.toml`: canonical benchmark suite specs.

## Governance
- Bench suite data is crate-owned and must not live at repository root.
- Suite paths are resolved through shared path helpers (`bijux_dna_infra::bench_suites_dir`).
- Runtime knobs belong in `configs/bench/`; suite definitions belong here.
