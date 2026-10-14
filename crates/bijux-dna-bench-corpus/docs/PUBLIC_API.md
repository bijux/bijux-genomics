# Public corpus services

Public modules are `fastq`, `amplicon`, `edna`, `damage`, and feature-gated `bam`
and `vcf`. Each exposes its corpus manifest/report types, schema identity,
governed default fixture path and named loading/validation functions.

The main entrypoints are `validate_fastq_corpus_fixture_manifest_path`,
`validate_amplicon_corpus_fixture_manifest_path`,
`validate_edna_corpus_fixture_manifest_path`,
`validate_bam_damage_fixture_manifest_path`,
`validate_bam_corpus_fixture_manifest_path` and
`validate_vcf_corpus_fixture_manifest_path`. Their arguments are explicit paths;
errors preserve the source file and violated corpus rule.

Specialized amplicon truth-table and eDNA taxon loaders support downstream
micro-pipeline consumers. VCF sample metadata supports cohort compatibility.
Implementation modules remain private and have no public deep-import contract.

Persisted schema identities, field names, default path values, and observed
validation behavior remain unchanged. Absolute manifest paths remain supported;
these APIs do not claim path containment, bounded total input size, external
provenance verification or complete tool substitution.

With `panel-metadata`, `panel_metadata` exposes governed panel/map constants,
`ResolvedVcfPanelAssetContract`, `ResolvedVcfPanelDerivationContract`,
`LocalVcfPanelFixtureReport`, authority resolution, metadata materialization and
fixture exclusion. Existing `bijux-dna-bench-vcf::fixtures::panel` imports remain
compatible. Only byte-identical existing markers are reusable.

The `panel-metadata` surface also exposes `governed_vcf_panel_species_context`,
the exact local panel fixture species, build and contig-alias context. Haplotype
and postprocessing benchmark consumers share this authority.

## BAM corpus assignment

`bam::assignment` exports `BamCorpusAssignmentRow`,
`BamCorpusAssignmentReport`, `AssignmentContext`,
`render_bam_corpus_assignment` and `collect_bam_corpus_assignment_rows`.
The alignment feature enables this surface. InventoryEvidence, CoverageEvidence and
CompatibilityEvidence provide consumed nonserialized observations; the corpus owner
loads its own canonical fixture reports and governed configuration shapes.

The v3 report schema, TSV columns, output defaults, row ordering, stage fixture routing,
sample/reference comparisons and kinship relationships remain unchanged.
Duplicate compatibility stages and non-finite threshold comparisons reject explicitly.
Source failure preserves the existing report; successful publication replaces it
atomically. Assignment evidence does not certify fresh native tool execution.

With `alignment`, `bam::validate_bam_alignment_truth_manifest_path` accepts an
explicit repository root and truth manifest path and returns
`BamAlignmentTruthValidationReport`. `BAM_ALIGNMENT_TRUTH_FIXTURE_ID` and
`BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION` retain the governed identities.
Manifest, expected bundle and corpus admission precede complete observed record
comparison. Report fields, rejection order, path resolution and schema identities
are preserved. Declared absolute paths remain supported; this read-only service
does not establish external provenance or publish new truth.

With `variants`, `vcf::validate_vcf_filter_truth_manifest_path` accepts an explicit
repository root and declaration path and returns `VcfFilterTruthValidationReport`.
`VCF_FILTER_TRUTH_FIXTURE_ID` and `VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION` retain
the governed identities. This read-only fixture service preserves its complete
comparison, schema fields, support/error ordering, optional output evidence and
slash-normalized report paths. [VCF filter truth](VCF_FILTER_TRUTH.md) documents
the supported combinations and retained expected-row set/map limitations.

With `variants`, `vcf::validate_vcf_genotype_truth_manifest_path` accepts an explicit
repository root and declaration path and returns `VcfGenotypeTruthValidationReport`.
`VCF_GENOTYPE_TRUTH_FIXTURE_ID` and `VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION`
retain the governed identities. The report keeps its schema, fixture/path fields,
case count, sorted unique stage/tool inventories and validity flag. Full comparison
includes genotype/missingness/likelihood summaries, optional command seed metadata
and calling-boundary records. Native interpretation stays in the VCF domain.

Absolute declaration, source, expected and input paths remain supported. Expected
paths resolve against the declaration directory; source/input/command paths resolve
against the supplied repository root. Display paths retain slash normalization.
Command metadata remains permissive about unknown fields; manifest and expected
wire records retain their existing closed fields. The service is read-only and
retains expected-row set/map duplicate semantics and original rejection ordering.
