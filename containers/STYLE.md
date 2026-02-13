# Containers Style Guide

## Scope
Applies to all files under `containers/docker/**` and `containers/apptainer/**`.

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

## Apptainer Definition Section Order
Use this section order:
1. Header comment + `Bootstrap`/`From`
2. `%labels`
3. `%post`
4. `%environment` (if needed)
5. `%runscript`
6. `%test`
7. `%help`

## Bijux Template Marker
- Bijux-owned defs for newly admitted tools must include `BIJUX_TEMPLATE: v1` near the top.
- Template contract lives in `containers/apptainer/bijux/TEMPLATE.def.inc`.
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

Any other Docker base requires policy update in this file and matching lint update in `scripts/containers/lint.sh`.
