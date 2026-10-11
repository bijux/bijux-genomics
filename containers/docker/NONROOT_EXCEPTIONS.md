# Docker Non-Root Exceptions

Purpose: Document tool images that require a root runtime and the upstream or
scientific constraint that prevents an unprivileged runtime.

Policy:
- Preferred: set an explicit non-root `USER` in the final stage.
- Exception path: if non-root is not yet supported, tool ID must be listed here with rationale.

## Exceptions

| tool_id | rationale | owner |
|---|---|---|
