# Container Smoke Contract

Purpose: Define deterministic, per-tool smoke behavior across Docker and Apptainer.

Per-tool smoke spec (resolved from registry fields with defaults):
- `smoke_version_cmd`: must execute and produce non-empty output matching `expected_version_regex`.
- `smoke_help_cmd`: must execute with `smoke_help_exit_code` (required to be `0`).
- `smoke_minimal_cmd`: deterministic minimal invocation with `smoke_minimal_exit_code`.
- `smoke_negative_cmd`: expected-failure invocation with `smoke_negative_exit_code` and `smoke_negative_expected_pattern`.
- network behavior: smoke runs must not require network unless `containers/network/<tool>.network.toml` declares `runtime_network = true`.

Exit code contract:
- `--help` path: exit `0`.
- invalid args path: default expected `2` unless registry override.
- minimal/missing-input path: tool-specific expected exit code from registry.

Isolation contract:
- Smoke scripts must run inside isolate (`bin/require-isolate`).
- Smoke writes are allowed only under isolate/artifact roots.

Cross-runtime parity:
- For tools available in both Docker and Apptainer, compare:
  - `version_output`
  - `help/minimal/negative` actual exit codes
  using `scripts/containers/check-cross-runtime-smoke.sh`.
