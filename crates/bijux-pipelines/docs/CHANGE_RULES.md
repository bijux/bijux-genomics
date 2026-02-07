# Change Rules

## What
Defines breaking vs non‑breaking changes for `bijux-pipelines`.

## Why
Prevents silent contract drift.

## Non-goals
- Automatic versioning.

## Contracts
- Breaking changes require explicit approval and snapshot updates.

## Examples
- Changing a public contract field is breaking.

## Failure modes
- Unversioned breaking changes are rejected in CI.
