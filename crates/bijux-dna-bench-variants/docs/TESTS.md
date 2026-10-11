# Variant verification

The owner suite retains the original filtering, damage, postprocessing and QC
fixture/assertion cases. Additional cases exercise invalid declarations, unsafe
tool paths, unsupported QC formats before effects, preservation of unknown files,
repeated publication, truthful QC provenance and immutable native checksum paths.

Public CLI tests exercise command parsing, request composition, readiness consumers
and stable returned artifact paths. Default and all-feature configurations require
separate native inventories. Local tool-backed results support only the tools
actually executed; projected QC reports do not prove external-tool substitution.

Private owner assertions live under `src/`: capability-local `tests.rs` modules and the crate-level `tests/` group. Existing `#[cfg(test)]` declarations compile each case exactly once in the owner library. Shared fixture support remains under `tests/support/`; no extra integration binary or production test API is required. Command integration tests stay with the CLI.
