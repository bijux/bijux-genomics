# ARCHITECTURE

Command tree lives under `src/commands/` with CLI parsing in `src/commands/cli/`.
Helpers are split by concern:
- `commands/formatting.rs`
- `commands/validation.rs`
- `commands/rendering.rs`
