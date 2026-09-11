# Public API

`probes` consumes typed `StagePlanV1` inputs and returns `MaterializedFastqProof`. Correction also requires explicit read paths, quality encoding and conservative-mode settings. Five producers own reference indexing, correction, host depletion, rRNA depletion and taxonomy fixtures. `probes::reports` selects fifteen canonical FASTQ domain report parsers and returns the JSON value from the same input observation after typed admission.

Stage identity and required input observations are admitted before fixture creation. Read framing checks cover four-line completeness, header/separator markers and matching sequence/quality lengths; they do not establish biological validity or paired-read identity. Publication is atomic per file and accepts existing gzip input without recompression. Synthetic unchanged and empty outputs establish parser compatibility only.
