# Contracts

`bijux-dna-db-ref` owns read-only reference metadata contracts and deterministic
resolver operations over checked-in configuration.

## Species Contracts

Species authority records define canonical species ids, default builds, contig
naming, sex chromosome policy, mitochondrial ids, and ploidy models. Species
alias resolution trims and lowercases aliases, then returns the requested build
or the configured default build.

## Reference Contracts

Reference bank records define reference FASTA metadata, checksums, licenses, and
required index names. Reference bundles define the concrete FASTA, FAI, dict,
contig set digest, lock digests, declared contigs, optional BED files, and
contig normalization policy used by planners.

## Contig Contracts

Strict bundles accept only declared contig names. Deterministic remap bundles
accept configured aliases and must remap to a declared canonical contig. Empty
contig names are rejected.

## Catalog and Lock Contracts

Panel and map catalogs must declare lock references, files, checksums, and
compatibility metadata. Lock references must point to relative lock files under
the catalog directory and anchors must use `locks.<key>`. Lock entries must
match catalog species, build, and id fields.

## Provider Contracts

`RefService`, `ReferenceProvider`, `PanelProvider`, and `MapProvider` are
read-only resolver traits. `RuntimeRefService` is the default implementation
backed by checked-in runtime config.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --test contracts --no-default-features
```
