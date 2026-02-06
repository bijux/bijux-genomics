# Architecture Litmus

These rules are executable. If they drift, fix the code or update the contract with intent.

- engine does not depend on runner or environment
- prelude is exports-only
- defaults live only in bijux-pipelines
- composition roots are only in API/CLI
