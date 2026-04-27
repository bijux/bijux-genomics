# Container Smoke Contract

Purpose: Define deterministic, per-tool smoke behavior across Docker and Apptainer.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [NETWORK_USAGE.md](NETWORK_USAGE.md)
- [SECURITY_BOUNDARY.md](SECURITY_BOUNDARY.md)
- [../versions/LOCK.md](../versions/LOCK.md)

Per-tool smoke spec (resolved from registry fields with defaults):
- `smoke_version_cmd`: must execute and produce non-empty output matching `expected_version_regex`.
- `smoke_help_cmd`: must execute with `smoke_help_exit_code` (required to be `0`).
- `smoke_minimal_cmd`: deterministic minimal invocation with `smoke_minimal_exit_code`.
- `smoke_minimal_rationale`: required whenever `smoke_minimal_cmd` is effectively the same as
  `smoke_help_cmd`; help-only minimal smoke is allowed only when the registry records why a real
  runnable minimal invocation is not yet governed.
- `smoke_negative_cmd`: expected-failure invocation with `smoke_negative_exit_code` and `smoke_negative_expected_pattern`.
- network behavior: smoke runs must not require network unless `containers/network/<tool>.network.toml` declares `runtime_network = true`.

Exit code contract:
- `--help` path: exit `0`.
- invalid args path: default expected `2` unless registry override.
- minimal/missing-input path: tool-specific expected exit code from registry.

Isolation contract:
- Smoke scripts must run under the shared artifacts contract.
- Smoke writes are allowed only under isolate/artifact roots.

Apptainer artifact identity:
- SIF artifact names under `artifacts/containers/hpc/<tool>/` must use a concrete
  digest key, never a pending or all-zero placeholder.
- The smoke manifest must record both `registry_digest` and the observed
  `sif_sha256` so release gates can distinguish registry identity from the
  actual built SIF payload hash.

Cross-runtime parity:
- For tools available in both Docker and Apptainer, compare:
  - `version_output`
  - `help/minimal/negative` actual exit codes
  using `cargo run -p bijux-dna-dev -- containers run check-cross-runtime-smoke`.
