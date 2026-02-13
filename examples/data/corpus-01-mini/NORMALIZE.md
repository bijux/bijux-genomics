# Normalize Corpus-01-Mini

Purpose: define deterministic normalization from `raw/` into `normalized/`.

Steps:
1. Read FASTQ/metadata from `raw/`.
2. Apply canonical filename normalization and stable ordering.
3. Write normalized outputs under `normalized/`.

Contract:
- normalization is deterministic
- source `raw/` files remain unchanged
