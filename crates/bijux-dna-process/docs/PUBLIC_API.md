# bijux-dna-process Public API

## Module Inventory

There are no public modules. The crate root exposes the following operations and records.

| Symbol | Contract |
| --- | --- |
| `capture` | Spawn the supplied command with null stdin and piped outputs; return bounded raw evidence. |
| `CaptureLimits` | Positive capture deadline and combined raw stdout/stderr byte budget. |
| `SupervisedOutput` | Exit status, separate raw streams and explicit deadline/output-limit cancellation flags. |
| `SupervisedStatus` | Direct child exit status and deadline cancellation flag. |
| `isolate_process_group` | Arrange an isolated Unix group before spawning. |
| `wait_for_status` | Wait for a caller-owned child under its deadline. |
| `wait_for_output` | Unix-only collection for an already isolated child with configured pipes or files. |

## Stability

Consumers own argv, environment, domain admission and publication. A zero exit status
alone does not establish successful capture: inspect cancellation flags as well.
The high-level bounded API rejects unsupported platforms and invalid deadlines before
spawning. Low-level wait callers must own an isolated group. An absent low-level byte
budget requests complete binary capture and does not bound memory.

See [BOUNDARY.md](BOUNDARY.md), [ARCHITECTURE.md](ARCHITECTURE.md) and [TESTS.md](TESTS.md).
