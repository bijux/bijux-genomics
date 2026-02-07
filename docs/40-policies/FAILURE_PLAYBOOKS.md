# FAILURE_PLAYBOOKS

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

Each failure links to the matching policy test file.
