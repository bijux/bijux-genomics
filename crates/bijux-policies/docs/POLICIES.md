# POLICIES

## Policy Registry
| Policy | Intent | Scope | Failure mode | How to fix | Allowlist format |
| --- | --- | --- | --- | --- | --- |
| effect_boundary_map | Prevent forbidden effects | workspace | build fails | move effect to allowlisted crate | list of crate names |
| ssot_catalog_authority | enforce single owner of IDs | workspace | build fails | move definitions to owner crate | list of owner modules |
| docs_spine | enforce docs placement | workspace | build fails | move docs into crate/docs | allowlist file names |
| no_thin_modules | prevent mod.rs-only dirs | src/ | build fails | collapse or expand module | allowlist dirs |

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
