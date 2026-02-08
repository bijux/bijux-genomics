# bijux-dna-engine Architecture

## Purpose
The engine plans, validates, and orchestrates execution. It never executes tools
itself and never touches runtime/container APIs directly.

## Boundary Summary
- Planning and validation live in engine modules.
- Execution is delegated to the runner boundary.
- Recording is emitted via runtime/recording schemas.
- External effects are forbidden (see `EFFECT_BOUNDARY.md`).

## Key Modules
- `ENGINE_MODEL.md` for the core execution model.
- `ENGINE_CONTRACT.md` for cross-crate guarantees.
- `RECORDING_TRUTH_SET.md` for runtime artifacts emitted.
