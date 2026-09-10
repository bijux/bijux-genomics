# bijux-dna-containers-qa Architecture

QA is a separate crate because image readiness checks are intentionally heavy and effectful. Keeping
this code isolated prevents production crates from inheriting Docker, Apptainer, SQLite, and QA
dataset concerns.

## Source Layout

- `src/bin/`: thin command wrappers for Docker QA, image QA, and Apptainer evidence collection.
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
- `src/image_qa/runtime_verification/`: full-catalog Apptainer evidence publication with bounded
  execution for changed images and digest-verified reuse of unchanged baseline evidence.
- `src/security_verification/`: exact-image SBOM, vulnerability, secret, signature, structural,
  and runtime probe collection for changed images plus digest-verified unchanged-image baseline
  reuse using container-owned evidence types, without policy or release verdicts.
- `src/qualification/`: binding-derived immutable specification compilation, incremental catalog
  coverage, pipeline-derived probe plans, declared-output validation, metric evaluation, and
  classified qualification verdicts consumed by execution backends.

## Data Flow

1. Platform and tool-image catalog facts come from `bijux-dna-containers`.
2. QA datasets are discovered or hydrated under declared QA inputs.
3. Catalog, run, image, stage-binding, and governed-input identities compile into immutable
   qualification pipelines; those pipelines become typed QA probe and acceptance plans.
4. Docker/Apptainer checks run only through explicit commands owned by execution backends.
5. Security probes remain raw QA observations for policy-owned admission.
6. QA evaluates stage probe observations, declared outputs, and metrics into classified verdicts.
7. Results become typed records persisted under governed evidence roots.
8. Validation helpers read those records when higher layers require prior QA evidence.

## Tree Rules

- Keep binaries thin; move reusable behavior into `image_qa/`.
- Keep Docker command construction in `support/docker_exec/`, `support/docker_runtime.rs`, or
  `qa_docker_images/`.
- Keep public exports documented in `PUBLIC_API.md`.
- Update `COMMANDS.md` before adding any host command execution.

Qualification specification models and generated schemas live in `qualification/specification/model.rs`. Pure identity, selection and lexical path admission live in `admission.rs`; bounded source/selection decoding and schema publication live in `storage.rs`. Compilation and materialization consume this shared contract directly. Filesystem identity, repository HEAD and frozen repository checks remain explicit workflow effects. This module separation does not introduce an independently compiled Cargo package.

Qualification compilation coordinates `binding` input/output projection, `commands` shell and discovery construction, `pipeline` persisted projection, `storage` file/image/frozen-repository identity, and `coverage` per-tool observation. Binding projection performs no filesystem reads; compilation still verifies every declared input digest. Missing or altered input bytes cannot qualify through successful projection.
