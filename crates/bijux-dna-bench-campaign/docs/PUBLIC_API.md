# Corpus campaign public API

## Module Inventory

- artifacts: artifact bundle observations for campaign/database composition.
- catalog: governed stage command, report and resume mappings.
- ports: explicit `CampaignExecution`, `CampaignPaths` and `CampaignClock` dependencies, with argument boundaries and execution context carried by `CampaignCommand`.
- request: resolved campaign input and execution declarations.

`run_corpus_campaign` is the orchestration entry point. It returns the canonical corpus run manifest and preserves existing stage, artifact and sample identities. Callers supply canonical workspace configuration, command execution and publication time. Internal planning, admission, cache and QC modules are not public extension surfaces.

Nonzero tool exits remain report data; transport/setup failures return errors. Dry runs do not reset or resume existing reports. Multi-file generation is not transactional, so callers must own concurrency over admitted destinations.
