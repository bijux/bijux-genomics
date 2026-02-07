# PUBLIC_API

## Public Modules
- `contract`
- `ids`
- `metrics`
- `prelude`

## Why each item is public
contract: required by planners/engine/runtime.  
ids: shared identifier types.  
metrics: shared metrics types.  
prelude: stable import ergonomics.

## How to extend without widening surface
- Add new types under existing modules.
- Prefer pub(crate) and expose through `prelude` if needed.
- Update tests + docs before adding new pub modules.
