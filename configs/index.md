# Configs Index

## What
`configs/` contains versioned, deterministic configuration inputs used by CI, runtime selection, coverage/test tooling, benchmarking, and schema references.

## Layout
- `configs/ci/`: generated and policy-governed CI/SSOT config inputs.
- `configs/coverage/`: coverage thresholds and coverage gate inputs.
- `configs/rust/`: Rust fmt, lint, audit, and nextest configuration.
- `configs/logging/`: logging presets and logging format configuration.
- `configs/bench/`: benchmark suite and benchmark profile configuration; benchmark-local tracked
  configs live under `benchmarks/configs/`.
- `configs/runtime/`: runtime defaults and platform/local runtime knobs.
- `configs/schema/`: schema-oriented config docs or schema descriptors.
- `configs/domain/`: domain policy/config mappings.
- `configs/docs/`: docs toolchain pins.
- `configs/hpc/`: HPC sync and transfer profiles.
- `configs/lab/`: local-lab contract examples.
- `configs/vcf/`: VCF-specific contracts such as reference panel governance and locks.

## Root Files
- `configs/OWNERS.toml`

## Rules
- No random config files are allowed directly under `configs/`.
- New config files must be placed in one of the typed subdirectories above.
