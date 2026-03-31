# Architecture

## Why QA is a separate crate
QA is heavy and effectful (docker runs, large IO, optional network). Keeping it separate prevents production crates from inheriting those dependencies and side effects.

## Modules
- image_qa/
- bin/* wrappers

## Image QA module tree
- `contracts.rs`: QA stage and dataset contracts shared across the crate.
- `datasets/`: corpus discovery, hydration, and subset creation.
- `records/`: QA record construction and prior-pass lookup.
- `validation/`: input inventory lookup and pass enforcement.
- `support/`: path layout helpers, output validation, seqkit metrics, and Docker execution helpers.
- `qa_docker_images/`: Docker image planning, container probing, and smoke-check reporting.

## Data flow
- Image inputs → QA reports.
