# Architecture

## Why QA is a separate crate
QA is heavy and effectful (docker runs, large IO, optional network). Keeping it separate prevents production crates from inheriting those dependencies and side effects.

## Modules
- `public_api.rs`
- `image_qa/`
- `bin/*` wrappers

## Image QA module tree
- `contracts.rs`: QA stage and dataset contracts shared across the crate.
- `facade.rs`: stable image QA entrypoints.
- `behavioral/`: preprocessing stage QA and postprocessing/reporting QA.
- `datasets/`: corpus discovery, hydration, and subset creation.
- `records/`: QA record construction, persistence, and prior-pass lookup.
- `validation/`: input inventory lookup and pass enforcement.
- `support/`: diagnostics, output validation, image resolution, seqkit metrics, and Docker execution helpers.
- `qa_docker_images/`: Docker image planning, option parsing, runtime adapters, container probing, and smoke-check reporting.

## Data flow
- Image inputs → QA reports.
