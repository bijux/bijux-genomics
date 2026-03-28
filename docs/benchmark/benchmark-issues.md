# Benchmark Issues

This file tracks benchmark hard-wiring, publication drift, storage layout ambiguity, and missing governance hooks discovered while auditing the FASTQ benchmark surface and the supporting Lunarc workflow.

8. Push and pull behavior is spread across environment variables rather than a single benchmark workspace contract.
11. Remote storage currently contains both `.cache/results` and `.cache/bijux-dna-results` benchmark trees.
12. Remote storage currently contains both `.cache/reference` and `.cache/bijux-reference` trees.
14. Duplicate remote roots make it ambiguous which tree is authoritative for publication.
15. Local benchmark results also mix a top-level stage mirror with a separate archival `home/.../.cache` mirror path.
48. Benchmark renderers duplicate markdown rendering patterns across many nearly identical scripts.
49. Benchmark renderers mix path normalization, contract validation, and narrative rendering in the same files.
50. There is no single stage-agnostic renderer framework for corpus-01 FASTQ benchmark dossiers.
88. `fastq.correct_errors` publication currently depends on the presence of a run manifest in one mirror layout and a bench tree in another.
89. `fastq.trim_reads` publication currently depends on stale local mirrors unless the user manually re-syncs.
