# NO_DOMAIN

## Forbidden
- imports from domain/stage/planner crates
- defining StageId/ToolId catalogs

## Enforcement
Policy scans check crate dependencies and string literal scans for IDs.
