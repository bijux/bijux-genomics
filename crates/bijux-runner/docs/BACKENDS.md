# BACKENDS

## Invariants (Docker == Local)
| Invariant | Description |
| --- | --- |
| cwd | Same working directory semantics. |
| env | Same environment variable filtering and injection. |
| mounts | Same mount resolution rules. |
| stdout/stderr | Same capture behavior and encoding. |
| exit semantics | Same success/failure mapping. |

## Enforcement
The test `tests/backend_invariants.rs` asserts these invariants for both backends.
