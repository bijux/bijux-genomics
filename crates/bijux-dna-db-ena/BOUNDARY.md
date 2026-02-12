# BOUNDARY

## What this crate owns
- Typed ENA metadata/query models.
- ENA selection and download client logic.

## What this crate must not do
- Must not hardcode host-specific paths.
- Must not write artifacts outside caller-provided output roots.
- Must not own pipeline planning or stage execution logic.
