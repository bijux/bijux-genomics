# Execution Spec

The execution spec is the runner contract for turning already-planned tool intent into a backend process call and a stable execution record.

## Execution Semantics
- `cwd`: absolute working directory for backend process execution.
- `env`: explicit environment variables only; no implicit mutation.
- `mounts`: explicit host paths only; inputs are read-only and outputs are writable.
- `stdout/stderr`: captured verbatim and stored per step.
- `exit semantics`: exit code `0` is success; nonzero exits are recorded as failed tool execution.
- `timeouts`: graph step deadlines are propagated to every backend runner; a timed-out local child
  is killed, reaped, and returned as an explicitly classified timeout failure.
- `file permissions`: output writes follow the declared runtime root and backend permissions.
- `external artifacts`: `external-artifact://<tool>` plans require an admission record; the runner
  reverifies its catalog, checksum, version evidence, stage allowlist, and command binding before
  selecting local binary or Apptainer SIF execution.

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

## Slurm Lifecycle
- Submission invokes `sbatch --parsable` once and returns the exact scheduler job identity without polling.
- Batch submission requires an explicit maximum and retains successful handles beside per-run submission failures.
- Observation invokes `squeue` once and consults `sacct` once only after the allocation leaves the queue.
- Accounting state and exit status come from the allocation row; peak RSS includes matching step rows such as `.batch`.
- Missing accounting is represented as `accounting_pending`, not guessed as success or failure.
- Collection is a read-only one-shot observation, and cancellation invokes `scancel` once for the exact job identity.
- Scheduling cadence belongs to the caller; this crate contains no sleep, wait, or polling loop.
- Qualification-cycle requests may name a governed external-artifact admission root. Missing or
  access-blocked records remain explicit blocked units with no submission action; an admitted
  record advances only to preparation of an artifact-bound qualification attempt.
- New container qualification submissions use scheduler specification v5 and persist evidence under
  `containers/evidence/attempts/<run-id>/<tool-id>/attempts/<campaign-id>/<attempt-id>` using the
  shared governed attempt-layout manifest. The specification result root is that same attempt root;
  declared tool outputs occupy its `outputs/` directory. Failed v5 attempts use the infra-owned
  archive, digest verification, and rollback transaction. Historical v3 and v4 specifications
  remain readable at their original paths but are never emitted or archived by current operations.

## Failure Semantics
| Failure | Detection | Expected remediation |
| --- | --- | --- |
| Docker or Apptainer missing | Backend executable is unavailable. | Install or configure the selected runtime. |
| Image missing | Backend cannot resolve or run the declared image. | Fix the image reference or registry access. |
| External access blocked | Admission records licensed artifact access as unavailable. | Supply and admit a lawful artifact or retain the blocker as the execution result. |
| External identity drift | Catalog, checksum, version evidence, tool identity, or stage binding differs from admission. | Re-admit the exact artifact against the current governed contract. |
| Permission error | Backend returns a filesystem permission failure. | Fix declared mount or output-root permissions. |
| Timeout | Backend exceeds the declared runtime limit. | Increase the limit or reduce input size. |
| Nonzero exit | Tool exits with a nonzero status. | Inspect captured stdout/stderr and tool arguments. |
| Slurm accounting delay | A job has left `squeue` but has no allocation row in `sacct`. | Observe again in a later caller-owned cycle. |
| Malformed scheduler output | Slurm output violates the typed parser contract. | Preserve raw scheduler evidence and correct the scheduler integration. |

## Change Rules
- Public field changes, invocation-hash changes, replay semantics changes, and backend invariant changes are breaking unless explicitly approved.
- Contract changes must update this file, `PUBLIC_API.md` when exports change, `DETERMINISM.md` when identity or replay changes, and the matching tests.
