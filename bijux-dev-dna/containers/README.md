# bijux-dev-dna/containers

Purpose: canonical container runtime build/lint/smoke entrypoints.

Scripts in this directory are the first-class implementation owned by the Rust-native container control plane in `crates/bijux-dev-dna`.
Compatibility shims remain in `scripts/containers/` and execute the same command with `exec` for legacy entrypoints.

Back to index: `scripts/README.md`.

Run surface: `cargo run -p bijux-dev-dna -- containers ...`.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
