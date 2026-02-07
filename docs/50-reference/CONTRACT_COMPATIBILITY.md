# Contract Compatibility

## What
Compatibility rules for contract evolution.

## Why
Supports forward/backward compatibility.

## Non-goals
- Full migration tooling.

## Contracts
- New fields must be additive.

## Examples
- Optional fields are safe additions.

## Failure modes
- Breaking field removal requires major bump.
