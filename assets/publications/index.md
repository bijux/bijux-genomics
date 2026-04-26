# Publication Assets

## What
This directory contains publication-scoped datasets and metadata manifests.

## Rules
- Every `assets/publications/<pub-id>/` directory must include `MANIFEST.toml`.
- Use stable publication IDs for directory names.
- Publication metadata is authored manually; toy, golden, and reference refresh commands do not rewrite this subtree.

## Update Workflow
1. Edit `MANIFEST.toml` in the target publication directory.
2. Keep `title`, `authors`, `year`, `license`, and `provenance_notes` accurate and reviewable.
3. Update the companion `index.md` when the bundle purpose or authority changes.
4. Re-run `cargo run -q -p bijux-dna-dev -- checks run check-assets-contracts`.
