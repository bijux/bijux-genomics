# Haplotype ownership

Each workflow composes admitted declarations, prepared stage inputs, native stage
execution, scientific observations and explicit report adapters. Imputation quality
consumes the imputation service; neither depends on CLI or benchmark aggregation.

Native execution files remain in immutable `executions/run-*` directories because
manifests and checksums bind their physical paths and exact command provenance.
Publication atomically copies files to stable command output addresses. It never
rewrites an executed command or a checksum-linked native manifest. The public
benchmark `stage-result.json` is an explicit address projection: its realized
outputs use stable command paths, while the retained execution record still
indexes immutable execution paths. Native evidence files are copied byte for byte.

All replacement and retirement paths are admitted before any prior output is
replaced. The stage/tool/schema ownership inventory identifies generated files;
unknown files survive. Stable obsolete files can retire. Prior immutable executions
remain because downstream reports can still reference them. Deleting retained
executions requires dependency-aware retention; no automatic garbage collection is
claimed. Individual file replacement is atomic; a whole publication is not a
filesystem transaction. Concurrent mutation of one publication root is unsupported.
