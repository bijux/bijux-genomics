# REPLAY

Replay must never execute tools. It only:

- Validates artifact presence and size.
- Reconstructs records deterministically.

Any attempt to spawn a process during replay is a contract violation.
