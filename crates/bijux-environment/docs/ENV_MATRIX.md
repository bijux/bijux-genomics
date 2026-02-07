# ENV_MATRIX

Supported environment kinds and resolution rules.

## Supported kinds
- Docker
- Apptainer
- Singularity

## Resolution precedence
1. Explicit platform name argument (CLI/API).
2. `BIJUX_PLATFORM` environment variable.
3. Default platform from `platforms.toml`.

## Caching rules
- Image metadata is read from the local catalog first.
- Digest resolution is deterministic for the same catalog + platform.
- Reference assets are cached under `~/.cache/bijux/references`.
