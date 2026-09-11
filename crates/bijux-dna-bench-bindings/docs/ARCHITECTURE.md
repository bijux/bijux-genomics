# Binding evidence architecture

`serving` admits corpus declarations, selects integrated stage/tool bindings, applies
canonical capability facts and keeps governed serving rows. `expected` joins output
contracts with corpus assignments and readiness kinds, then constructs result identities,
artifact roles and paths. Both own their unchanged serialized models.

`observations` contains only consumed, nonserialized facts. Completeness and readiness
come from their canonical domain/planner owner; this crate does not reinterpret native
tool payloads or maintain a competing stage registry. `ports` separates serving and
expected acquisition. CLI adapters map existing records and canonical enums into those
facts. BAM definitions are cached for one serving request, including acquisition errors;
a subsequent request reacquires evidence. No global cache conceals changed declarations.

Expected acquisition preserves its seven-source order. Each top-level source is queried
once; nested providers may acquire their own inputs. Duplicate corpus stages, serving
bindings and output contracts are rejected before final publication. Original governed
row requirements and artifact paths remain enforced. Scheduler layout values are explicit
inputs, and the core owner supplies result IDs and stage constants.
