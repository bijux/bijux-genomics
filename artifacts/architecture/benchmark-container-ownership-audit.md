# Benchmark and Container Ownership Audit

- Generated on 2026-08-04 from commit `e84d6c6aa`.
- Total files audited: 635.

## Classification Counts

- `canonical crate logic`: 137
- `declarative product asset`: 100
- `duplicate legacy logic`: 124
- `obsolete simulation/readiness logic`: 272
- `thin external adapter`: 2

## Scope Counts

- `benchmarks/**/*.py`: duplicate legacy logic=25
- `containers/**/*.py`: duplicate legacy logic=20, thin external adapter=1
- `crates/bijux-dna-bench`: canonical crate logic=66, declarative product asset=84
- `crates/bijux-dna-bench-model`: canonical crate logic=71, declarative product asset=16
- `crates/bijux-dna-dev/src/commands/containers`: duplicate legacy logic=22, thin external adapter=1
- `crates/bijux-dna/src/commands/benchmark`: duplicate legacy logic=57, obsolete simulation/readiness logic=272

## Key Signals

- `bijux-dna-bench` had no reverse dependencies before the migration slice.
- Root Python benchmark and container code still owns production schema identities and report assembly.
- `crates/bijux-dna-dev/src/commands/containers/contracts.rs` still shells out to `containers/contracts/qualification_report.py`.
- `crates/bijux-dna/src/commands/benchmark/**` is dominated by duplicate or obsolete readiness/simulation surfaces.
