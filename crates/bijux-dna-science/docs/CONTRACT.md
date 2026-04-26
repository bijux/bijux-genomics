# bijux-dna-science Contract

## Role
`bijux-dna-science` validates authored science inputs, resolves typed references, compiles
traceability outputs, and cuts immutable science release bundles.

## Inputs and Outputs
- Authored input: `science/specs/**`
- Generated output: `science/generated/**`
- Release output: `artifacts/science-releases/**`

## Boundaries
- No pipeline runtime orchestration.
- No stage execution.
- No runner dependency.
- No graph compilation logic inside rendering code.
- No filesystem concerns inside pure domain types.

## Owns
- Authored science spec loading and validation.
- Typed science identifier data structures.
- Deterministic evidence-row compilation.
- Stable TSV and JSON rendering for science outputs.
- Science release bundle writing under `artifacts/science-releases/`.

## Does Not Own
- Pipeline planning.
- Stage execution.
- Container runtime resolution.
- Benchmark policy decisions outside compiled science evidence.
- Tool invocation or filesystem writes outside governed science outputs.
