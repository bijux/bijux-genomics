# FAILURES

| Failure | Detection | User remediation |
| --- | --- | --- |
| docker missing | docker client not found | install docker (local runner not supported) |
| image missing | pull fails | check image ref or registry access |
| permission error | EACCES on mount | fix file permissions |
| OOM | exit code + stderr | increase memory or reduce input |

## Failure semantics
- timeouts: normalized to a timeout error and captured in records as a failed step.
- OOM: normalized via exit code + stderr marker (backend-specific), recorded as failure.
- nonzero exit: always recorded as failure; stdout/stderr captured verbatim.
