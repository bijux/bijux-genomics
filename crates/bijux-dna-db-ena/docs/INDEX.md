# Index

## Scope

- ENA metadata query and download support for corpus materialization.

## Effects

- Network I/O to ENA endpoints.
- Filesystem writes under explicit output and manifest paths.

## Boundaries

- No pipeline planning.
- No tool execution orchestration.
- No dependency on the top-level CLI or downstream execution layers.

## Extension Points

- Add ENA filters and selection policies in `model/` and `client/`.
- Add transfer strategies in `download/`.
- Add helper-binary commands only when `docs/COMMANDS.md` is updated first.

## How to Test

- Unit tests in `src/*`.
- Guardrails in `tests/guardrails.rs`.
- Layout and documentation guards in `tests/boundaries.rs`.

## Documents

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [BOUNDARY.md](BOUNDARY.md)
- [CHANGE_RULES.md](CHANGE_RULES.md)
- [COMMANDS.md](COMMANDS.md)
- [CONTRACTS.md](CONTRACTS.md)
- [DEPENDENCIES.md](DEPENDENCIES.md)
- [PUBLIC_API.md](PUBLIC_API.md)
- [SCOPE.md](SCOPE.md)
- [TESTS.md](TESTS.md)
