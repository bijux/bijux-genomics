# Execution publication architecture

Schema and report identities are separate from scientific payloads. Native files
are copied byte-for-byte and retained at their original physical execution paths.
Only the public benchmark stage-result output addresses are projected; commands,
native manifests and native checksum representations remain unchanged.

Prior inventories authorize replacement and retirement. Unknown files and
nonempty directories survive. Old execution files remain in the inventory because
downstream claims may reference them; dependency-aware retention is a separate
responsibility. Haplotype and variant workflows choose distinct persisted
inventory names/schemas while sharing this mechanism.

File replacements are individually atomic, but the publication is not a multi-file
transaction. Concurrent mutation of one output root is unsupported. Admission
rejects symlinked ancestry; callers must preserve exclusive ownership between
admission and writes.
