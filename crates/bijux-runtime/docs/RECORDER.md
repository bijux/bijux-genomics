# RECORDER

The recorder is the only write path for manifests and records.

No ad-hoc JSON serialization is allowed outside the recorder.

Example:
```
recorder.write_manifest(&manifest)?;
```
