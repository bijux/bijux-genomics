# PARAMS

Canonicalization normalizes:
- ordering of keys
- float formatting
- path normalization

This ensures stable hashing and comparisons.

## Checklist: add a new stage param
- Update param schema and canonicalization in `src/params/*`.
- Update invariant expectations in `src/invariants/*`.
- Update metric semantics in `src/metrics/*`.
- Refresh stage contract snapshots in `tests/contracts/stage_contract_snapshots.rs`.

## Tool-specific trim params
The FASTQ trim domain now exposes typed tool models in `src/params/trim.rs`:
- `SkewerTrimParamsV1`
- `LeeHomTrimParamsV1`
- `AlienTrimmerParamsV1`
- `FastxClipperParamsV1`

Shared axes are modeled explicitly:
- adapter modes
- minimum length (bp)
- quality trim mode and cutoff (Phred)
- overlap/collapse behavior
- paired/single read handling

`LeeHomTrimParamsV1` also captures overlap-specific controls for ancient-DNA style merge/collapse behavior.
