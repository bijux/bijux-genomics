# BACKENDS

## Conceptual module symmetry
| Concept | Docker module | Local module |
| --- | --- | --- |
| execution spec | `backend/docker/execution_spec.rs` | `backend/local/execution_spec.rs` |
| execution core | `backend/docker/executor.rs` | `backend/local/executor.rs` |
| replay | `backend/docker/replay.rs` | `backend/local/replay.rs` |

## Invariants (Docker == Local)
| Invariant | Description | Test |
| --- | --- | --- |
| cwd | Same working directory semantics. | `tests/backend_invariants.rs` |
| env | Same environment variable filtering and injection. | `tests/backend_invariants.rs` |
| mounts | Same mount resolution rules. | `tests/backend_invariants.rs` |
| stdout/stderr | Same capture behavior and encoding. | `tests/backend_invariants.rs` |
| exit semantics | Same success/failure mapping. | `tests/backend_invariants.rs` |
