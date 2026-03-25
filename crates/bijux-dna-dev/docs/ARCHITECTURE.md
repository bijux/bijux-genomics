# bijux-dna-dev Architecture

`bijux-dna-dev` is organized as a small control-plane crate with four durable layers:

- `cli` parses the developer-facing command surface and hands off to the application layer.
- `application` coordinates commands, catalogs, and runtime services into stable workflows.
- `catalog` and `model` define the durable command vocabulary and typed outcomes.
- `commands` and `runtime` own repository-scoped effects such as filesystem mutations and delegated process execution.

This boundary keeps production runtime crates free of development-only automation concerns while still giving the workspace a typed, testable automation surface.
