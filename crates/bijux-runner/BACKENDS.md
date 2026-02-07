# BACKENDS

Runner backends must obey the same execution contract:
- Same CommandSpec yields the same invocation hash (timestamps excluded).
- CWD, mounts, and env handling are consistent across backends.
- Stdout/stderr capture and exit semantics are identical.
- Replay validates artifacts only and never executes tools.

See STYLE.md for boundary rules.
