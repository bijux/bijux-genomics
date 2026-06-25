# Benchmark Configs

This directory is reserved for tracked benchmark-specific configuration that should not remain
under disposable local run roots.

The local benchmark matrix and compatibility surfaces live under `local/`.
That includes governed decision inputs such as `local/stage-scoring.toml`, which turns active
stage benchmark evidence into explicit recommendation-scoring rules.
The governed local benchmark pipeline DAG configs live under `pipelines/local/`.
The governed benchmark HPC campaign profiles live under `hpc/campaign/`.
