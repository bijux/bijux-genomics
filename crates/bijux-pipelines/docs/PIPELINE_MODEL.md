# PIPELINE_MODEL

Pipelines compose stages and defaults into reproducible profiles.

## Profiles
A profile chooses a pipeline ID and applies explicit overrides.

## Override precedence
- profile overrides pipeline defaults
- pipeline defaults override global defaults

Example:
- global `trim_min_len = 20`
- pipeline default `trim_min_len = 25`
- profile override `trim_min_len = 30`
- effective value = `30`

## Rules
- Explicit overrides only.
- No implicit magic defaults.
- Override precedence: profile > pipeline > global.
