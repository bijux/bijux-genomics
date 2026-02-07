# Boundary Map

The boundary map defines which effects are allowed in each crate. It is the narrative counterpart to the policy tests.

| Crate | Allowed effects | Forbidden effects |
| --- | --- | --- |
| bijux-core | None | IO, process, network |
| bijux-engine | Filesystem writes under run layout | Process spawn, docker APIs, network |
| bijux-runtime | Filesystem writes under run layout | Process spawn, docker APIs, network |
| bijux-runner | Process spawn, docker, mounts | Domain logic, planner selection |
| bijux-api | Filesystem writes under run layout | Direct runner internals |
| bijux-stages-* | None (parsing only) | Process spawn, docker, network |
| bijux-stage-contract | None | Process spawn, docker, network |
| bijux-planner-* | None | Process spawn, docker, network |
| bijux-pipelines | None | Process spawn, docker, network |
| bijux-analyze | Filesystem reads/writes | Process spawn, docker, network |
| bijux-benchmark | Filesystem reads/writes | Process spawn, docker, network |
| bijux-domain-* | None | Process spawn, docker, network |
| bijux-environment | Filesystem reads | Process spawn, docker, network |
| bijux-infra | None (pure helpers) | Process spawn, docker, network |
| bijux-cli | Delegates to API | Direct runner internals |
| bijux-environment-qa | Docker/network (allowlisted) | Production deps |
