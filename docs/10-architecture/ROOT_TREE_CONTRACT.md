# Root Tree Contract

Owner: Architecture
Scope: Repository root and the direct container and benchmark product roots
Last reviewed: 2026-08-10
Contract version: v1

## Purpose

Keep the repository layout reviewable and prevent generated paths from recreating a host's absolute workspace hierarchy inside the checkout.

## Repository root

The repository root is governed by the native `check-root-layout` allowlist. New root entries require an architecture decision and a corresponding validator change. Generated output belongs under `artifacts/` unless another governed destination is part of a product contract.

## Container product root

`containers/` has these direct directories:

- `apptainer/`
- `contracts/`
- `docker/`
- `docs/`
- `licenses/`
- `network/`
- `reports/`
- `runs/`
- `tools/`
- `versions/`

Its direct files are `README.md`, `TOOL_IDS.txt`, and `index.md`.

## Benchmark product root

`benchmarks/` has these direct directories:

- `adapters/`
- `configs/`
- `contracts/`
- `corpora/`
- `execution/`
- `reports/`
- `runs/`
- `schemas/`
- `tests/`

Its only direct file is `README.md`.

## Absolute-prefix prohibition

No directory at the repository root or anywhere below `containers/` or `benchmarks/` may recreate an absolute workspace prefix. Directory components named `Users` or `home`, and Windows drive components such as `C:`, are rejected. This applies to tracked, untracked, and ignored paths.

Writers must resolve governed output destinations before creating directories. A host-absolute source path must never be appended beneath a repository-relative destination. Raw attempt logs and scientific payloads belong in identity-addressed governed STUDY paths; repository run and report roots contain only reviewed definitions, indices, citations, publications, and small reports allowed by their contracts.

If an embedded mirror is discovered, record its inventory and checksums, move it to a recoverable quarantine outside the repository, verify the canonical destination, and only then remove the in-repository copy. Do not silently delete evidence.

## Validation

- `CARGO_TARGET_DIR=artifacts/cargo-target cargo run -q -p bijux-dna-dev -- checks run check-root-layout`
- `make _ci-fast`

The root-layout check is part of `ci-fast` and fails before longer checks when the governed tree drifts.
