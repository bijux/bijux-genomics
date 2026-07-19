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
- `github-all`
- `github-all-frozen`
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
- shared Rust targets: `format-rs`, `fmt-rs`, `lint-rs`, `test-rs`, `test-slow-rs`, `test-all-rs`, `audit-rs`, `coverage-rs`, `doctor-rs`, and `rustdoc-check`

Repository-owned implementation targets outside this list are internal and must be prefixed with `_`.

Benchmark note:
- benchmark orchestration and dossier rendering belong to `bijux-dna`
- benchmark make targets are thin wrappers around `bijux-dna bench ...`
- benchmark path authority belongs to `configs/bench/benchmark.toml`

Internal targets can be listed with:
- `SHOW_INTERNAL=1 make help`

Target -> implementation mapping (no hidden magic):
- `fmt`, `lint`, `test`, `test-slow`, `test-all`, and `coverage` -> shared `bijux-makes-rs` targets through `makes/bin/run_genomics_rust_gate.sh`
- `audit` -> the shared Rust audit after Genomics audit-governance prerequisites
- `test-all-frozen`, `lint-frozen`, and `audit-frozen` -> shared immutable-ref launcher
- `github-all` -> the unique Make commands declared by `.github/workflows/ci.yml`, discovered from the pinned source and run concurrently with isolated logs; the complete `test-all` lane replaces the narrower CI `test` lane
- `github-all-frozen` -> the shared immutable-ref launcher running `github-all`
- `doctor` -> `cargo run -q -p bijux-dna-dev -- tooling run repo-doctor --fast` + fast parity checks
- `release-gate` -> docs + root layout + registry lock + container version lock/authority checks
- `gate-essential` -> fast architecture + domain schema + planner determinism + runtime/evidence contract lane
- `gate-execute` -> local/container runner + governed execute/dry-run/status/replay lane, including runtime operations contracts for backend descriptors, queue state, run leases, run control, and operator health
- `gate-evidence` -> FASTQ manifest->plan->runtime->artifact->evidence verification lane plus planner-only BAM/VCF evidence contracts
- `gate-release-essential` -> Level 1 release gate across integrity, canonical smoke execution, dry-run/status evidence coverage, refusal coverage, and advisory/enforced admission checks
- `ci` -> `make fmt lint audit test coverage` under the shared `artifacts/` contract

Rust gate artifact layout:
- fast Rust gates write under `artifacts/rust/`
- `make test-all-frozen`, `make lint-frozen`, `make audit-frozen`, and `make github-all-frozen` start the requested gate from `PINNED_REF` (default `HEAD`) and write the run under `artifacts/<sha>/`
- pinned-ref gate runs materialize the exact source snapshot under `artifacts/<sha>/frozen-repo/` so reports and code stay pinned to the same commit
- pinned-ref gate runs isolate Cargo state under `artifacts/<sha>/rust/`
- pinned-ref gate runs record launcher state under `artifacts/<sha>/background/`, including `<gate>.console.log`, `<gate>.pid`, and `<gate>.exit.status`
- `make github-all-frozen` records per-gate logs, exit statuses, and an aggregate summary under `artifacts/<sha>/github-all/`
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
