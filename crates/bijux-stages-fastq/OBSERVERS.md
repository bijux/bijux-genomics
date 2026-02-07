# OBSERVERS

Observers parse tool outputs into metrics and reports. They should be pure,
fixture-driven, and independent of execution transport.

## Fixture Naming Convention
- Directory: `tests/fixtures/<tool>/`
- Files:
  - `<tool>_<version>.txt` for stdout text fixtures
  - `<tool>_<version>.json` for JSON fixtures
  - `<tool>_<version>.tsv` for tabular fixtures
- Parsers must be deterministic: same fixture input yields identical metrics JSON.
