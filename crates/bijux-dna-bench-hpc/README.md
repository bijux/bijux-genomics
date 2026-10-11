# Local HPC benchmark evidence

## Ownership

This crate owns local submission readiness assessment and deterministic watchdog simulations. Explicit provider contracts supply stage, harness, corpus, pipeline, SLURM and local campaign evidence. Pipeline simulation consumes typed DAG models. Persisted readiness reports retain their benchmark-model authority.

## Boundaries

Normal dependencies are benchmark models with default features disabled, canonical core identifiers and infrastructure filesystem primitives. This crate does not launch backend processes or discover application roots. The dependency boundary excludes the CLI, API, benchmark orchestration, planners and runner transports. Owner effects are local diagnostic publication and profile support preparation through infrastructure helpers. Injected adapters own their declared local acquisition and publication operations; the campaign interface exposes mock submission only. Simulations establish policy examples, not remote execution or scientific performance.

## Public API

`readiness` accepts explicit input paths, provider contracts and an output path; it publishes all 99 check outcomes before reporting assessment failure. `watchdog` accepts an explicit validated DAG, path context and a scenario. It returns the existing typed simulation report. See [public contracts](docs/PUBLIC_API.md).

## Architecture

Scenario scheduling, evidence models, input admission and publication are separate modules. See [architecture](docs/ARCHITECTURE.md).

## Tests

Owner tests exercise invalid DAG admission, dependency ordering, retained output and deterministic scheduling. Original command tests preserve the CLI integration. See [tests](docs/TESTS.md).


Local submission readiness accepts typed stage, harness, corpus, pipeline, SLURM and local campaign evidence ports. CLI composition provides current adapters; this owner does not depend on command parsing or remote submission. Existing report schemas and all 99 diagnostic checks remain authoritative.

`dry_run` owns nine cross-provider decisions, report publication and fresh replay. One request evidence record preserves acquisition failures without duplicate report/error inputs. The CLI supplies canonical provider paths, local collectors and path admission. No remote submission method is exposed.

`candidate` admits canonical asset/execution evidence and explicit job/resource declarations, selects bounded dependency-free representatives and publishes the existing candidate schema. Result locations are observed once per eligible job and retained for stop-condition projection.
