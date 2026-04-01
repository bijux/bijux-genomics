# PUBLIC_API

The crate root preserves the durable module surface:
- `client`
- `download`
- `model`

`src/public_api/mod.rs` curates the stable re-exports:
- `EnaClient`
- `download_tasks`
- `DownloadConfig`
- `DownloadReport`
- `DownloadTask`
- `EnaFileSource`
- `EnaQuery`
- `EnaRecord`
- `EnaResultKind`
- `EnaRunManifest`
- `EnaSourcePreference`

New stable exports should be added to `src/public_api/mod.rs`, not spread directly through
`src/lib.rs`.
