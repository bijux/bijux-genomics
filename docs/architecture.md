# Bijux Architecture Ledger

## Overview (ASCII)

```
            +------------------+
            |    bijux-cli     |
            |  (orchestration) |
            +---------+--------+
                      |
                      v
            +------------------+
            |   bijux-engine   |
            |  plan/execute    |
            +---------+--------+
                      |
          +-----------+-----------+
          |                       |
          v                       v
 +------------------+   +------------------+
 | bijux-domain-*   |   |  bijux-core      |
 | contracts/specs  |   | shared schemas   |
 +------------------+   +------------------+
```

## Known Compromises

- CLI currently hosts FASTQ execution orchestration for bench runs.
- Engine executor contains hardcoded container command logic for some tools.
- Coverage is improving, but not yet enforced in CI.

## Postponed / Explicitly Deferred

- Scheduler integrations (Slurm, Nextflow, CWL)
- Distributed execution
- Cross-domain pipeline composition
- Report visualizations (dashboards, notebooks)

## Intentional Boundaries

- Domains must be pure: no engine/runtime/container dependencies.
- Engine must not import domain crates.
- Core must remain domain-agnostic.

