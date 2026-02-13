# Tool Lifecycle

Purpose: define the lifecycle states and required gates for containerized tools.

## Lifecycle Diagram
```text
planned -> experimental -> production -> deprecated -> removed
    |          |              |
    |          |              +-- requires lock + smoke + provenance + policy gates
    |          +-- requires runnable defs and smoke contract
    +-- registry-only declaration, no production guarantees
```

## State Semantics
- `planned`: declared in registry/backlog, may not have full runtime coverage.
- `experimental`: buildable/testable path exists, not yet production-stable.
- `production`: locked, reproducible, and policy-gated.
- `deprecated`: retained for compatibility window with replacement guidance.
- `removed`: no longer shipped.

## References
- `containers/docs/PROMOTION_POLICY.md`
- `containers/versions/deprecations.toml`
- `scripts/containers/promote.sh`
- `scripts/containers/demote.sh`
