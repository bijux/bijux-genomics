# bijux-pipelines scope

Defines canonical pipeline profiles, defaults ledgers, and stable profile metadata across FASTQ,
BAM, and cross-domain workflows. This crate owns pipeline IDs, defaults ledgers, and profile
capabilities, and it must not depend on execution/runtime machinery.

This crate is contract-only: it declares scientific intent and defaults; it does not plan, execute,
or analyze runs.
