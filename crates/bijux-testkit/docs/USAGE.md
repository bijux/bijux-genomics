# USAGE

## Stable JSON snapshots
Use `snapshots::stable_json` to serialize a value before snapshotting.

Example:
```rust
use bijux_testkit::snapshots::stable_json;

#[test]
fn schema_snapshot_is_stable() {
    let rendered = stable_json(&serde_json::json!({"schema":"v1"}));
    insta::assert_snapshot!("example_schema", rendered);
}
```

## Fixture helper
See `docs/FIXTURE_STANDARDS.md` and `docs/ADD_FIXTURE.md` for fixture layout rules.
