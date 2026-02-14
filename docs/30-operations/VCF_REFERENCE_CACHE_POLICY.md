# VCF Reference Cache Policy

VCF panel/map assets are acquired once on frontend/shared storage and reused by pipelines.

Rules:
- Network fetch for panel/map artifacts is allowed only in:
  - `scripts/tooling/acquire-panels.sh`
  - `scripts/tooling/acquire-maps.sh`
- Runtime/planner stages must not redownload references.
- Cache layout must be:
  - `raw/` immutable downloads
  - `normalized/` indexed/tool-ready outputs
  - `derived/` chunk indices and converted artifacts
- Enabled panel/map in `configs/runtime/profiles/vcf_downstream_local.toml` must:
  - exist in catalog + lock entries,
  - be materialized under cache root,
  - match lock checksums.
