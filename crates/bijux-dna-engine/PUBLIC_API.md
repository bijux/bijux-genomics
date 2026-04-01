# bijux-dna-engine Public API

The crate root stays intentionally small and routes its stable surface through `src/public_api/`.

`src/public_api/mod.rs` re-exports:
- `Engine`
- `EngineConfig`
- `CancellationToken`
- `EngineEvent`
- `EngineHooks`
- `EngineError`

New stable items should be added to `src/public_api/mod.rs`, not defined directly in `src/lib.rs`.
