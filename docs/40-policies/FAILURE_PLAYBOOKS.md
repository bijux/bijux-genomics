# FAILURE_PLAYBOOKS

## What
Common failure modes and the fastest fix paths for policy violations.

## Why
Short playbooks reduce triage time and keep enforcement consistent.

## Non-goals
- Exhaustive troubleshooting guides.
- Replacing the enforcement and diagnostics surfaces.

## Contracts
- Policy IDs and owners are defined in [POLICY_INDEX.md](POLICY_INDEX.md).
- Coverage expectations are defined in [POLICY_MATRIX.md](POLICY_MATRIX.md).
- Diagnostics and test locations are owned by
  [crates/bijux-dna-policies/docs/ENFORCEMENT.md](../../crates/bijux-dna-policies/docs/ENFORCEMENT.md).

## Examples
## Top 10 Failures
1. **docs_spine**: doc in wrong location → move to crate/docs.
2. **docs_tree_contract**: docs tree drift → update snapshot.
3. **readme_policy**: missing section → update README skeleton.
4. **effect_boundary_map**: process spawn in core → move to runner.
5. **dependency_boundaries**: forbidden edge → adjust Cargo.toml.
6. **no_serde_json_writer**: raw JSON write → use canonical writer.
7. **id_literal_policy**: string IDs → use core IDs.
8. **no_helpers_policy**: helpers.rs found → rename.
9. **no_thin_modules_policy**: mod.rs only dir → collapse/expand.
10. **ssot_catalog_authority**: duplicate IDs → move to owner.

Each failure summary stays aligned with those authorities instead of re-describing the
full enforcement implementation here.

## Failure modes
- Missing entries lead to inconsistent fixes and reviewer confusion.
