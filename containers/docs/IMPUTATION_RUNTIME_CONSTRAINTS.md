# Imputation Runtime Constraints

Purpose: define minimum runtime envelopes for core VCF downstream tools on frontend/HPC shared environments.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [IMPUTATION_NETWORK_POLICY.md](IMPUTATION_NETWORK_POLICY.md)
- [FRONTEND_BUILD_AUTHORITY.md](FRONTEND_BUILD_AUTHORITY.md)
- [../../docs/30-operations/HPC_FRONTEND_RUNBOOK.md](../../docs/30-operations/HPC_FRONTEND_RUNBOOK.md)

Contract:
- These are planning baselines, not performance guarantees.
- `cpu_threads`, `ram_gb`, and `scratch_gb` must be declared for each core tool.
- Pipeline/runtime checks may refuse execution when requested resources are below minimums.

| tool_id | cpu_threads_min | ram_gb_min | scratch_gb_min | notes |
|---|---:|---:|---:|---|
| `glimpse` | 4 | 8 | 20 | lowcov chunk/ligate workflows; chunk fanout drives scratch. |
| `impute5` | 4 | 12 | 30 | requires map + reference panel locality for stable throughput. |
| `minimac4` | 4 | 12 | 30 | phased input required; m3vcf preparation may increase scratch. |
| `shapeit5` | 4 | 12 | 20 | map-backed phasing; PBWT depth can increase RAM pressure. |
| `beagle` | 4 | 10 | 20 | joint phasing/imputation mode can increase walltime and IO. |
| `eagle` | 4 | 10 | 20 | license and upstream constraints apply; only use when enabled. |
| `bcftools` | 2 | 4 | 10 | helper/normalization/indexing baseline. |
| `plink2` | 4 | 8 | 20 | downstream QC/ROH/structure helpers; large cohorts increase RAM. |
