# Process transport

## Ownership

This crate owns process effects: capture and cancellation for runner and container callers.
It must not own domain, scheduler or container policy, serialization or command-line parsing.
The crate has no workspace production dependencies.

## Public API

`capture` accepts an owned command configuration and explicit deadline/output limits.
`wait_for_output`, `wait_for_status` and `isolate_process_group` support callers that
already own spawning and stream configuration. Results retain raw streams, child status
and explicit limit flags. Bounded capture currently requires Unix.

See [architecture](docs/ARCHITECTURE.md) for ownership and platform limits and the
[repository](../../README.md) for workspace entrypoints.

## Tests

Run `cargo nextest run -p bijux-dna-process` using the repository build configuration.
[Contract coverage](docs/TESTS.md) documents meaningful process and consumer cases.
