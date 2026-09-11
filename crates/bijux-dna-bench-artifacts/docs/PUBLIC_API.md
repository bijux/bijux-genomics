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
