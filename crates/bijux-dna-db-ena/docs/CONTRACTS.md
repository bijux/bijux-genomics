# Contracts

`bijux-dna-db-ena` owns typed contracts for ENA metadata selection, decoded ENA
records, download planning, and helper-binary manifests.

## Query Contract

`model::EnaQuery` contains project, sample, and explicit accession selectors plus
an `EnaResultKind`. Selectors are trimmed, sorted, deduplicated, and validated
before network requests. Empty selector sets and selectors containing characters
outside accession-safe ASCII are rejected.

## Filereport Contract

`client::filereport` owns ENA filereport field selection, URL construction,
header validation, row width validation, numeric field parsing, and sample
filtering. Missing required columns, duplicate columns, malformed rows, and
invalid numeric values are errors.

## Record Contract

`model::EnaRecord` represents one decoded ENA row. File URL fields are stored as
lists because ENA may return semicolon-delimited values. `preferred_urls` applies
the requested `EnaFileSource` and `EnaSourcePreference` without mutating the
record.

## Manifest Contract

`model::EnaRunManifest` records the query, selected source, URL preference, and
decoded records used by the helper binary. Manifest writes are explicit caller
effects and must stay under the requested manifest path.

## Download Contract

`download::build_download_tasks` maps records to deterministic output paths under
`DownloadConfig::output_dir`. The task list is sorted by output path and
deduplicates repeated output destinations. `download::download_tasks` either
dry-runs the task set or transfers files with the configured job count and retry
count.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ena --no-default-features
```
