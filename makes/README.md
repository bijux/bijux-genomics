# Makes Public Surface

Public targets (stable contract):
- `fmt`
- `lint`
- `audit`
- `test`
- `coverage`
- `ci`
- `doctor`
- `release-gate`
- `refresh-assets-toy`
- `refresh-assets-golden`

All other make targets are internal and must be prefixed with `_`.

Internal targets can be listed with:
- `SHOW_INTERNAL=1 make help`

Target -> control-plane mapping (no hidden magic):
- `fmt` -> `cargo run -q -p bijux-dev-dna -- tooling run ci-fmt`
- `lint` -> `cargo run -q -p bijux-dev-dna -- tooling run repo-doctor --fast` + policy checks via `cargo run -q -p bijux-dev-dna -- checks run ...`
- `audit` -> `cargo run -q -p bijux-dev-dna -- tooling run ci-audit`
- `test` -> `cargo run -q -p bijux-dev-dna -- tooling run ci-test`
- `coverage` -> `cargo run -q -p bijux-dev-dna -- tooling run ci-coverage`
- `doctor` -> `cargo run -q -p bijux-dev-dna -- tooling run repo-doctor --fast` + fast parity checks
- `release-gate` -> docs + root layout + registry lock + container version lock/authority checks
- `ci` -> `make fmt lint audit test coverage` under the shared `artifacts/` contract

CI profile automation:
- Fast: `cargo run -q -p bijux-dev-dna -- tooling run ci-fast`
- Slow: `cargo run -q -p bijux-dev-dna -- tooling run ci-slow`

Current internal targets surfaced by help:
- `domain-validate`
- `examples-validate`
- `_policy-fast`
- `_ci-fast`
- `_ci-slow`
