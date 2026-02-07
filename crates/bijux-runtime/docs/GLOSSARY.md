# GLOSSARY

This glossary defines the runtime vocabulary used across bijux runtime contracts.

- **run**: A single end-to-end execution of a pipeline for a given input and profile.
- **layout**: The canonical directory structure for a run, including where manifests,
  records, and artifacts are written.
- **manifest**: The canonical summary of a run, including graph hash, contract
  versions, tool identity, and declared artifacts.
- **record**: A per-step execution record containing timing, exit status, and
  produced artifacts.
- **provenance**: The immutable metadata describing tool versions, parameters,
  input hashes, and build context for a run.
