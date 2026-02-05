# Architecture Litmus

This document is the single checklist for architectural truth.

## Non-negotiables
- engine does not depend on runner or environment
- runner depends on engine and implements engine::Runner
- prelude is exports-only (no functions or impl blocks)
- defaults live only in bijux-pipelines
- composition roots are only in API/CLI
- no AppleDouble or .DS_Store files in repo
