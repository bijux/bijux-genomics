# OBSERVERS

## Parser contracts
- Accepted versions: mapDamage2 v2, pydamage v0.7, mosdepth v0.3
- Strictness: missing required fields => ParseError
- Error handling: include tool name and field in error

## Determinism
Observer outputs are canonical JSON with stable key ordering.
Numeric rounding follows the core canonicalizer rules; see `bijux-dna-core` canonicalization helpers.

## Tool coverage
See `TOOL_COVERAGE.md` for supported outputs and planned gaps.

## Fixtures
See `FIXTURES.md` for fixture inventory and intent.
