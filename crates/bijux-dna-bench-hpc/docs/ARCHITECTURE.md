# Local HPC evidence architecture

`watchdog::admission` validates the canonical benchmark DAG schema, identity and topology, then requires dependency-first node iteration before any output effects. Each scenario owns its scheduling and assessment: no global wait, failure isolation, partial resume and completion rules. Model records preserve the existing serialized field names and schema identity. Publication uses infrastructure atomic JSON writing.

The caller acquires the DAG and supplies all path context. No CLI parsing, global working-directory lookup, backend process launch or scheduler transport belongs to this crate. The CLI composes the existing pipeline validator with this owner. Readiness orchestration can consume these records without importing a frontend. Native dependency closure is measured separately; model dependencies can still bring their own domain contracts. This split establishes an independently compilable owner but does not itself prove a hosted latency reduction.

Simulation timings and injected failures are synthetic policy evidence. Per-file atomic replacement does not provide a multi-artifact transaction or a remote readiness claim.


The readiness service groups inventory/materialization, harness observations, corpus families, pipeline/watchdog checks and campaign/SLURM checks by responsibility. Provider projections are nonserialized views of consumed fields; canonical provider reports and the persisted readiness schema retain their existing owners. Request paths are supplied explicitly. Only a mock campaign submission port is available.

Dry-run assessment separates provider acquisition, cross-provider checks, summary/behavior projection and canonical publication/replay. Provider results use benchmark-model types directly. A request acquires each top-level provider once; nested provider acquisition remains owned by its adapter. Replay reacquires evidence on every call, preserving drift detection.
