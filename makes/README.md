# Makes Public Surface

Public targets (stable contract):
- `fmt`
- `lint`
- `audit`
- `test`
- `test-slow`
- `test-all`
- `test-all-frozen`
- `lint-frozen`
- `audit-frozen`
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
- `test-all-frozen` -> `makes/bin/run_pinned_ref_gate.sh` with `PINNED_REF_GATE_TARGET=test-all`
- `lint-frozen` -> `makes/bin/run_pinned_ref_gate.sh` with `PINNED_REF_GATE_TARGET=lint`
- `audit-frozen` -> `makes/bin/run_pinned_ref_gate.sh` with `PINNED_REF_GATE_TARGET=audit`
- `coverage` -> `makes/bin/rust_gate.sh coverage`
- `doctor` -> `cargo run -q -p bijux-dna-dev -- tooling run repo-doctor --fast` + fast parity checks
- `release-gate` -> docs + root layout + registry lock + container version lock/authority checks
- `gate-essential` -> fast architecture + domain schema + planner determinism + runtime/evidence contract lane
- `gate-execute` -> local/container runner + governed execute/dry-run/status/replay lane, including runtime operations contracts for backend descriptors, queue state, run leases, run control, and operator health
- `gate-evidence` -> FASTQ manifest->plan->runtime->artifact->evidence verification lane plus planner-only BAM/VCF evidence contracts
- `gate-release-essential` -> Level 1 release gate across integrity, canonical smoke execution, dry-run/status evidence coverage, refusal coverage, and advisory/enforced admission checks
- `ci` -> `make fmt lint audit test coverage` under the shared `artifacts/` contract

Rust gate artifact layout:
- fast Rust gates write under `artifacts/rust/`
- `make test-all-frozen`, `make lint-frozen`, and `make audit-frozen` start the requested gate from a pinned checkout for `TEST_ALL_FROZEN_REF` (default `HEAD`) and write the run under `artifacts/<sha>/`
- pinned-ref gate runs materialize the exact source snapshot under `artifacts/<sha>/frozen-repo/` so reports and code stay pinned to the same commit
- pinned-ref gate runs isolate cargo state per gate under `artifacts/<sha>/target/<gate>`, `artifacts/<sha>/cargo/home/<gate>`, and `artifacts/<sha>/tmp/<gate>`
- pinned-ref gate runs record launcher state under `artifacts/<sha>/background/`, including `<gate>.console.log`, `<gate>.pid`, and `<gate>.exit.status`
- `make test-all` and `make test-all-frozen` run the complete suite with no fast/slow filter expression and no slow timeout
- `make lint` is the fast product-crate clippy lane and excludes `bijux-dna-dev`
- workspace governance checks remain available through `make lint-workspace`
- `make test` is the fast Rust lane: it excludes named plus rostered slow tests above the 1-second threshold
- `make test-slow` is the lane for tests that exceed the fast-lane budget
- `make test-all` is the unfiltered, unbounded full-suite lane

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
