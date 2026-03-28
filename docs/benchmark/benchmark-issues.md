# Benchmark Issues

This file tracks benchmark hard-wiring, publication drift, storage layout ambiguity, and missing governance hooks discovered while auditing the FASTQ benchmark surface and the supporting Lunarc workflow.

1. Hardcoded local benchmark mirror root in `makes/bin/corpus_01_fastq_benchmark_support.py` points to `/Users/bijan/bijux/bijux-dna-results`.
2. Multiple benchmark renderers still default `--corpus-root` to `/home/bijan/bijux/corpus_01`.
3. `configs/runtime/platforms.toml` hardcodes a user-specific Lunarc Apptainer SIF directory.
4. The benchmark tooling does not make the private frontend repo root under `/home/bijan/bijux/...` versus the shared benchmark cache root under `/home/bijan/lu2024-12-24/.cache` explicit enough.
5. `hpc_lunarc_pull` defaults the local pull base to `~/bijux/bijux-dna-results` instead of the actual governed local results workspace.
6. `hpc_lunarc_pull` encodes a timestamped destination convention rather than a stable benchmark mirror contract.
7. `hpc_lunarc_push` and `hpc_lunarc_pull` are cluster-specific command names instead of generic benchmark environment sync commands.
8. Push and pull behavior is spread across environment variables rather than a single benchmark workspace contract.
9. There is no checked-in schema-bound benchmark workspace configuration file covering local and remote roots.
10. Benchmark publication support derives local roots from code constants instead of configuration.
11. Remote storage currently contains both `.cache/results` and `.cache/bijux-dna-results` benchmark trees.
12. Remote storage currently contains both `.cache/reference` and `.cache/bijux-reference` trees.
13. Remote storage still contains non-cache roots such as `results`, `corpus_01`, and `extra-data` beside the governed `.cache` layout.
14. Duplicate remote roots make it ambiguous which tree is authoritative for publication.
15. Local benchmark results also mix a top-level stage mirror with a separate archival `home/.../.cache` mirror path.
16. The local mirror contract is not documented anywhere under `docs/benchmark`.
17. The local results workspace contains `.DS_Store` files, which pollute benchmark mirrors.
18. `docs/benchmark/corpus-01-status.md` reports `fastq.correct_errors` as missing even though Lunarc has a completed benchmark tree.
19. `docs/benchmark/corpus-01-status.md` reports `fastq.screen_taxonomy` as missing even though Lunarc has a completed benchmark tree.
20. `docs/benchmark/corpus-01-status.md` reports stale `fastq.trim_reads` coverage despite a more complete remote run.
21. `docs/benchmark/corpus-01-publication-findings.json` is empty even while `corpus-01-status.md` reports 27 issues.
22. `docs/benchmark/corpus-01-results-status.md` audits only 17 published stages and does not help close the remaining publication gap.
23. The publication audit and the mirror audit are separate ledgers and can diverge.
24. The published dossier refresh target in `makes/benchmarks-fastq.mk` omits `fastq.correct_errors`.
25. The published dossier refresh target in `makes/benchmarks-fastq.mk` omits `fastq.screen_taxonomy`.
26. The published dossier refresh target in `makes/benchmarks-fastq.mk` omits `fastq.extract_umis`.
27. The published dossier refresh target in `makes/benchmarks-fastq.mk` omits `fastq.remove_duplicates`.
28. The published dossier refresh target in `makes/benchmarks-fastq.mk` omits `fastq.deplete_host`.
29. The published dossier refresh target in `makes/benchmarks-fastq.mk` omits `fastq.deplete_reference_contaminants`.
30. Publication refresh depends on manually curated make targets rather than the governed contract list.
31. Benchmark stage exclusions are embedded in Python support code instead of shared configuration.
32. The benchmark support module localizes `/results/` and `/bijux-dna-results/` paths but not remote `extra-data` paths.
33. `fastq.screen_taxonomy` run manifests record remote taxonomy database paths that local publication does not normalize.
34. `fastq.deplete_host` and `fastq.deplete_reference_contaminants` depend on extra-data indexes but do not share a common path resolver contract.
35. `default_extra_data_root()` in the Python support module assumes benchmark assets always live under `extra-data/benchmark`.
36. `default_screen_taxonomy_database_root()` hardcodes the taxonomy database directory formula in Python.
37. `default_host_reference_index_root()` hardcodes the host depletion index directory formula in Python.
38. `preferred_report_run_root()` assumes a single local results topology and cannot represent mixed mirror roots cleanly.
39. `default_results_stage_root()` assumes `corpus_root.parent / results / <corpus> / <stage>` rather than a governed workspace contract.
40. `default_local_results_stage_root()` assumes the local mirror is always rooted at one fixed macOS path.
41. `infer_cache_root()` only works when the path contains a literal `.cache` segment.
42. `benchmark_runtime_env()` derives `BIJUX_HPC_ROOT` from `.cache` heuristics instead of explicit config.
43. `load_published_sample_metadata()` is anchored to `fastq.validate_reads` as the fallback metadata source.
44. `load_published_sample_metadata()` hardcodes an expected total of 20 samples rather than loading the count from a contract file.
45. Benchmark renderers duplicate argument parsing instead of using a shared workspace/path resolver.
46. Benchmark renderers duplicate runtime summary calculations across many nearly identical scripts.
47. Benchmark renderers duplicate cohort and layout aggregation logic across many nearly identical scripts.
48. Benchmark renderers duplicate markdown rendering patterns across many nearly identical scripts.
49. Benchmark renderers mix path normalization, contract validation, and narrative rendering in the same files.
50. There is no single stage-agnostic renderer framework for corpus-01 FASTQ benchmark dossiers.
51. Many dossier files are named `lunarc.md`, which encodes the execution site into the published artifact name.
52. The published document naming scheme does not separate benchmark content from environment-specific provenance cleanly.
53. The benchmark docs tree does not contain a single index of dossier freshness and remote source roots.
54. The docs tree does not record which local mirror root was used for each dossier refresh.
55. The docs tree does not record whether a dossier was rendered from a remote path or a local mirror.
56. The mirror sync process does not emit a per-stage freshness manifest under version control.
57. `configs/hpc/lunarc_sync_profiles.toml` controls rsync include and exclude files but not benchmark workspace semantics.
58. There is no benchmark-specific sync profile that explicitly targets the governed `.cache` tree.
59. The dev commands still refer to `LUNARC_ROOT`, `LUNARC_RESULTS_DIR`, and related variables rather than a neutral benchmark environment model.
60. The current sync helpers do not record extra-data dependencies alongside results pulls.
61. The current sync helpers do not record which local destination path corresponds to which remote `.cache` subtree.
62. The current sync helpers do not validate the private-repo root and shared-cache roots as separate contracts.
63. The current sync helpers do not reject stale duplicate roots when both `.cache/results` and `.cache/bijux-dna-results` are present.
64. The current push helper syncs repo content, but not a structured benchmark environment contract.
65. The current push helper does not document clearly that repo sync belongs on the private frontend home while benchmark artifacts belong on shared storage.
66. `shared_cache_root()` in `env_registry_commands.rs` forces `.cache` under any HPC root instead of reading a workspace contract.
67. `env_registry_commands.rs` hardcodes `bijux-dna-container`, `corpus_01`, and `results` subdirectory names.
68. `env_registry_commands.rs` implicitly treats `.cache` as the only valid shared-root layout instead of a configurable benchmark environment.
69. `resolved_container_dir()` in `crates/bijux-dna-environment` still falls back to derived Lunarc cache conventions instead of a platform config contract.
70. Environment resolution tests still use `lunarc-apptainer` and `/scratch/cache-root` as named fixtures.
71. `configs/runtime/platforms.toml` ships a runner profile whose default path belongs to one user account.
72. The platform config does not separate a portable default from cluster-local overrides.
73. `containers/apptainer/lunarc/` remains the only governed location for Apptainer definitions.
74. The policy suite explicitly enforces `containers/apptainer/lunarc/`, which bakes a site name into repository structure.
75. Container documentation repeatedly names Lunarc as the canonical Apptainer authority.
76. The repository has no neutral `containers/apptainer/shared/` or equivalent location for non-site-specific definitions.
77. Some examples still publish `/scratch/$USER/...` as the output convention.
78. The benchmark publication workflow has no shared command for “sync remote results, render dossiers, refresh audits”.
79. There is no machine-readable remediation queue for publication issues.
80. There is no per-stage ownership/status field for unresolved benchmark documentation gaps.
81. `docs/benchmark/fastq.correct_errors/corpus-01-method.md` exists without the corresponding published `corpus-01` dossier directory.
82. `docs/benchmark/fastq.screen_taxonomy/corpus-01-method.md` exists without the corresponding published `corpus-01` dossier directory.
83. The benchmark audit script reports missing corpus directories but does not explain where the completed remote run actually lives.
84. The benchmark audit script does not surface duplicate-result-root ambiguity as a first-class issue type.
85. The benchmark audit script does not cross-check `corpus-01-publication-findings.json` freshness.
86. The benchmark audit script does not confirm that the published dossier source run is the newest available matching run.
87. The benchmark audit script does not warn when make targets omit governed publication stages.
88. `fastq.correct_errors` publication currently depends on the presence of a run manifest in one mirror layout and a bench tree in another.
89. `fastq.trim_reads` publication currently depends on stale local mirrors unless the user manually re-syncs.
90. `fastq.screen_taxonomy` publication depends on a local mirror of the taxonomy database lineage file, but there is no governed sync command for that extra-data dependency.
91. Benchmark support uses Python-only contracts, so Rust tooling cannot validate the same workspace path assumptions directly.
92. There is no repo check that fails on hardcoded `/Users/bijan/` paths in benchmark tooling.
93. There is no repo check that fails on hardcoded `/home/bijan/` paths in benchmark tooling.
94. There is no repo check that fails on hardcoded `lunarc` host names in benchmark tooling.
95. There is no repo check that ensures all governed corpus-01 benchmark stages have render targets.
96. There is no repo check that ensures all governed corpus-01 benchmark stages have publication audit coverage.
97. The benchmark support layer still treats the local mirror as a special case rather than a first-class configured environment.
98. The repository lacks a single documented procedure for moving the benchmark workflow from Lunarc to another cluster using config only.
99. The repository lacks a single documented procedure for mirroring the governed `.cache` tree into the local results workspace with a stable path contract.
100. The benchmark documentation surface still reflects historical storage decisions instead of one clear, durable benchmark workspace model.
