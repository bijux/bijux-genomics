# ENDPOINT_GUIDES

## plan
Contract: returns graph + hash.
Failure modes: invalid profile, contract validation.

## execute
Contract: returns run id + manifest + report pointer.
Failure modes: tool error, contract error.

## report
Contract: returns report bundle paths.
Failure modes: missing artifacts.

## run-index
Contract: list available runs.
Failure modes: missing storage.

## explain
Contract: selection reasons + defaults diff.
Failure modes: unknown pipeline.
