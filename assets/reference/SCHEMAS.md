# Reference Bank Schemas

## Purpose
Define the contract shape and versioning rules for YAML reference banks under `assets/reference/`.

## Scope
Applies to YAML banks used by adapters, contaminants, polyx, and references.

## Non-goals
- Replacing scientific rationale docs for stage defaults.
- Defining runtime planner behavior directly.

## Contracts
- Every YAML bank must include `schema_version`.
- IDs inside a bank must be unique.
- Preset files may only reference IDs that exist in sibling bank files.
- Versioning policy:
  - Backward-compatible field additions: increment minor suffix in file naming convention when needed (e.g. `v1` remains valid).
  - Breaking key/structure changes: create a new major bank file (`*.v2.yaml`) and keep prior major for compatibility windows.

## Bank Inventory
- `adapters/bank.v1.yaml`
  - schema: `bijux.ref.adapters.v1`
  - key contract: list entries with `id`, sequence payload, and metadata fields.
- `adapters/presets.v1.yaml`
  - schema: `bijux.ref.adapters.presets.v1`
  - key contract: preset maps to `adapter_ids` that must exist in adapter bank.
- `contaminants/db_bank.v1.yaml`
  - schema: `bijux.ref.contaminants.db.v1`
  - key contract: contaminants DB entries with stable `id` and reference file mapping.
- `contaminants/contaminant_motifs.v1.yaml`
  - schema: `bijux.ref.contaminants.motifs.v1`
  - key contract: motif entries keyed by unique `id`.
- `contaminants/presets.v1.yaml`
  - schema: `bijux.ref.contaminants.presets.v1`
  - key contract: preset references to contaminant/motif IDs present in sibling banks.
- `polyx/bank.v1.yaml`
  - schema: `bijux.ref.polyx.v1`
  - key contract: unique `id` entries for polyX motifs/policies.
- `polyx/presets.v1.yaml`
  - schema: `bijux.ref.polyx.presets.v1`
  - key contract: preset references to valid polyx bank IDs.
- `references/bank.v1.yaml`
  - schema: `bijux.ref.references.v1`
  - key contract: named reference IDs and pinned source metadata.
- `qc_thresholds.yaml`
  - schema: `bijux.ref.qc_thresholds.v1`
  - key contract: threshold key/value mappings with explicit units or semantics.
