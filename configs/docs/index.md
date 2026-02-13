# Docs Config

Purpose: canonical location for docs toolchain dependency pins.

Files:
- `requirements.txt`: MkDocs and docs build dependencies.
- `requirements.lock.txt`: lockfile mirror for deterministic docs dependency pin auditing.
- `mkdocs.toml`: pinned docs build contract consumed by scripts/tooling/docs-build.sh.
