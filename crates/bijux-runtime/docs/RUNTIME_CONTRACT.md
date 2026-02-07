# RUNTIME_CONTRACT

## Run Layout
Defines the filesystem layout for a run: run root, step directories, artifact locations.

## Run Record
A structured record of a step execution, including timing and tool identity.

## Provenance
Immutable metadata about tool version, image digest, and input fingerprints.

## Manifest Integrity
A manifest is valid only if it includes:
- Graph hash
- Contract versions
- Input hashes
- Artifact list
- Tool identity per step

## Reference Example
See `tests/reference_example.rs` for a minimal end-to-end layout/record/manifest example.
