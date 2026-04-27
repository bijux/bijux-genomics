# Container Science Evidence Boundary

Purpose: define what container evidence can and cannot prove for scientific review.

## What Containers Prove
- The definition file builds the intended runtime image.
- The packaged command exposes expected `--version`, `--help`, minimal, and negative-path behavior.
- The image identity is traceable through version metadata, lock metadata, registry digest, and SIF hash where applicable.
- Runtime network and write behavior follow the declared container policy.

## What Containers Do Not Prove
- A tool's algorithm is scientifically valid.
- A tool-stage pair is appropriate for a dataset, library chemistry, reference database, or biological question.
- A software-only citation is equivalent to a peer-reviewed method paper.
- A smoke-passing SIF or Docker image closes missing local paper/archive evidence.

## Required Cross-Links
- Tool citation and stage binding:
  [domain/fastq/execution_support.yaml](../../domain/fastq/execution_support.yaml)
  and [docs/20-science/fastq/REFERENCES.md](../../docs/20-science/fastq/REFERENCES.md).
- FASTQ closure state:
  [domain/fastq/docs/EVIDENCE_CLOSURE.md](../../domain/fastq/docs/EVIDENCE_CLOSURE.md).
- Generated evidence status:
  [science/generated/current/evidence/README.md](../../science/generated/current/evidence/README.md).
- Container smoke behavior: [containers/docs/SMOKE_CONTRACT.md](SMOKE_CONTRACT.md).
- Container promotion requirements: [containers/docs/PROMOTION_POLICY.md](PROMOTION_POLICY.md).

## Review Rule
When a container change touches a scientific tool, reviewers should verify two independent paths:

1. Container path: version pin, lock, recipe provenance, and smoke behavior.
2. Science path: stage admission, citation locator, evidence map, paper map, and required local archives.

Only the combination supports production confidence. Container success alone must be reported as runtime evidence, not scientific closure.
