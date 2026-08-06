# Public API

The crate root preserves stable module paths for downstream callers:

- `client`
- `download`
- `model`
- `public_api`

`src/public_api/mod.rs` curates ergonomic re-exports for the stable surface:

- `EnaClient`
- `download_tasks`
- `download_verified_file`
- `download_verified_files`
- `DownloadConfig`
- `DownloadReport`
- `DownloadTask`
- `VerifiedDownloadDisposition`
- `VerifiedDownloadOutcome`
- `VerifiedDownloadRequest`
- `EnaFileSource`
- `EnaQuery`
- `EnaRecord`
- `EnaResultKind`
- `EnaRunManifest`
- `EnaSourcePreference`

## Extension Rules

1. Add new ENA protocol behavior under `client/`.
2. Add new download planning or transfer behavior under `download/`.
3. Add new serialized query, record, manifest, or source-selection contracts
   under `model/`.
4. Add stable ergonomic re-exports in `src/public_api/mod.rs`.
5. Do not add new root public modules without updating `README.md`,
   `docs/ARCHITECTURE.md`, this file, and the boundary tests.

## Non-API Modules

The binary-only modules `cli`, `cli_entrypoint`, and `manifest_store` are not
library public API. They exist to support the `bijux-dna-db-ena` helper binary
and must not become dependencies of downstream library code.
