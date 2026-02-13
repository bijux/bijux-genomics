# Makefiles Public Surface

Public targets (stable contract):
- `fmt`
- `lint`
- `audit`
- `test`
- `coverage`
- `ci`
- `refresh-assets-toy`
- `refresh-assets-golden`

All other make targets are internal and must be prefixed with `_`.

Internal targets can be listed with:
- `SHOW_INTERNAL=1 make help`

Current internal targets surfaced by help:
- `domain-validate`
- `examples-validate`
- `_policy-fast`
- `_ci-fast`
- `_ci-slow`
