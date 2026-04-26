# Dependencies

The VCF stage crate has a wider dependency graph than the FASTQ and BAM stage
spec crates because it owns executable VCF stage helpers.

## Normal Dependencies

- `anyhow`: error propagation for stage and IO contracts.
- `serde` and `serde_json`: manifest, metrics, and artifact payloads.
- `regex`: local tool wrapper version validation.
- `sha2`: deterministic checksums and stage-tool digest fallbacks.
- `bijux-dna-core`: shared core utility contracts.
- `bijux-dna-domain-vcf`: VCF domain IDs, params, metrics, and taxonomy.
- `bijux-dna-db-ref`: reference panel, map, and bundle lookup.
- `bijux-dna-infra`: atomic writes, directory helpers, and hashing utilities.

## Dev Dependencies

- `bijux-dna-policies`: guardrail policy loading.
- `bijux-dna-testkit`: deterministic contract test support.
- `tempfile`: isolated contract-test output directories.

## Forbidden Edges

This crate must not depend on API, planner, runtime, runner, or environment
crates. Those crates may consume the VCF stage surface, but the ownership arrow
must not point back from this crate into orchestration surfaces.

## Validation

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo tree -p bijux-dna-stages-vcf --no-default-features --edges normal,dev
```
