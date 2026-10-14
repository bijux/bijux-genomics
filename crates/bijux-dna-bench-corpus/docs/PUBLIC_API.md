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
