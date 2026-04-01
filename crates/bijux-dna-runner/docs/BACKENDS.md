# BACKENDS

## Conceptual module layout
| Concept | Docker module |
| --- | --- |
| execution spec | `backend/docker/execution_spec.rs` |
| execution core | `backend/docker/executor.rs` |
| replay | `backend/docker/replay.rs` |

## Backend invariants checklist (Docker)
| Invariant | Description | Test |
| --- | --- | --- |
| cwd | Same working directory semantics. | `tests/boundaries/backend/backend_invariants.rs` |
| env | Same environment variable filtering and injection. | `tests/boundaries/backend/backend_invariants.rs` |
| mounts | Same mount resolution rules. | `tests/boundaries/backend/backend_invariants.rs` |
| stdout/stderr | Same capture behavior and encoding. | `tests/boundaries/backend/backend_invariants.rs` |
| exit semantics | Same success/failure mapping. | `tests/boundaries/backend/backend_invariants.rs` |
| timeouts | Same timeout application and error mapping. | `tests/boundaries/backend/backend_invariants.rs` |
| file permissions | Same output permission behavior. | `tests/boundaries/backend/backend_invariants.rs` |
