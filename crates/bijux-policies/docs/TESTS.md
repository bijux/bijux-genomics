# Tests

## What
Maps policy tests to intent and failure meaning.

## Why
Tests become documentation for governance rules.

## Non-goals
- Full test implementation detail.

## Contracts
- Every policy test file must be listed and explained.

## Examples
- `docs_required_policy.rs` → ensures docs placement.

## Failure modes
- Missing test documentation fails `policy_docs_anchor`.

## deps
- `dependency_budgets.rs` — enforces dependency size caps.  
  Failure means a crate pulled in heavy dependencies.
- `effect_boundary_map.rs` — enforces effect boundaries.  
  Failure means a crate uses forbidden effects.
- `infra_boundaries.rs` — infra must not depend on domain/stage/planner.  
  Failure means infra leaked domain semantics.
- `qa_dependency_policy.rs` — QA crates cannot be depended on by production crates.  
  Failure means a production crate depends on QA.

## surface
- `docs_required_policy.rs` — crate docs required in docs/.  
  Failure means docs placement rule violated.
- `docs_spine.rs` — root/authority docs template and metadata checks.  
  Failure means docs missing required sections.
- `docs_tree_contract.rs` — snapshot of docs trees.  
  Failure means unblessed tree drift.
- `domain_purity.rs` — domain crates must be pure.  
  Failure means execution concerns leaked.
- `id_literal_policy.rs` — ID literals only from canonical owners.  
  Failure means stray literal IDs.
- `mod_naming_policy.rs` — no legacy/flat internal naming.  
  Failure means disallowed module name.
- `no_duplicate_canonicalizers.rs` — canonicalization only in core.  
  Failure means duplicate canonicalizer found.
- `no_empty_dirs_policy.rs` — no empty dirs.  
  Failure means placeholder directory exists.
- `no_helpers_policy.rs` — no helpers/utils buckets.  
  Failure means ambiguous naming.
- `no_policy_duplication.rs` — policies only in bijux-policies.  
  Failure means policy logic outside this crate.
- `no_serde_json_writer.rs` — canonical JSON only via core helpers.  
  Failure means direct serde_json writer usage.
- `no_src_crowd_policy.rs` — limit root src modules.  
  Failure means overcrowded src root.
- `no_thin_modules_policy.rs` — no single‑file module dirs.  
  Failure means thin module detected.
- `planner_purity.rs` — planners must not parse or execute.  
  Failure means forbidden logic in planner.
- `runner_tree_policy.rs` — runner tree shape contract.  
  Failure means drift in runner layout.
- `ssot_catalog_authority.rs` — SSOT ownership of catalogs.  
  Failure means duplicate catalog ownership.
- `stage_specs_purity.rs` — stage specs must be declarative.  
  Failure means execution logic in stage specs.
- `style_policy.rs` — style policy entrypoint.  
  Failure means style checks not listed in matrix.
- `tool_id_uniqueness.rs` — tool IDs unique.  
  Failure means duplicate tool ID.

## data
- `contract_handshake.rs` — planner ⇄ runtime ⇄ analyze handshake.  
  Failure means contract mismatch across layers.

## tooling
- `docs_links.rs` — link integrity.  
  Failure means broken intra‑doc links.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
