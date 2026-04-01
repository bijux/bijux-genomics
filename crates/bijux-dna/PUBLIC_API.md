# bijux-dna Public API

`bijux-dna` exposes a curated library surface for CLI integration tests and the package binary:

- `public_api::cli`: CLI schema and parsing types
- `public_api::hpc`: HPC layout contract helpers used by CLI-facing tests
- `public_api::run_with_args` and `public_api::run_with_cli`: process-free command entrypoints for integration coverage
- `run_from_args` and `run_from_env`: crate-local CLI launchers
