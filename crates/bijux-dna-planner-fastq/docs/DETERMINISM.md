# DETERMINISM

Same inputs → same plan JSON ordering + same graph hash.

Inputs include pipeline id, tool allow/deny lists, and profile overrides.
Ordering is canonicalized before hashing.
