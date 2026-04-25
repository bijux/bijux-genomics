# Reference Assets

## What
Production reference data, banks, and presets that are not toy or golden fixtures.

## Rules
- Keep only deterministic data artifacts.
- Domain crates should reference these paths via stable relative paths.
- Source update and pin policy is defined in `assets/reference/LOCK.md`.
- Evidence and sentinel-vs-production status are defined in `assets/reference/EVIDENCE.md`.

---
Asset Provenance Footer
Last regenerated: 2026-02-13
Regenerate command: `cargo run -p bijux-dna-dev -- assets run refresh-reference`
