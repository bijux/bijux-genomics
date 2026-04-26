# bijux-dna-science Scope

Workspace documentation style is governed by `docs/40-policies/STYLE.md`.

## Owns
- Authored science spec loading and validation.
- Typed science identifier data structures.
- Deterministic evidence-row compilation.
- Stable TSV and JSON rendering for science outputs.
- Science release bundle writing under `artifacts/`.

## Does Not Own
- Pipeline planning.
- Stage execution.
- Container runtime resolution.
- Benchmark policy decisions outside compiled science evidence.
- Tool invocation or filesystem writes outside governed science outputs.
