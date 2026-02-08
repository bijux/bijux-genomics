# POLICIES

## Policy Registry
| Policy | Intent | Scope | Failure mode | How to fix | Allowlist format |
| --- | --- | --- | --- | --- | --- |
| effect_boundary_map | Prevent forbidden effects | workspace | build fails | move effect to allowlisted crate | list of crate names |
| ssot_catalog_authority | enforce single owner of IDs | workspace | build fails | move definitions to owner crate | list of owner modules |
| docs_spine | enforce docs placement | workspace | build fails | move docs into crate/docs | allowlist file names |
| no_thin_modules | prevent mod.rs-only dirs | src/ | build fails | collapse or expand module | allowlist dirs |
| readme_policy | README required links + link validity | workspace | build fails | add missing links or fix targets | none |
| architecture_pointer_policy | crate architecture docs stay brief | workspace | build fails | shorten ARCHITECTURE.md | none |
| docs_spine_contract | required doc spine snapshot | workspace | build fails | add missing docs or update allowlist | per-crate missing docs |
| test_grouping_policy | test suites grouped into subsuites | workspace | build fails | add tests/ subdirs + spines | per-crate allowlist |
| no_appledouble | ban AppleDouble/DS_Store | workspace | build fails | delete files and re-run | none |

## Policy boundaries rationale
- Effects boundary prevents accidental process spawn in core/runtime (regression: a test helper spawned `bash` in core).
- Dependency boundaries prevent CLI pulling sqlite (regression: CLI depended on database crate).
- SSOT prevents duplicate StageId definitions (regression: stage ID drift between domain and planner).

## Docs quality policy
Each crate `docs/INDEX.md` must contain sections:
- Scope
- Effects
- Boundaries
- Extension Points
- How to Test

## Policy exception protocol
- Open a change with a justification and owner.
- Add allowlist entry with reason and expiry (or "never").
- Review required by policy owners.

## How to add a new policy without duplicating logic in other crates
- Implement policy logic only in `crates/bijux-dna-policies/tests/*` or shared helpers in `tests/support`.
- Do not copy policy logic into production crates; use allowlists instead.
- Document the policy in `docs/INDEX.md` and `docs/TESTS.md`.
- If the policy is a style rule, add it to `docs/40-policies/POLICY_MATRIX.md`.
- Add or update snapshots only when the policy change is intentional (see `BLESS_WORKFLOW.md`).
