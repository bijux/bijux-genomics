# VCF filter fixture truth

With `variants`, `vcf::validate_vcf_filter_truth_manifest_path` accepts an explicit
repository root and declared truth manifest path. It returns
`VcfFilterTruthValidationReport`, preserving the report's schema, fixture and
source paths, case count, sorted stage/tool inventories and validity field.
`VCF_FILTER_TRUTH_FIXTURE_ID` and `VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION`
identify the governed declaration.

`vcf/filter_truth/model.rs` owns the persisted manifest, expectation and report
records. `admission.rs` admits declared sources, case identities and supported
combinations. `observations.rs` reads local evidence and delegates normalization
and native VCF interpretation to `bijux-dna-domain-vcf`. The facade compares
complete expected and observed case records. It does not run tools, print output,
write evidence or orchestrate benchmark execution.

The manifest, expected bundle and validation schemas are
`bijux.bench.vcf_filter_truth.v1`,
`bijux.bench.vcf_filter_truth.expected.v1` and
`bijux.bench.vcf_filter_truth.validation.v1`. Private typed records retain their
unknown-field rejection, optional fields and exact numeric comparisons. Filter
labels support `bcftools` with `vcf.filter`; damage removal supports `bcftools` or
`angsd` with `vcf.damage_filter`. The governed declaration covers two `bcftools`
cases; accepting a combination does not establish tool interchangeability.

Declared source and case admission precedes expected-bundle acquisition. For each
case, optional native output truth is loaded before metrics. Absolute paths remain
supported. Report paths retain slash normalization, including backslash replacement.
The original CLI named fixture and root aggregation entrypoints delegate through
their existing imports.

Manifest case identities must be unique. Expected bundle identities retain their
existing set/map comparison: row order is immaterial and duplicate rows can collapse.
This service does not establish stricter uniqueness, path containment, provenance,
input-size bounds or complete native output coverage when output paths are omitted.

Owner contracts under `tests/contracts/vcf/filter_truth.rs` exercise input
preservation, reference and support failure priority, case admission, set/map
compatibility, threshold/site drift and optional output evidence. The original
CLI text and JSON cases remain separate presentation contracts. Native verification
of the owner and those consumers is required for changes to this boundary.
