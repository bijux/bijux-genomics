# FAILURE_TAXONOMY

## Authority
Failure kinds and remediation hints are owned by `src/failure.rs`.
Snapshots live in `tests/contracts/failure_hints.rs`.

## Failure classes
- ContractError: schema/validation failures and invariant violations.
- ToolError: tool execution failures or observer parse errors.
- EnvironmentError: infrastructure failures such as image or resource exhaustion.

## Failure kinds
- tool_exit: tool crashed or exited with error.
- contract_violation: schema or invariant failed.
- observer_parse: tool output could not be parsed.
- data_invalid: inputs failed validation.
- resource_exhaustion: time/memory limits exceeded.
- image_error: missing/invalid runtime image.

## Remediation hints
Hints are structured messages with severity and suggested actions.
Examples are validated in:
- `tests/contracts/failure_hints.rs`
- `tests/snapshots/failure_hint_adapter.json`
- `tests/snapshots/failure_hint_timeout.json`
- `tests/snapshots/failure_hint_invalid.json`
