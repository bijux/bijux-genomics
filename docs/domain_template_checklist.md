# Domain Template Checklist

This checklist defines the expected module and test shape for domain crates.
The goal is to keep FASTQ/BAM/VCF symmetric so new domains are mechanical.

Required modules
- `types/` (public domain types)
- `params/` (tool parameter schemas + defaults)
- `metrics/` (metric schemas + completeness rules)
- `invariants/` (domain invariants + validation)
- `stage_registry/` or `stage_registry.rs` (tool registry + stage ordering)
- `pipeline_contract.rs` (domain pipeline requirements and report sections)

Required tests/support
- Contract/snapshot tests for pipeline contracts and defaults.
- Metrics/invariants completeness checks.
- Registry coverage test (stages are registered and ordered).

Optional modules (domain-specific)
- `banks/` or bank helpers (reference/adapter/motif banks).
- `run/` (domain run layout, manifest helpers, or report adapters).
- `stages/` (stage-specific helpers where the domain owns them).

Refactor guidance
- Keep public exports consistent (`types`, `params`, `metrics`, `invariants`, `stage_registry`).
- Use the same naming conventions across domains.
- Add new domain functionality behind these modules; avoid ad-hoc top-level exports.
