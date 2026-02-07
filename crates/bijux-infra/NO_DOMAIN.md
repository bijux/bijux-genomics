# NO_DOMAIN

`bijux-infra` is an infrastructure-only crate.

- It must not define or reference domain concepts (stages, tools, pipelines, metrics).
- It must not depend on planner, stage, or domain crates.
- It provides generic utilities only (IO, paths, logging, formats).
