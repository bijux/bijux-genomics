# Change Routing Guide

Owner: Architecture  
Scope: contributor routing for new genomics changes  
Last reviewed: 2026-04-30  
Contract version: v1

## Purpose

Route common change types to the right crate, config path, fixture path, and proof set before implementation starts.

## Routing table

| Change type | Crate(s) to edit first | Config/domain authority | Fixture/test anchor | Required proof |
| --- | --- | --- | --- | --- |
| New FASTQ/BAM/VCF stage ID or domain vocabulary | `bijux-dna-domain-*`, then `bijux-dna-domain-compiler` if generated surfaces change | `domain/<domain>/stages/*.yaml`, `domain/<domain>/index.yaml` | `domain/<domain>/fixtures/`, domain crate contract tests | domain fixture, domain test, compiler parity if generated output changes |
| New backend binding for an existing stage | `bijux-dna-domain-*`, `bijux-dna-stages-*`, planner crate for the domain | `domain/<domain>/tools/*.yaml`, `configs/ci/registry/*.toml` | `domain/<domain>/fixtures/<stage-id>/`, stage contract tests | tool registry proof, stage invocation tests, planner plan snapshots |
| New stage parameter or parameter contract | domain crate types, planner crate parameter resolution, stage crate parsing if runtime-visible | domain YAML plus pipeline/default config if applicable | planner/unit snapshots and negative fixtures | canonicalization test, negative fixture, plan snapshot |
| New workflow/profile/template default | `bijux-dna-pipelines`, planner crate if explain output changes | `configs/runtime/profiles/*.toml`, pipeline defaults ledgers | planner contract snapshots | defaults ledger diff, explain snapshot, invariant test |
| Runtime/executor policy change | `bijux-dna-runtime`, `bijux-dna-runner`, `bijux-dna-environment`, `bijux-dna-api` | `configs/runtime/*.toml` | runtime or runner contract fixtures | runtime contract test, run layout proof, operator-facing docs if behavior changes |
| New API request/response or route field | `bijux-dna-api` | request/response contracts in crate code | `crates/bijux-dna-api/tests/contracts/`, `tests/snapshots/` | route contract tests, schema snapshot update |
| New evidence/report rule | `bijux-dna-analyze`, `bijux-dna-runtime`, `bijux-dna-api` | evidence/report schema owners in code | analyze and API contract tests | evidence bundle fixture, schema snapshot, negative gap test |
| New science caveat or scientific policy rule | domain crate, `bijux-dna-science`, `bijux-dna-policies` | domain docs/specs and science docs | domain/science fixtures | explicit advisory/enforced label, negative proof if it can be over-claimed |
| New benchmark/report comparison surface | `bijux-dna-bench`, `bijux-dna-bench-model`, `bijux-dna-analyze` | `configs/bench/*.toml` if knob-driven | bench/analyze contract tests | comparison fixture, snapshot, caveat docs |
| New governance or CI guardrail | `bijux-dna-policies`, `bijux-dna-dev`, workspace docs/configs | `configs/ci/*.toml`, `docs/40-policies/` | policy tests in `crates/bijux-dna-policies/tests` | failing-before/passing-after policy test and docs pointer |

## Example PR checklists

### New stage

- update domain YAML and any generated/index authority
- add positive fixture and at least one refusal fixture
- update planner/stage contract coverage
- update docs only after the tests point to the truth

### New backend

- admit backend in domain/tool registry
- implement invocation/parsing contract in the stage crate
- add planner selection or capability proof
- add registry/tooling parity checks if the backend is production-visible

### New parameter

- define typed parameter shape and canonicalization
- prove defaults/user overrides resolve deterministically
- add at least one refusal for invalid or incoherent values
- update explain or evidence output if operators can observe the parameter

### Runtime policy change

- keep scientific policy and operational policy separate
- prove run layout, retry, timeout, or executor behavior under tests
- update operator docs for new failure/remediation behavior
- avoid hidden CLI-only logic
