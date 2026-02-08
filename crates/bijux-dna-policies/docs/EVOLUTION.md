# Policy Evolution

## What
Defines how to add new policies without creating policy sprawl.

## Why
Ensures policies are centralized and documented.

## Non-goals
- Adding policy logic outside `bijux-dna-policies`.

## Contracts
1. New policies must live in `bijux-dna-policies`.
2. One entrypoint per policy area (deps/surface/data/tooling).
3. No duplication of policy logic in other crates.
4. Add a doc entry in `docs/INDEX.md`.
5. Add fixtures/snapshots in the crate under test.

## Examples
- Adding `no_new_bucket_modules.rs` under `tests/surface` and listing it in INDEX.md.

## Failure modes
- Policy added without documentation fails `policy_docs_anchor`.

## Checklist
- [ ] Add test under `crates/bijux-policies/tests/...`
- [ ] Document in `docs/INDEX.md`
- [ ] Add allowlist/exception if required
- [ ] Update snapshots if necessary
