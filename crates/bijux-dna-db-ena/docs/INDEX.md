# INDEX

## Scope
- ENA metadata query + download support for corpus materialization.

## Effects
- Network I/O to ENA endpoints.
- Filesystem writes under explicit output directories.

## Boundaries
- No pipeline planning.
- No tool execution orchestration.

## Extension Points
- Add filters/selection policies in model/client modules.
- Add transfer strategies in download module.

## How to Test
- Unit tests in `src/*`.
- Guardrails in `tests/guardrails.rs`.
- Contract docs in this directory.

## Documents
- [SCOPE.md](SCOPE.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [TESTS.md](TESTS.md)
