# COMMANDS

## bijux plan
Purpose: plan a pipeline run.
Inputs: pipeline id, profile.
Outputs: graph + manifest skeleton.
Exit codes: 0 success, 1 failure.
Artifacts: none.

## bijux execute
Purpose: execute a pipeline run.
Inputs: pipeline id, profile.
Outputs: run manifest + report bundle.
Exit codes: 0 success, 1 failure.
Artifacts: report.json, report.html, summary.tsv.

## bijux dry-run
Purpose: emit graph + manifest only.
Inputs: pipeline id, profile.
Outputs: graph.json + run_manifest.json.
Exit codes: 0 success, 1 failure.
