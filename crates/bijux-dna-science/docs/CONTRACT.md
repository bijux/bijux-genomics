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
