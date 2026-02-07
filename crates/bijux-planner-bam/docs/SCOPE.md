# Scope

## Belongs here
- Tool selection logic and plan construction.

## Does not belong here
- Execution or parsing.

## Guardrail: no parsing or execution
Planner code must not import observer parsing APIs or spawn processes. Purity is enforced
by the planner test suite.

See docs/40-policies/STYLE.md for documentation and policy style.
