# BOUNDARIES

API wires requests to planners/engine. It does not invent truth.

## Allowed
- Orchestration via bijux-dna-core + bijux-dna-runtime

## Forbidden
- Tool execution
- Domain-specific selection logic

## Internal modules
`src/internal/*` is non-public wiring code and may change at any time.
