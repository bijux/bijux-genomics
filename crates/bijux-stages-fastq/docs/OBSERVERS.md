# OBSERVERS

## Parsing contract
Observers parse tool outputs into canonical metrics JSON.

## Fixtures
Fixtures are named after the tool output they represent and live under `tests/fixtures/observer/`.

## Determinism
Parsing the same fixture must yield identical metrics JSON (canonical ordering).

## Failure modes
- Missing expected fields → ParseError
- Unexpected format → ParseError
