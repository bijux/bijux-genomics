# FAILURES

| Failure | Detection | User remediation |
| --- | --- | --- |
| docker missing | docker client not found | install docker or use local runner |
| image missing | pull fails | check image ref or registry access |
| permission error | EACCES on mount | fix file permissions |
| OOM | exit code + stderr | increase memory or reduce input |
