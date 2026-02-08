# DATASETS

## Provenance
QA uses small, local fixtures with recorded provenance.

## Checksums
Each dataset includes a checksum in the dataset registry. Checksums must be updated when fixtures change.

## Included
- Minimal FASTQ samples for tool smoke tests.
- Synthetic inputs for deterministic validation.

## In-repo datasets
- `tests/fixtures/qa_artifacts/*` (manifest/report fixtures)
- Minimal FASTQ samples (small synthetic inputs)

## External datasets (not in repo)
- Larger real-world samples (not checked in).
- Any PII or protected data (never fetched).

## Fetching external data
External datasets are fetched manually and must never be pulled in CI.
Document the source, checksum, and license in the dataset registry.

## Licensing
Fixtures are either synthetic or derived from public domain sources.

## Purpose
Each dataset exists to validate specific image behaviors.
