# Containers Style Guide

## Scope
Applies to all files under `containers/docker/**` and `containers/apptainer/**`.

Related governed surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [../docker/NONROOT_EXCEPTIONS.md](../docker/NONROOT_EXCEPTIONS.md)
- [../docker/ENTRYPOINT_EXCEPTIONS.md](../docker/ENTRYPOINT_EXCEPTIONS.md)
- [../apptainer/shared/NON_BIJUX_SOURCES.md](../apptainer/shared/NON_BIJUX_SOURCES.md)
- [../apptainer/shared/TEMPLATE.def.inc](../apptainer/shared/TEMPLATE.def.inc)

## Tone And Language
- Use imperative, technical English.
- Keep comments short and factual.
- Avoid conversational voice, marketing phrasing, and jokes.
- Avoid mixed casing for tool names in prose; use canonical executable names.

## Required Header
Every file must begin with the shared GPL header block used in this repository.

## Dockerfile Section Order
Use this section order, with section comments in this sequence:
1. `ARG`/`FROM` base definition
2. OCI `LABEL` metadata
3. Dependency installation
4. Source fetch/build/install (pinned)
5. Runtime wrapper or entrypoint
6. Smoke/default command (`CMD` or documented equivalent)

## Docker Security And Repro Policy
- Forbidden: `curl | bash`, `wget | sh`, or equivalent pipe-to-shell installers.
- Require digest-pinned `FROM` base image.
- APT strategy:
  - Prefer pinned package versions where practical.
  - At minimum, base image digest pinning is mandatory and package installation must be explicit.
- Prefer non-root runtime (`USER` non-root). If not feasible, document in `containers/docker/NONROOT_EXCEPTIONS.md`.
- Prefer JSON `ENTRYPOINT` to the tool binary and JSON `CMD` defaulting to `--help` or `--version`.
  - Temporary exceptions must be documented in `containers/docker/ENTRYPOINT_EXCEPTIONS.md`.
- Healthcheck is optional. If present, it must be deterministic and lightweight (typically `--version`), with explicit `--interval` and `--timeout`.

## Apptainer Definition Section Order
Use this section order:
1. Header comment + `Bootstrap`/`From`
2. `%labels`
3. `%environment`
4. `%post`
5. `%runscript`
6. `%test`
7. `%help`

## Standard Header Block (All `.def`)
Every Apptainer definition must include these comment markers near the top:
- `# Tool ID: <tool_id>`
- `# Version policy: ...`
- `# Upstream source: ...`
- `# Build date policy: ...`

## `%labels` Contract
Required labels:
- `org.opencontainers.image.source`
- `org.opencontainers.image.revision`
- `org.opencontainers.image.created`
- `org.opencontainers.image.licenses`
- `org.opencontainers.image.version`
- `org.opencontainers.image.tool`
- `org.opencontainers.image.title`

## `%environment` Contract
Must export deterministic runtime env:
- `PATH=...`
- `LC_ALL=C` (or equivalent deterministic locale)
- `TZ=UTC`
- Must not include user-specific absolute paths.

## `%post` Contract
- First non-empty line must be `set -eux`.
- No interactive prompts (`read -p`, `select`, `dialog`, `whiptail`).
- If `apt-get` is used, cleanup with `rm -rf /var/lib/apt/lists/*`.

## Floating Versions
- Production tools must not use floating values (`latest`, `main`, `master`, `head`, `unknown`) for `org.opencontainers.image.version`.
- Experimental/planned tools may use transitional values only when lock/provenance policy is still satisfied.

## Base Image Policy (Apptainer)
- `From:` must be digest-pinned (`@sha256:`).
- Allowed bases: `ubuntu`, `debian`, `python`, or approved `quay.io/*` sources.

## Download/Provenance Policy
- Network downloads in `%post` require checksum verification or explicit lock-policy marker.
- Non-bijux definitions must have provenance rows in `containers/apptainer/shared/NON_BIJUX_SOURCES.md`.

## Runtime UID Safety
- Definitions must avoid broad write permissions (`chmod 777` forbidden).
- Containers should remain non-root runnable and write only to intended runtime dirs.

## Bijux Template Marker
- Bijux-owned defs for newly admitted tools must include `BIJUX_TEMPLATE: v1` near the top.
- Template contract lives in `containers/apptainer/shared/TEMPLATE.def.inc`.
- Lint enforces marker presence for downstream bijux-owned defs.

## Wrapper Policy
- Prefer direct execution.
- If a wrapper is needed for stable `--version`/`--help`, it must be minimal and deterministic.

## Reproducibility Notes
- Pin all fetched sources to immutable commits/digests.
- Do not rely on floating branches/tags at build time.
- Keep build metadata in labels, not only comments.

## Docker Base Image Policy
Allowed base repositories (must still be digest pinned):
- `ubuntu` for general compiled-tool images.
- `python` for Python-first toolchains.
- `quay.io/biocontainers/bcftools` for the bcftools compatibility path.

Any other Docker base requires policy update in this file and matching lint update in `cargo run -p bijux-dna-dev -- containers run lint`.
