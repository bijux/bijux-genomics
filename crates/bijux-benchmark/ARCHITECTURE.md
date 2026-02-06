# ARCHITECTURE

Modules:
- `legacy/`: compatibility layer for older FASTQ benchmarking.
- `repo/`: repository boundary; traits in `repo/mod.rs`, SQLite impls in `repo/sqlite/`.
- `artifacts/` and `summarize/`: data IO + summary logic.
