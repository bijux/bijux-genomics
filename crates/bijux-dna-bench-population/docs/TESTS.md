# Population verification

Owner tests retain cohort/parser assertions and explicit dependency admission.
Public CLI consumers cover composition and report compatibility. Tests with native
execution require configured runner bindings. Test fixture readiness observations
are inputs to service tests, not claims about current production readiness.

Private owner assertions live under `src/`: capability-local `tests.rs` modules and the crate-level `tests/` group. Existing `#[cfg(test)]` declarations compile each case exactly once in the owner library. Shared fixture support remains under `tests/support/`; no extra integration binary or production test API is required. Command integration tests stay with the CLI.
