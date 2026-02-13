# configs/runtime/profiles

Purpose: define named runtime profiles that compose execution defaults and use-cases.

Composition and precedence:
1. Base platform defaults from `configs/runtime/platforms.toml`.
2. Selected profile file (`configs/runtime/profiles/<name>.toml`).
3. Explicit CLI/runtime flags (highest precedence).

Rules:
- Profile filename is the stable profile ID.
- Profile values must not redefine unknown platform IDs.
- Profile values may override defaults but must remain deterministic.
- Every profile TOML must declare `use_case`.
