# CONTRACT_VERSIONING

Contracts carry `contract_version { major, minor }`.

Compatibility rules:
- Backward compatible changes increment `minor`.
- Breaking changes increment `major` and require explicit migration notes.
- All serialized truth artifacts must include `contract_version`.
