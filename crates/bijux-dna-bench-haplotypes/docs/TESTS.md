# Haplotype verification

Owner tests preserve original genotype fixtures, missing-quality observations,
panel identities and supported tool substitution. Test composition admits the
repository's governed matrix explicitly without importing CLI or benchmark services.

Boundary regressions reject malformed templates, unsafe tool paths, unbound
substitutions, foreign inventories and symlinked output paths. Publication cases
exercise checksum-linked native paths across repeated runs and preserve downstream
references plus unrelated user evidence. CLI consumers retain real executable
composition coverage. Native tool availability is recorded separately from marker
fixtures and captured evidence; markers do not certify native execution.

Private owner assertions live under `src/`: capability-local `tests.rs` modules and the crate-level `tests/` group. Existing `#[cfg(test)]` declarations compile each case exactly once in the owner library. Shared fixture support remains under `tests/support/`; no extra integration binary or production test API is required. Command integration tests stay with the CLI.
