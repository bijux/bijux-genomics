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
- `qualification`: generic checksum-bound specification compilation and coverage, typed
  pipeline-derived probe plans, and classified output and metric verdicts.
- `runtime_verification`: bounded changed-image Apptainer probe execution, digest-verified baseline
  reuse, and immutable full-catalog evidence publication.
- `security_verification`: bounded changed-image security probe execution and admitted
  unchanged-image evidence reuse using container-owned raw publication types; policy and release
  verdicts remain outside this crate.

## Stability Rules

- `src/lib.rs` is the public module source of truth.
- New public modules require this file and the public API docs test to change together.
- New command surfaces require `COMMANDS.md` and command boundary tests.
- Public QA record or artifact changes require `CONTRACTS.md` and artifact contract tests.

## Features

`verification-tools` is enabled by default and retains every existing verification module and binary. Disabling default features retains `qualification` and the corresponding `api` exports, together with container contract reexports. Image, runtime and security verification entrypoints require `verification-tools`; Cargo skips their binaries when the feature is absent.
