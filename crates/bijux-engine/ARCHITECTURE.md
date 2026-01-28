# Engine Architecture

The engine is split into internal domains. Every file in `crates/bijux-engine/src` belongs to exactly one domain, and cross-domain communication must go through canonical types in `types/`.

## Domains

- planner
  - Purpose: decide what to run
  - Owns: tool normalization, manifest loading, stage/tool resolution, RunPlan creation
  - Must not: touch containers, touch metrics, create output directories

- executor
  - Purpose: run tools and return raw results
  - Owns: container execution, process invocation, raw stdout/stderr capture
  - Must not: interpret metrics, apply invariants, make policy decisions

- observer
  - Purpose: measure and compute metrics
  - Owns: runtime/memory collection, metric extraction, delta computation
  - Must not: make execution decisions, enforce invariants

- validator
  - Purpose: gate correctness
  - Owns: invariants, contracts, schema validation
  - Must not: run tools, compute metrics, create plans

- composer
  - Purpose: pipelines and meta-stages
  - Owns: preprocess, replay, image QA, benchmark orchestration
  - Must not: bypass validator or compute metrics outside observer

- types
  - Canonical boundary types: RunPlan, ToolInvocation, StageResult, MetricSet, ExecutionContext
  - Utility: trace mode and logging helpers

- errors
  - Engine error taxonomy

## Trace Mode

Set `BIJUX_TRACE_ENGINE=1` to print:
- planner decisions
- execution steps
- validation gates

## Dependency Rules

- planner does not import executor/observer/validator/composer internals
- executor does not import planner/observer/validator internals
- observer does not import planner/executor/validator internals
- validator does not import planner/executor/observer internals
- composer may orchestrate all domains via types

Tests enforce these rules.
