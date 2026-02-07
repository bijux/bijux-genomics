# PUBLIC_API

The following modules are the only intended public surface:

- contract — stable serialized contracts and validators.
- foundation — core errors, hashing, and cache keys.
- ids — strongly typed identifiers.
- metrics — metrics envelope and semantics.
- prelude — curated re-exports.

## Why each item is public
- `contract`: required by planners/engine/runtime.
- `foundation`: shared core utilities for hashing/errors.
- `ids`: shared identifier types.
- `metrics`: shared metrics types.
- `prelude`: stable import ergonomics.

## How to extend without widening surface
- Add new types under existing modules.
- Prefer pub(crate) and expose through prelude if needed.
- Update tests + docs before adding new pub modules.
