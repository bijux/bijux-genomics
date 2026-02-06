# Contract Compatibility Rules

Bijux contracts follow semantic compatibility rules. Contract versions are
encoded as `{major, minor}` in truth artifacts.

## Forward Compatibility
- Consumers MAY ignore unknown fields.
- Adding optional fields is a MINOR version bump.
- Adding new enum variants is a MINOR bump and requires a safe default.

## Backward Compatibility
- Removing or renaming fields is a MAJOR version bump.
- Changing the meaning of a field is a MAJOR version bump.
- Tightening validation rules is a MAJOR version bump unless it only rejects
  previously invalid inputs.

## Serializer Stability
- All truth artifacts use canonical JSON: sorted keys, normalized floats,
  normalized paths.
- Hashes are computed from canonical JSON bytes.
