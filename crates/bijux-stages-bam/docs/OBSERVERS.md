# OBSERVERS

## Parsing contract
Observers parse tool outputs into canonical metrics JSON.

## Fixtures
Fixtures live under `tests/fixtures/observer/` and mirror tool output formats.

## Determinism
Parsing must be deterministic for identical fixtures.

## Failure modes
- Missing fields → ParseError
- Invalid format → ParseError
