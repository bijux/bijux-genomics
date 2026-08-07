# bijux-dna-containers-qa Public API

## Public Modules

- `image_qa`
- `public_api`
- `qa_docker_images`
- `runtime_verification`

## Major Export Groups

- `public_api::api`: facade over declarative container contracts plus QA-owned runtime execution.
- `image_qa`: image QA runner, artifact paths, record helpers, validation helpers, and support
  functions used by QA workflows.
- `qa_docker_images`: Docker image catalog probe command implementation.
- `runtime_verification`: bounded catalog-wide Apptainer probe execution and immutable evidence
  publication.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and the public API docs test to change together.
- New command surfaces require `COMMANDS.md` and command boundary tests.
- Public QA record or artifact changes require `CONTRACTS.md` and artifact contract tests.
