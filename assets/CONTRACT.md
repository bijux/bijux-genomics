# Assets Contract

## Purpose
Define what qualifies as a repository asset and the enforceable rules for layout, immutability, checksums, and regeneration.

## Asset Definition
An asset is deterministic, version-controlled data (not executable code) required by tests, smoke runs, references, publications, or golden contract outputs.

## Allowed Subtrees
- `assets/toy/`
- `assets/golden/`
- `assets/reference/`
- `assets/publications/`

## Immutability Rules
- Assets are immutable-by-default after commit.
- Changes must be made via documented regeneration workflow and committed together with updated checksums/manifests.
- Hand edits to generated golden artifacts are forbidden.

## Checksum Rules
- Every asset package directory must include `CHECKSUMS.sha256`.
- A package directory is any directory under `assets/toy/**` or `assets/golden/**` containing data files.
- `CHECKSUMS.sha256` must validate all package data files.

## Regeneration Workflow
1. Run the relevant generator script (for example `./scripts/run.sh assets refresh-toy` or `./scripts/run.sh assets refresh-golden`).
2. Update package checksums and generation metadata.
3. Run asset policy checks (`./scripts/run.sh checks check-assets-contracts`).
4. Commit data + metadata + check updates together.

## What Must Be Committed
- Regenerated data files.
- Updated `CHECKSUMS.sha256` for each affected package.
- Updated `GENERATE.md` with exact command and tool versions.
- Updated metadata manifests (`manifest.json`, `artifact_checksums.json`, `metrics.json`) where applicable.
- Any policy/config updates needed to keep checks green.

## GENERATE.md Contract
Each package `GENERATE.md` must include:
- `Command(s)`
- `Tool versions`
- `Input origins`
- `Expected outputs`

## Naming Contract
Canonical data filenames in toy/golden packages:
- `reads_1.fastq`
- `reads_2.fastq`
- `reads.fastq`
- `toy.vcf`
- `toy.sam`

## Size Contract
Files larger than the configured threshold require explicit allowlisting in `assets/LARGE_FILE_ALLOWLIST.txt`.
