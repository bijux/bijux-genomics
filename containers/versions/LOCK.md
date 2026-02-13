# Container Version Lock Rules

Purpose: Define how `containers/versions/versions.toml` is updated and owned.

## Authority
- Owner: platform/tooling maintainers responsible for container reproducibility.
- Canonical file: `containers/versions/versions.toml`.

## Pin Meaning
- A pin is a reviewed tool version value tied to a reproducible container build input.
- Pins must not use floating references (`latest`, branch names, empty values).

## Update Workflow
1. Update container definition(s) and related registry config.
2. Update `containers/versions/versions.toml` to the reviewed version values.
3. Run container lint/policy checks to verify parity and coverage.
4. Commit changes in one logical unit with rationale for the pin changes.
