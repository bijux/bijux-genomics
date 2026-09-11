# Local HPC benchmark evidence

## Ownership

This crate owns deterministic watchdog simulations and their persisted reports. Pipeline evidence enters through typed benchmark models; scenario scheduling and assessment execute independently of application composition.

## Boundaries

Normal dependencies are benchmark models with default features disabled, canonical core identifiers and infrastructure filesystem primitives. This crate does not launch backend processes or discover application roots. The dependency boundary excludes the CLI, API, benchmark orchestration, planners and runner transports. Effects are limited to atomic report publication after input admission. Simulations establish policy examples, not remote execution or scientific performance.

## Public API

`watchdog` accepts an explicit validated DAG, path context and a scenario. It returns the existing typed simulation report. See [public contracts](docs/PUBLIC_API.md).

## Architecture

Scenario scheduling, evidence models, input admission and publication are separate modules. See [architecture](docs/ARCHITECTURE.md).

## Tests

Owner tests exercise invalid DAG admission, dependency ordering, retained output and deterministic scheduling. Original command tests preserve the CLI integration. See [tests](docs/TESTS.md).
