# Benchmark Issues

This file tracks benchmark hard-wiring, publication drift, storage layout ambiguity, and missing governance hooks discovered while auditing the FASTQ benchmark surface and the supporting Lunarc workflow.

8. Push and pull behavior is spread across environment variables rather than a single benchmark workspace contract.
11. Remote storage currently contains both `.cache/results` and `.cache/bijux-dna-results` benchmark trees.
12. Remote storage currently contains both `.cache/reference` and `.cache/bijux-reference` trees.
13. Remote storage still contains non-cache roots such as `results`, `corpus_01`, and `extra-data` beside the governed `.cache` layout.
14. Duplicate remote roots make it ambiguous which tree is authoritative for publication.
15. Local benchmark results also mix a top-level stage mirror with a separate archival `home/.../.cache` mirror path.
20. `docs/benchmark/corpus-01-status.md` reports stale `fastq.trim_reads` coverage despite a more complete remote run.
30. Publication refresh depends on manually curated make targets rather than the governed contract list.
45. Benchmark renderers duplicate argument parsing instead of using a shared workspace/path resolver.
46. Benchmark renderers duplicate runtime summary calculations across many nearly identical scripts.
47. Benchmark renderers duplicate cohort and layout aggregation logic across many nearly identical scripts.
48. Benchmark renderers duplicate markdown rendering patterns across many nearly identical scripts.
49. Benchmark renderers mix path normalization, contract validation, and narrative rendering in the same files.
50. There is no single stage-agnostic renderer framework for corpus-01 FASTQ benchmark dossiers.
51. Many dossier files are named `lunarc.md`, which encodes the execution site into the published artifact name.
52. The published document naming scheme does not separate benchmark content from environment-specific provenance cleanly.
73. `containers/apptainer/lunarc/` remains the only governed location for Apptainer definitions.
74. The policy suite explicitly enforces `containers/apptainer/lunarc/`, which bakes a site name into repository structure.
75. Container documentation repeatedly names Lunarc as the canonical Apptainer authority.
76. The repository has no neutral `containers/apptainer/shared/` or equivalent location for non-site-specific definitions.
88. `fastq.correct_errors` publication currently depends on the presence of a run manifest in one mirror layout and a bench tree in another.
89. `fastq.trim_reads` publication currently depends on stale local mirrors unless the user manually re-syncs.
