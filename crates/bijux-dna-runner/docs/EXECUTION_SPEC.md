# Execution Spec

The execution spec is the runner contract for turning already-planned tool intent into a backend process call and a stable execution record.

## Execution Semantics
- `cwd`: absolute working directory for backend process execution.
- `env`: explicit environment variables only; no implicit mutation.
- `mounts`: explicit host paths only; inputs are read-only and outputs are writable.
- `stdout/stderr`: captured verbatim and stored per step.
- `exit semantics`: exit code `0` is success; nonzero exits are recorded as failed tool execution.
- `timeouts`: timeout failures are normalized into failed execution outcomes with captured context.
- `file permissions`: output writes follow the declared runtime root and backend permissions.

## Hashing
Invocation hash includes command, args, env, cwd, mounts, and image digest.

## Backend Invariants
| Invariant | Description | Test |
| --- | --- | --- |
| `cwd` | Backend uses the working directory from the execution spec. | `tests/boundaries/backend/backend_invariants.rs` |
| `env` | Backend injects only declared environment variables. | `tests/boundaries/backend/backend_invariants.rs` |
| `mounts` | Backend uses declared mount resolution rules. | `tests/boundaries/backend/backend_invariants.rs` |
| `stdout/stderr` | Backend captures output without rewriting content. | `tests/boundaries/backend/backend_invariants.rs` |
| `exit semantics` | Backend maps zero/nonzero exits consistently. | `tests/boundaries/backend/backend_invariants.rs` |
| `timeouts` | Backend timeout failures become recorded failures. | `tests/boundaries/backend/backend_invariants.rs` |
| `file permissions` | Backend output permissions follow runtime-root policy. | `tests/boundaries/backend/backend_invariants.rs` |

## Failure Semantics
| Failure | Detection | Expected remediation |
| --- | --- | --- |
| Docker or Apptainer missing | Backend executable is unavailable. | Install or configure the selected runtime. |
| Image missing | Backend cannot resolve or run the declared image. | Fix the image reference or registry access. |
| Permission error | Backend returns a filesystem permission failure. | Fix declared mount or output-root permissions. |
| Timeout | Backend exceeds the declared runtime limit. | Increase the limit or reduce input size. |
| Nonzero exit | Tool exits with a nonzero status. | Inspect captured stdout/stderr and tool arguments. |

## Change Rules
- Public field changes, invocation-hash changes, replay semantics changes, and backend invariant changes are breaking unless explicitly approved.
- Contract changes must update this file, `PUBLIC_API.md` when exports change, `DETERMINISM.md` when identity or replay changes, and the matching tests.
