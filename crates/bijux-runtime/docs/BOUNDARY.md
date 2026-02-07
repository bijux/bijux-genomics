# BOUNDARY

## Allowed Effects
- Filesystem writes strictly under the run layout.

## Forbidden
- Process spawning
- Docker/network access
- System clock reads (except when explicitly recorded by the runner)

## Enforcement
Policy scans ensure runtime stays effect-free outside the run layout.
