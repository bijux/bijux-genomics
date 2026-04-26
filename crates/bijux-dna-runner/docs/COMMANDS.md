# Commands

`bijux-dna-runner` is a library crate. It does not expose CLI commands or `src/bin` entrypoints.

## Runtime Commands
None.

## Managed Backend Commands
This crate may construct and execute only these backend command families from typed execution specs:

- `docker run`
- `apptainer exec`
- declared observer commands through `execute_observer_command`
- declared low-level commands through `run_command` and `run_command_with_context`

## Non-Executing Commands
Replay never executes a backend command. `replay_run` reads an execution manifest and verifies artifacts on disk.

## Ownership Rules
- CLI parsing stays in CLI/API crates.
- Tool selection and stage planning stay in planner/domain crates.
- Backend command construction must start from typed contracts, not ad hoc strings.
- Network access is disabled by default unless runtime policy explicitly declares it.
