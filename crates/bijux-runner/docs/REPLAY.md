# REPLAY

Replay never executes tools. It only verifies artifacts on disk.

## Does
- Validate artifacts exist and are non-empty.
- Reconstruct records deterministically.

## Does not
- Spawn processes
- Pull images

## Integrity failure
Missing or mismatched artifacts cause failure.
