# OBSERVERS

## Parser contracts
- Accepted tool versions: fastp >=0.23, fastq_screen v1, seqkit v2
- Strictness: unknown fields are ignored, missing required fields cause ParseError.
- Error reporting: ParseError includes tool name and missing field.

## Fixtures
Fixtures live under `tests/fixtures/`.
Naming convention: `<tool>/<fixture>.<ext>` and must match the tool output version.
Versioning: bump fixture name or folder when tool output format changes.
