# RUNBOOK

## Running QA locally
- `cargo run -p bijux-environment-qa --bin qa_docker_images -- --platform <name>`
- `cargo run -p bijux-environment-qa --bin image_qa -- --platform <name>`

## Expected outputs
- QA produces per-tool logs and a summary report under the configured output directory.
- Failures are reported with the tool name, image, and failing step.

## Common failures
- Missing docker image or digest mismatch.
- Tool executable not present inside the container.
- Probe command exits non-zero.

## Notes
- Network and docker access are required for live image checks.
- Offline fixtures are used for deterministic QA unless explicitly overridden.
