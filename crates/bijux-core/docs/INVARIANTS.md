# INVARIANTS

## ExecutionGraph
The validator enforces:
- **Acyclic**: no cycles in step dependencies.
- **Unique step IDs**: no duplicates across steps.
- **Resolvable edges**: every edge references existing steps.
- **Resolvable artifacts**: every artifact reference points to a declared artifact.

### Counterexamples
- A cycle between `step_a -> step_b -> step_a`.
- Two steps with the same `StepId`.
- An edge references a step that is not in the graph.

## RunManifest
The validator enforces:
- **Graph hash present** and matches the graph.
- **Contract version present**.
- **Input fingerprints present**.
- **Declared artifacts list is complete** and matches graph outputs.
- **Tool identity present** for each executed step.

### Counterexamples
- A manifest with no `contract_version`.
- Missing an artifact listed in the execution graph.
- Tool invocation missing image digest or version.
