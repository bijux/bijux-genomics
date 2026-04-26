# Effects

The VCF stage crate is effectful by design. Production code may read VCF,
reference, panel, map, manifest, and metrics files; execute declared VCF tools;
and write stage artifacts under caller-provided output directories.

## Allowed Production Effects

- Read VCF, BCF, TBI, text, JSON, TOML, TSV, panel, map, and manifest inputs.
- Write stage outputs, normalized VCFs, metrics, manifests, checksums, reports,
  logs, readiness markers, and runtime explanation artifacts.
- Invoke declared local tools used by VCF stage contracts, including `bcftools`,
  `bgzip`, `tabix`, and stage-family wrappers where the stage API explicitly
  owns that execution.
- Read workspace reference configuration and lock files through `bijux-dna-db-ref`.
- Read documented environment overrides for VCF stage policy and local tool
  behavior.

## Forbidden Production Effects

- Network access by default.
- CLI argument parsing, API request handling, or user command routing.
- Planner ownership, cross-domain pipeline profile selection, queue scheduling,
  retry orchestration, or environment provisioning.
- Generated configuration writes outside caller-provided stage output trees.

## Determinism Rules

- Fixed fixtures and fixed stage params must produce stable metrics and manifest
  structures.
- Best-effort external-tool paths must have deterministic fallback behavior.
- Environment overrides must be named explicitly in code, tests, and docs.

## Enforcement

- `tests/contracts.rs` exercises stage outputs and refusal behavior.
- `tests/boundaries.rs` owns tree, docs, dependency, and command-boundary tests.
- `tests/guardrails.rs` loads the workspace guardrail policy for this crate.
