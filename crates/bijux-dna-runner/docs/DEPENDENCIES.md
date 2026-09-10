# Dependencies

`bijux-dna-runner` depends only on execution contracts, runtime handoff types, environment image/runtime resolution, infra filesystem helpers, and deterministic identity support.

## Runtime Dependencies
- `anyhow` for execution and replay errors.
- `serde` and `serde_json` for backend specs, execution manifests, records, and runner-owned artifact payloads.
- `tracing` for backend image-resolution warnings.
- `uuid` for temporary Docker container names.
- `sha2` for invocation and input identity hashing.
- `walkdir` for deterministic directory input hashing.
- `bijux-dna-core` for execution steps, tool execution specs, IDs, hashing helpers, canonical JSON, and manifests.
- `bijux-dna-runtime` for the `Runner` trait, invocations, runner results, and artifact handoff.
- `bijux-dna-containers` for runtime kind, image catalog, image resolution, platform contracts,
  frozen-repository admission, and external-artifact verification and binding.
- `bijux-dna-containers-qa` for pipeline-derived qualification probe plans, declared-output
  validation, acceptance metrics, and classified verdicts.
- `bijux-dna-infra` for governed filesystem writes, temp dirs, path helpers, and hashing.

The Slurm backend introduces no scheduler library or async runtime dependency. It invokes the configured `sbatch`, `squeue`, `sacct`, and `scancel` executables through the standard process boundary. Managed workers reuse these existing dependencies and infra hashing; they add no shell parser, scheduler client, or product crate dependency.

## Dev Dependencies
- `bijux-dna-testkit` for shared workspace source scanning support.
- `cargo_metadata` for dependency boundary tests.
- `assert_cmd` for command-runner contract tests.
- `regex` for hostname fixture construction in qualification contract tests.
- `tempfile` for isolated filesystem tests.
- `toml` for the local manifest dependency graph test.

## Forbidden Runtime Dependencies
- Engine, planner, stage, domain, API, CLI, analyzer, benchmark, and pipeline crates.
- Database clients and network clients.
- Compression, time, and async runtime crates unless a concrete runner contract requires them.

## Boundary Rule
If a dependency is only needed for tests, policy inspection, or fixture setup, it belongs in `dev-dependencies`.

The existing `bijux-dna-bench-model` dependency supplies campaign bundle sidecars and scheduler evidence contracts. Bundle backends use the existing command binding and process supervision boundary, SHA-256 support and infra filesystem authority. The `age-cli` backend requires a caller-provided `age` executable or declared execution binding; it introduces no mandatory cryptography library.

The runner consumes the container QA qualification library with default features disabled. Qualification specifications, plans and verdicts do not require the image QA command harness or its SQLite analysis. The API's separate dependency explicitly enables verification tools when that broader public capability is compiled.

Bundle recipient admission rejects empty, blank, padded or duplicate declarations before backend effects. On decode, ordered fingerprints must match the declared recipient strings; the deterministic mock envelope must also match those declarations. The age sidecar is unsigned: matching fingerprints are declaration consistency, not proof that the ciphertext targets those recipients. Age decryption authenticates its payload through the supplied identity, while the mock transport provides no confidentiality. No new dependency is required for this admission.

Bundle publication reuses the infrastructure `FileLock` and managed temporary directory authorities. `FileLock::try_acquire_existing` opens an existing lease without creating files or requiring write access, preserving read-only replay inputs. No new runner dependency or Cargo feature is introduced.

Unix process supervision consumes the independent `bijux-dna-process` leaf. That owner uses the already locked `rustix` event, filesystem and process interfaces; runner no longer declares a direct rustix dependency.

The shared `bijux-dna-process` leaf owns Unix readiness and process lifecycle. Runner keeps execution bindings and scheduler policy; the non-Unix compatibility reader remains private.
