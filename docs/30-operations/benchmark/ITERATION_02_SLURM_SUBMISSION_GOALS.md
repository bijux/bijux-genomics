# Iteration 02 Slurm Submission Goals

This report tracks the second delivery slice on `feat/deep-foundation-0502`.

| Goal | Status | Implementation summary |
| --- | --- | --- |
| G061 | done | Added `slurm submit-stage-benchmark` with stage/tool/sample selector, generated script, and mock/real submission mode. |
| G062 | done | Added `slurm submit-domain-benchmark` with domain expansion over planned campaign rows. |
| G063 | done | Added `slurm submit-cross-benchmark` with multi-domain validation and cross selection behavior. |
| G064 | done | Added `slurm submit-campaign` for full campaign submission from campaign config. |
| G066 | done | Submission writes `*.log` lifecycle files with scheduler IDs and encrypted bundle targets. |
| G067 | done | Submission writes operator-readable `*.out` placeholders with stable locations. |
| G068 | done | Submission writes operator-readable `*.err` placeholders with stable locations. |
| G071 | done | Added deterministic Slurm script generator with strict shell mode and structured metadata headers. |
| G073 | done | Added dependency graph handling from `jobs[].depends_on` plus sample-local ordering fallback, emitted as `--dependency=afterok:`. |
| G080 | done | Added `slurm copy-back-manifest` writer with path inventory and rsync hint for local investigation import. |

Additional delivered support:

- Campaign jobs support `name` and `depends_on` fields.
- Copy-back manifest includes script paths in addition to log/out/err/results/code targets.
