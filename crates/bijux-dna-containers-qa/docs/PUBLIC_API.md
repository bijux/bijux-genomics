# bijux-dna-containers-qa Public API

## Public Modules

- `image_qa`
- `public_api`
- `qa_docker_images`
- `qualification`
- `runtime_verification`
- `security_verification`

## Major Export Groups

- `public_api::api`: facade over declarative container contracts plus QA-owned runtime execution.
- `image_qa`: image QA runner, artifact paths, record helpers, validation helpers, and support
  functions used by QA workflows.
- `qa_docker_images`: Docker image catalog probe command implementation.
- `qualification`: typed pipeline-derived probe plans plus classified output and metric verdicts.
- `runtime_verification`: bounded changed-image Apptainer probe execution, digest-verified baseline
  reuse, and immutable full-catalog evidence publication.
- `security_verification`: bounded exact-image security probe execution using container-owned raw
  evidence publication types; policy and release verdicts remain outside this crate.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and the public API docs test to change together.
- New command surfaces require `COMMANDS.md` and command boundary tests.
- Public QA record or artifact changes require `CONTRACTS.md` and artifact contract tests.
