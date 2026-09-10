# Corpus admission boundaries

`src/` is organized by input family: `fastq`, `bam`, `vcf`, `amplicon`,
`edna` and `damage`. Each family separates wire models, manifest admission,
repository observations and specialized truth/reference/sample evaluation.
`paths.rs` owns only manifest-relative resolution and report path display.

BAM observations delegate native alignment interpretation to the BAM domain.
VCF observations delegate native variant interpretation to the VCF domain.
Those dependencies are independently selectable features. This crate must not
depend on CLI, API, benchmark aggregation/model, planner, stage execution,
runtime, container or scheduler crates. Corpus facts feed those higher owners.

The services read declared local files and return typed validation reports.
They do not acquire datasets, run tools, publish campaigns or print output.
The caller owns command defaults, presentation and subsequent execution.

Contracts under `tests/contracts/` retain the six families' existing success
and rejection cases in one integration target. Test repository resolution uses
runtime authority so archived executables do not embed an obsolete checkout.

These existing contracts have different strengths. Suffix and sidecar existence
checks are not universally equivalent to native content validation. Extraction
preserves those semantics; additional scientific admission belongs in the
relevant family and requires observable regressions.
