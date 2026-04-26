# bijux-dna-environment-qa Architecture

QA is a separate crate because image readiness checks are intentionally heavy and effectful. Keeping
this code isolated prevents production crates from inheriting Docker, Apptainer, SQLite, and QA
dataset concerns.

## Source Layout

- `src/bin/`: command wrappers for `image_qa`, `qa_docker_images`, and `build_docker_images`.
- `src/lib.rs`: public module exports.
- `src/public_api.rs`: stable facade that re-exports environment APIs for QA consumers.
- `src/image_qa/contracts.rs`: QA stage roster and dataset contract models.
- `src/image_qa/facade.rs`: entrypoint for full image QA runs.
- `src/image_qa/datasets/`: FASTQ input discovery and subset hydration.
- `src/image_qa/records/`: QA record construction, prior-pass lookup, JSONL/SQLite persistence, and
  summary writing.
- `src/image_qa/validation/`: checks that required image QA inputs have passing records.
- `src/image_qa/behavioral/`: FASTQ preprocessing and postprocessing behavioral scenarios.
- `src/image_qa/support/`: Docker execution helpers, output contracts, diagnostics, image
  resolution, layout helpers, and seqkit metrics.
- `src/image_qa/qa_docker_images/`: Docker catalog probe planning, runtime abstraction, probe
  execution, and reporting.

## Data Flow

1. Platform and tool-image catalog facts come from `bijux-dna-environment`.
2. QA datasets are discovered or hydrated under declared QA inputs.
3. Docker/Apptainer checks run only through explicit commands.
4. Results become `ImageQaRecord` values and are persisted under `artifacts/image-qa/<platform>/`.
5. Validation helpers read those records when higher layers require prior QA evidence.

## Tree Rules

- Keep binaries thin; move reusable behavior into `image_qa/`.
- Keep Docker command construction in `support/docker_exec/`, `support/docker_runtime.rs`, or
  `qa_docker_images/`.
- Keep public exports documented in `PUBLIC_API.md`.
- Update `COMMANDS.md` before adding any host command execution.
