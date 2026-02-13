# Makefiles Public Surface

Public targets (stable contract):
- `fmt`
- `lint`
- `audit`
- `test`
- `coverage`
- `ci`
- `doctor`
- `refresh-assets-toy`
- `refresh-assets-golden`

All other make targets are internal and must be prefixed with `_`.

Internal targets can be listed with:
- `SHOW_INTERNAL=1 make help`

Target -> script mapping (no hidden magic):
- `fmt` -> `./scripts/run.sh tooling ci-fmt`
- `lint` -> `./scripts/run.sh tooling repo-doctor --fast` + policy checks via `./scripts/run.sh checks ...`
- `audit` -> `./scripts/run.sh tooling ci-audit`
- `test` -> `./scripts/run.sh tooling ci-test`
- `coverage` -> `./scripts/run.sh tooling ci-coverage`
- `doctor` -> `./scripts/run.sh tooling repo-doctor --fast` + fast parity checks
- `ci` -> `./bin/isolate ... make fmt lint audit test coverage`

Current internal targets surfaced by help:
- `domain-validate`
- `examples-validate`
- `_policy-fast`
- `_ci-fast`
- `_ci-slow`
