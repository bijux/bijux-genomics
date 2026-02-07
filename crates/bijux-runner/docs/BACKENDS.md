# BACKENDS

## Invariants (Docker == Local)
| Invariant | Description | Test |
| --- | --- | --- |
| cwd | Same working directory semantics. | `tests/backend_invariants.rs` |
| env | Same environment variable filtering and injection. | `tests/backend_invariants.rs` |
| mounts | Same mount resolution rules. | `tests/backend_invariants.rs` |
| stdout/stderr | Same capture behavior and encoding. | `tests/backend_invariants.rs` |
| exit semantics | Same success/failure mapping. | `tests/backend_invariants.rs` |
