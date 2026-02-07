# Scope

## Belongs here
- Tool selection logic and plan construction.

## Does not belong here
- Execution or parsing.

## Guardrail: no observer parsing
Planner code must not import observer parsing APIs from stage crates. Those belong to
observer-only crates and runtime validation. Purity is enforced by the planner test suite.

See docs/40-policies/STYLE.md for documentation and policy style.
