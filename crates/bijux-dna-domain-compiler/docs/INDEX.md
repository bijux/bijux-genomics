## Scope
- [SCOPE.md](SCOPE.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)

## Effects
- Pure compilation/validation plus generated file writes.
- No execution effects.

## Boundaries
- Must not execute tools or call runtime backends.
- Must not embed planner-specific behavior.

## Extension Points
- Add new domain keys through schema + compiler mapping in one change.

## How to Test
- [TESTS.md](TESTS.md)
