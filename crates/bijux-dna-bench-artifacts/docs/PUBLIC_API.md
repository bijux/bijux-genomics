# Publication API

`PublicationSchema` selects the capability's persisted ownership protocol.
`ReportOwner` identifies the final report schema, stage and tool.
`execution_parent` admits an execution-store path; callers prepare a unique
execution beneath it. `PublishedArtifactPaths` maps owned execution paths to their
stable public addresses and refuses foreign or escaping paths.

`PublicationPlan::admit` validates paths, prior ownership, replacements and the
optional typed benchmark stage-result projection before mutation of prior files.
The caller must retain the immutable execution directory before `publish` so
native manifest references survive a partial publication failure.

## Schema artifacts

`schemas::publication` publishes canonical benchmark execution and normalized-result
schemas from the model crate. Existing benchmark API exports forward to this owner.
`schemas::validation` accepts explicit paths and typed domain selections, compares
the shared and complete stage schema files, and publishes diagnostic reports even
when a readable schema disagrees. Unreadable or malformed inputs return an error.

`schemas::vcf_report` publishes and reports the VCF stage schema portfolio. Schema
file writes are atomic individually; the portfolio is not a multi-file transaction.
Validation of schema files is separate from scientific validation of result payloads.
