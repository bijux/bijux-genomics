# bijux-dev-dna/containers

Purpose: canonical container runtime build/lint/smoke entrypoints.

This directory documents the Rust-native container control plane implemented in `crates/bijux-dev-dna`.

Back to index: `scripts/README.md`.

Run surface: `cargo run -p bijux-dev-dna -- containers ...`.

Requires: bash, rg, coreutils (plus script-specific tools documented inline).
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.
