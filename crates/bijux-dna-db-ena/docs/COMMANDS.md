# Commands

This file is the SSOT for callable operations owned by `bijux-dna-db-ena`.

## Managed CLI Commands

The `bijux-dna-db-ena` binary owns two direct commands:

| Command | Rust entrypoint | Purpose | Writes |
| --- | --- | --- | --- |
| `query` | `cli::commands::query::execute_query` | Fetch ENA filereport metadata for project, sample, or accession selectors and build an `EnaRunManifest`. | Manifest path only when invoked through the binary flow. |
| `download` | `cli::commands::download::execute_download` | Fetch ENA metadata, plan download tasks, and optionally transfer ENA files. | Manifest path and download outputs under the requested output directory. |

Both commands accept the shared selectors `--project`, `--sample`, and
`--accession`. At least one usable selector is required.

## Managed Library Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `build-filereport-url` | `client::build_filereport_url` | Build the ENA filereport request URL for one accession and result kind. |
| `parse-filereport-tsv` | `client::parse_filereport_tsv` | Decode an ENA filereport TSV payload into typed records. |
| `fetch-records` | `EnaClient::fetch_records` | Validate a query and fetch ENA metadata records over HTTP. |
| `normalize-query` | `EnaQuery::normalized_accessions` | Trim, sort, and deduplicate project, sample, and accession selectors. |
| `validate-query` | `EnaQuery::validate` | Reject empty or malformed ENA selectors before network requests. |
| `select-record-urls` | `EnaRecord::preferred_urls` | Select source URLs using the requested ENA source and URL preference. |
| `build-download-tasks` | `download::build_download_tasks` | Convert records and download config into deterministic output tasks. |
| `download-tasks` | `download::download_tasks` | Execute or dry-run planned download tasks. |
| `write-manifest` | `manifest_store::write_manifest` | Persist the helper binary manifest to the requested path. |

## Local Verification Commands

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-db-ena --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ena --no-default-features
```
