# EXAMPLE_TEMPLATE

## Purpose
Define the required structure for runnable example READMEs.

## Scope
Applies only to example READMEs in directories that ship an
[example.toml](../../examples/_template/example.toml).

## Non-goals
- Replacing per-example scientific interpretation details.
- Defining recipe-only benchmark docs or corpus README structure.

## Contracts
- Every example README must include Step 1 through Step 4 sections in order.
- Output and interpretation sections are mandatory.
- Runnable example governance is defined in [examples/POLICY.md](../../examples/POLICY.md).
- Recipe-only benchmark docs listed in [examples/RECIPE_ONLY.txt](../../examples/RECIPE_ONLY.txt)
  are intentionally out of scope.
- Output expectations are governed by [EXAMPLE_RUNNER_CONTRACT.md](EXAMPLE_RUNNER_CONTRACT.md).

## Required Step Pattern
1. **Step 1: Containers Ready**
- Confirm required container images/tools are available.
- Document exact command used for readiness checks.

2. **Step 2: Bench Run**
- Execute the benchmark/example command with explicit profile and input.
- Include deterministic flags and isolate usage where required.

3. **Step 3: Collect Artifacts**
- List all produced artifacts (logs, metrics, traces, reports).
- Provide canonical output paths.

4. **Step 4: Analyze Results**
- Describe how to inspect outputs and derive the verdict.
- Include key thresholds or pass/fail rules.

## Required Output Section
- Enumerate generated files and expected locations.
- State reproducibility and determinism expectations.

## Required Interpretation Section
- Define success criteria.
- Define common failures and immediate debug actions.
