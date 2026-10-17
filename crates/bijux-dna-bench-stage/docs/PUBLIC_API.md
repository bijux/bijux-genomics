# Local stage public API

## Module Inventory

- execution: available with the `execution` feature; projects checked-resource stage commands, renders materialization argv and publishes command/fixture files from explicit caller inputs.
- repository: available with the `repository` feature; acquires matrix-backed inventories, accepts normalized rows and publishes admitted inventories.

The root exports `StageContractError`, readiness kinds, inventory/matrix/command records, schema constructors, schema identity constants and semantic inventory/command validators. These records retain canonical serialization identity behind compatibility aliases in benchmark model consumers.

The default feature set is empty. `execution` adds core, infrastructure and stage contracts; `repository` adds infrastructure and TOML decoding. Application callers supply canonical domain planners and native materializers. This owner does not execute native backends or discover application configuration. Structured command argv and relocatable scripts preserve the established persisted command contract.
