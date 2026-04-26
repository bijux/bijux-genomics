# Change Rules

These rules govern changes to ENA query, parsing, manifest, download, and
binary-command behavior.

## Breaking Changes

A change is breaking when existing callers, persisted manifests, planned output
paths, or CLI command behavior can observe different results for the same input.

Breaking changes include:

- Renaming or removing public fields on `EnaQuery`, `EnaRecord`,
  `EnaRunManifest`, `DownloadConfig`, `DownloadTask`, or `DownloadReport`.
- Changing ENA selector validation in a way that rejects previously valid
  accessions or accepts ambiguous selectors.
- Changing filereport required fields without matching parser and tests.
- Changing download output path layout.
- Changing URL normalization semantics.
- Changing CLI command names, selector flags, or write destinations.

Breaking changes require matching docs, tests, and downstream caller updates in
the same work item.

## Non-Breaking Changes

The following are normally non-breaking when existing behavior remains stable:

- Adding optional ENA fields to `EnaRecord`.
- Adding a new `EnaFileSource` variant with parser and selection tests.
- Adding a new command flag that has a documented default preserving existing
  behavior.
- Clarifying docs without changing behavior.

## Required Updates

- Commands and writable destinations: `docs/COMMANDS.md`.
- Source and test layout: `docs/ARCHITECTURE.md` and boundary tests.
- Dependency or effect rules: `docs/BOUNDARY.md` and dependency boundary tests.
- Public modules and stable re-exports: `docs/PUBLIC_API.md`.
- Query, record, manifest, or download contracts: `docs/CONTRACTS.md`.
- Test ownership: `docs/TESTS.md`.

## Verification

Run the narrowest relevant suite during development and the full crate suite
before handoff:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ena --no-default-features
```
