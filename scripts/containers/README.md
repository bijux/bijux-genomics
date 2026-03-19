# scripts/containers

Compatibility layer for legacy callers.

This directory keeps thin entrypoints that delegate to the canonical implementations in `bijux-dev-dna/containers/`.
The compatibility entrypoints preserve historical paths while making migration to the canonical directory safe for CI, users, and external tooling.

Back to index: `scripts/README.md`.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
