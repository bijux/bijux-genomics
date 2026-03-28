# Benchmark Issues

This file tracks benchmark hard-wiring, publication drift, storage layout ambiguity, and missing governance hooks discovered while auditing the FASTQ benchmark surface and the supporting Lunarc workflow.

15. Local benchmark results also mix a top-level stage mirror with a separate archival `home/.../.cache` mirror path.
48. Benchmark renderers duplicate markdown rendering patterns across many nearly identical scripts.
49. Benchmark renderers mix path normalization, contract validation, and narrative rendering in the same files.
50. There is no single stage-agnostic renderer framework for corpus-01 FASTQ benchmark dossiers.
