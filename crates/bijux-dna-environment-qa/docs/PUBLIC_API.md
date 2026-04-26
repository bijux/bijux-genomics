# bijux-dna-environment-qa Public API

## Public Modules

- `image_qa`
- `public_api`
- `qa_docker_images`

## Major Export Groups

- `public_api::api`: facade re-export of `bijux-dna-environment::api` for QA consumers.
- `image_qa`: image QA runner, artifact paths, record helpers, validation helpers, and support
  functions used by QA workflows.
- `qa_docker_images`: Docker image catalog probe command implementation.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and the public API docs test to change together.
- New command surfaces require `COMMANDS.md` and command boundary tests.
- Public QA record or artifact changes require `CONTRACTS.md` and artifact contract tests.
