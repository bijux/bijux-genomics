# HPC Campaign Profiles

These are baseline campaign profiles for Slurm dry-run and preflight validation.

- `lunarc-small.toml`: Lunarc-oriented defaults.
- `generic-small.toml`: generic Slurm defaults.
- `cross-mini.toml`: cross-domain fixture with explicit handoff dependency.
- `site-profiles/lunarc.toml`: Lunarc site defaults.
- `site-profiles/generic.toml`: generic site defaults.

Secrets must not be committed in these profiles. Use `security.env_file` and user overrides.

Resource templates can be selected globally with `resources.default`, or by stage family via
`resources.stage_defaults`.
