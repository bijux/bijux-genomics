# Architecture

## Goals
- Keep the crate root thin and public-surface oriented.
- Separate build-time catalog helpers from runtime resolution behavior.
- Split environment resolution by enduring concerns instead of one catch-all module.

## Source tree

```text
src/
├── build/
│   ├── defaults.rs
│   ├── mod.rs
│   ├── models.rs
│   └── version_parser.rs
├── lib.rs
├── resolve/
│   ├── cache.rs
│   ├── catalog.rs
│   ├── commands.rs
│   ├── mod.rs
│   ├── platform.rs
│   ├── reference.rs
│   ├── smoke.rs
│   └── types.rs
├── runtime_spec.rs
└── surface.rs
```

## Responsibilities
- `surface.rs`: crate-level public API surface.
- `build/`: docker tool definitions and dockerfile version parsing.
- `resolve/platform.rs`: platform loading and runner selection rules.
- `resolve/catalog.rs`: image catalog loading, digest hydration, and image resolution.
- `resolve/smoke.rs`: smoke command execution and shell capture support.
- `resolve/cache.rs`: cache roots and deterministic image-cache paths.
- `resolve/reference.rs`: prepared reference materialization and index registration.
- `runtime_spec.rs`: pure runtime spec pairing between platform and runner.

## Change rules
- Add new root files only for enduring top-level concerns.
- Prefer focused submodules over expanding `resolve/mod.rs` or `build/mod.rs`.
- Update this document and the boundary architecture contract together when the tree changes intentionally.
