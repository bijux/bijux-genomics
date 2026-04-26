# bijux-dna-infra Formats

Format helpers are for config-compatible payloads only. Contract JSON canonicalization and schema
ownership belong outside this crate.

## JSON

- `formats::parse_json` parses non-contract JSON payloads.
- `formats::to_json_pretty` renders deterministic pretty JSON for config-like payloads.
- `atomic_write_json` writes JSON atomically for callers that already own the output contract.

## TOML

- `formats::parse_toml` parses repository configuration inputs.
- `formats::to_toml_string` renders TOML for config-compatible records.

## YAML

YAML support is behind the `yaml` feature. It exists because operator-managed tool image and domain
manifests are YAML-first in parts of the repository. YAML is permitted only for config compatibility
and must not be used for contract JSON schemas.

## Change Rules

- New format helpers must document whether they parse config, operator input, or caller-owned
  artifacts.
- New contract serialization belongs in the contract-owning crate, not infra.
- YAML feature expansion requires dependency review in `DEPENDENCIES.md`.
