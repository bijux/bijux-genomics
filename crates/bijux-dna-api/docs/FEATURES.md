# Features

`bijux-dna-api` has an empty default feature set. Feature flags should expose
optional behavior without changing the stable meaning of v1 contracts.

## Feature Flags

| Feature | Purpose |
| --- | --- |
| `api_internal` | Enables internal API modules used by in-repo orchestration and tests. |
| `bam_downstream` | Propagates downstream BAM planning support into planner and pipeline crates. |
| `bench` | Enables benchmarking-oriented code paths and helper exports. |
| `docker-runner` | Enables Docker-runtime behavior hooks where runner/runtime support is available. |
| `report-html` | Enables HTML report rendering integration points. |
| `default` | Empty; callers opt in to optional behavior explicitly. |

## Rules

- Keep feature-gated behavior additive.
- Do not hide breaking schema changes behind a feature flag.
- Document new feature flags here and in `Cargo.toml` in the same change.
- Validate feature-sensitive changes with `cargo test -p bijux-dna-api
  --all-features` before release-facing handoff.
