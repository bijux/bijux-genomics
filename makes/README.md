# Makes Public Surface

Public targets (stable contract):
- `fmt`
- `lint`
- `audit`
- `test`
- `test-slow`
- `test-all`
- `coverage`
- `ci`
- `doctor`
- `release-gate`
- `gate-essential`
- `gate-execute`
- `gate-evidence`
- `gate-release-essential`
- `refresh-assets-toy`
- `refresh-assets-golden`

All other make targets are internal and must be prefixed with `_`.

Benchmark note:
- benchmark orchestration and dossier rendering belong to `bijux-dna`
- benchmark make targets are thin wrappers around `bijux-dna bench ...`
- benchmark path authority belongs to `configs/bench/benchmark.toml`

Internal targets can be listed with:
- `SHOW_INTERNAL=1 make help`

Target -> implementation mapping (no hidden magic):
- `fmt` -> `makes/bin/rust_gate.sh fmt`
- `lint` -> `makes/bin/rust_gate.sh lint`
- `audit` -> `makes/bin/rust_gate.sh audit`
- `test` -> `makes/bin/rust_gate.sh test`
- `test-slow` -> `makes/bin/rust_gate.sh test-slow`
- `test-all` -> `makes/bin/rust_gate.sh test-all`
- `coverage` -> `makes/bin/rust_gate.sh coverage`
- `doctor` -> `cargo run -q -p bijux-dna-dev -- tooling run repo-doctor --fast` + fast parity checks
- `release-gate` -> docs + root layout + registry lock + container version lock/authority checks
- `gate-essential` -> fast architecture + domain schema + planner determinism + runtime/evidence contract lane
- `gate-execute` -> local runner + governed execute/dry-run/status/replay contract lane
- `gate-evidence` -> FASTQ manifest->plan->runtime->artifact->evidence verification lane plus planner-only BAM/VCF evidence contracts
- `gate-release-essential` -> Level 1 release gate across integrity, canonical smoke execution, dry-run/status evidence coverage, refusal coverage, and advisory/enforced admission checks
- `ci` -> `make fmt lint audit test coverage` under the shared `artifacts/` contract

Rust gate artifact layout:
- fast Rust gates write under `artifacts/rust/`
- `make lint` is the fast product-crate clippy lane and excludes `bijux-dna-dev`
- workspace governance checks remain available through `make lint-workspace`
- `make test` is the fast Rust lane: it excludes `slow__` tests and enforces a 10-second per-test budget
- `make test-slow` and `make test-all` are the lanes for tests that exceed the fast-lane budget

CI profile automation:
- Fast: `cargo run -q -p bijux-dna-dev -- tooling run ci-fast`
- Slow: `cargo run -q -p bijux-dna-dev -- tooling run ci-slow`

Current internal targets surfaced by help:
- `domain-validate`
- `examples-validate`
- `lint-workspace`
- `_policy-fast`
- `_ci-fast`
- `_ci-slow`
