# bijux-dna-science Contract

## Role

`bijux-dna-science` is a control-plane crate.

It validates authored science inputs, resolves typed references, compiles traceability outputs, and
cuts immutable science release bundles.

## Boundaries

- no pipeline runtime orchestration
- no stage execution
- no runner dependency
- no graph compilation logic inside rendering code
- no filesystem concerns inside pure domain types

## Inputs and Outputs

- authored input: `science/specs/**`
- generated output: `science/generated/**`
- release output: `artifacts/science-releases/**`
