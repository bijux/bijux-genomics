# Container Smoke Contract

Authoritative smoke contract for container QA.

This file is the CI-checked contract location. It defines:
- required checks per tool: `--version`, `--help`, minimal run, and expected-failure run
- expected exit-code behavior for each check
- no-runtime-network rule unless explicitly declared in `containers/network/<tool>.network.toml`
- cross-runtime equivalence requirements for Docker vs Apptainer

Detailed runtime procedures and examples are documented in `containers/docs/SMOKE_CONTRACT.md`.

