# bijux-dev-dna Scope

`bijux-dev-dna` owns the versioned development control plane for this workspace.

It is responsible for:
- cataloging repository automation commands and checks,
- coordinating workspace-level maintenance workflows,
- enforcing repository-facing contracts that are not part of production pipeline execution,
- hosting process and filesystem effects that belong to developer automation rather than runtime execution.

It is not responsible for:
- production FASTQ, BAM, or VCF pipeline planning,
- stage execution inside runtime crates,
- domain semantics that belong in the domain and planner crates.
