# API Stability

## Versioning
The public surface is `v1::api`. All public request/response schemas are snapshot-tested.

## Compatibility rules
- Backward-compatible changes: additive fields with defaults.
- Breaking changes: rename/remove fields, change types, or change required fields.
- Any breaking change must bump the contract version and update schema snapshots.

## Tests
See `crates/bijux-api/tests/api_stability.rs` for schema snapshots.
