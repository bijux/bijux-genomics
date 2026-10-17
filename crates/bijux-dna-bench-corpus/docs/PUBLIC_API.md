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

With `variants`, `vcf::validate_phasing_imputation_truth_manifest_path` accepts
an explicit repository root and declaration path and returns
`PhasingImputationTruthValidationReport`. `PHASING_IMPUTATION_TRUTH_FIXTURE_ID` and
`PHASING_IMPUTATION_TRUTH_MANIFEST_SCHEMA_VERSION` retain the governed identities.
The report keeps schema/fixture/manifest/expected fields, case count, sorted unique
stage/tool inventories and validity. Complete records retain optional phasing,
impute and imputation-metrics projections plus canonical output and masked-site truth.

Phasing declarations support shapeit5, eagle and beagle on `vcf.phasing` without a
truth VCF. Imputation declarations support beagle, glimpse, impute5 and minimac4 on
`vcf.impute` or `vcf.imputation_metrics` with required truth evidence. Existing
support, reference, parser, schema and comparison errors retain their order.

Absolute paths remain supported. Relative source/artifact/output/truth paths resolve
under the supplied root and expected paths under the declaration directory; displayed
paths retain slash normalization. Wire records keep their closed fields and optional
serde behavior. Expected set/map duplicate semantics remain unchanged. This read-only
service does not certify independent provenance, backend execution or new science.

With `variants`, `vcf::validate_population_structure_truth_manifest_path` accepts
an explicit repository root and declaration path and returns
`PopulationStructureTruthValidationReport`. `POPULATION_STRUCTURE_TRUTH_FIXTURE_ID`
and `POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION` retain the governed identities.
The report keeps schema/fixture/manifest/expected fields, case count, sorted unique
stage/tool inventories and validity. Complete records retain arbitrary nested native
metrics plus canonical metadata-aware PCA, admixture and population truth summaries.

plink2 supports `vcf.pca`, `vcf.admixture` and `vcf.population_structure`; eigensoft
supports PCA and population structure. Absolute paths remain supported. Relative
source/artifact/metadata paths resolve under the supplied root and expected paths
under the declaration directory; displayed paths retain slash normalization.
Source references accept existing files and directories. Closed wire fields,
optional serde behavior, error/read ordering and expected-row set/map duplicate
semantics remain unchanged. This read-only service does not certify fresh backend
execution, scientific equivalence or independent provenance.

With `variants`, `vcf::validate_segments_demography_truth_manifest_path` accepts an
explicit repository root and declaration path and returns
`SegmentsDemographyTruthValidationReport`. `SEGMENTS_DEMOGRAPHY_TRUTH_FIXTURE_ID`
and `SEGMENTS_DEMOGRAPHY_TRUTH_MANIFEST_SCHEMA_VERSION` retain the governed identities.
The report keeps schema/fixture/manifest/expected fields, case count, sorted unique
stage/tool inventories and validity. Complete records retain dynamic native metrics
and canonical ROH interval, IBD relatedness and demography inference truth, including
insufficient marker-overlap and insufficient-data observations.

plink2 supports `vcf.roh`; germline, ibdseq and ibdhap support `vcf.ibd`; ibdne supports
`vcf.demography`. Absolute paths remain supported. Relative source/artifact paths
resolve under the supplied root and expected paths under the declaration directory;
displayed paths retain slash normalization. Source references accept existing files
and directories. Closed wire fields, optional serde behavior, acquisition/error
ordering and expected-row set/map duplicate semantics remain unchanged. This
read-only service does not certify fresh native execution or independent provenance.

With `alignment`, `bam::validate_adna_damage_truth_manifest_path` accepts an
explicit repository root and declaration path and returns
`AdnaDamageTruthValidationReport`. `ADNA_DAMAGE_TRUTH_FIXTURE_ID` and
`ADNA_DAMAGE_TRUTH_MANIFEST_SCHEMA_VERSION` retain the governed identities.
The report preserves schema, fixture and path fields, case/insufficient counts,
sorted terminal-pattern classes, declared case order and validity.

The read-only service preserves closed declaration/expected wire fields, UDG
models, terminal fractions within `[0, 1]`, sample-header identity, domain damage
classification, full-record comparison and original error/read order. Expected
row order is ignored and the final duplicate expected record wins; duplicate
declaration IDs reject. It does not establish external provenance or execute
native tools. CLI validation and expected-validation wires remain unchanged.

With `alignment`, `bam::validate_adna_contamination_truth_manifest_path` accepts
an explicit repository root and declaration path and returns
`AdnaContaminationTruthValidationReport`. The matching
`ADNA_CONTAMINATION_TRUTH_FIXTURE_ID` and
`ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION` retain governed identities.
The report preserves schema, fixture/path fields, declared case order, case
and insufficient counts, sorted tool inventory and validity.

The read-only service preserves schmutzi, verifybamid2 and contammix fixture
contexts; closed wire fields, flattened context/compatibility defaults, coverage
and interval checks, complete comparison and original error/read order remain.
Existing directory source references are admitted. Expected set/map duplicate
semantics and declared absolute paths remain supported. Canonical domain
interpretation stays in domain-bam; this API does not run external tools or
establish independent scientific provenance.

## Duplicate and insert fixture truth

With `alignment`, `bam::validate_bam_duplicate_insert_truth_manifest_path(&Path, &Path)` returns `BamDuplicateInsertTruthValidationReport` or the existing declaration/expected/corpus error. The report fields and serde wire are unchanged. `BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID` and `BAM_DUPLICATE_INSERT_TRUTH_MANIFEST_SCHEMA_VERSION` retain their persisted values. Corpus and expected observation order, exact sample ordering, histogram consistency and full-record equality are preserved. This API consumes local fixture observations through the existing domain calculator; it does not certify a fresh external tool run.

## Endogenous-count truth API

With `alignment`, `bam::validate_bam_endogenous_truth_manifest_path(&Path, &Path)` returns `BamEndogenousTruthValidationReport` or the existing acquisition/admission/observation error. The report fields, serde wire, persisted constants and CLI JSON/text remain unchanged. Preserve original corpus-before-expected acquisition, expected sample order, complete record equality, saturating count identity, existing unsigned aggregation and exact 1e-9 fraction tolerance. Method/host-scope declarations and all summary/caveat/provenance fields remain unchanged.

## Coverage and gc-bias truth API

With `alignment`, `bam::validate_bam_gc_coverage_truth_manifest_path(&Path, &Path)` returns `BamGcCoverageTruthValidationReport` or the existing acquisition/admission/observation error. The report fields, serde wire, persisted constants and CLI JSON/text remain unchanged. Preserve corpus-before-expected acquisition; separate expected coverage/GC ordered identities; complete keyed record equality; source/region/reference admission; weighted region depth, breadth, GC-bin/window/read-start totals, dropout-derived score and exact 1e-9 approximate comparison. Existing arithmetic and domain calculation/order remain unchanged.

## Haplogroup truth API

With `alignment`, `bam::validate_bam_haplogroup_truth_manifest_path(&Path, &Path)` returns `BamHaplogroupTruthValidationReport` or the existing acquisition/admission/observation error. The report fields, serde wire, persisted constants and CLI JSON/text remain unchanged. Preserve corpus-before-expected acquisition, declared case order, complete normalized record equality, first declared TSV panel selection, marker count/confidence constraints and ready/partial-support/coverage-gate refusal semantics. Required status coverage remains enforced on both expected and actual bundles; thresholds, read/error order, panel identity and wire fields remain unchanged.

## Sex-inference truth API

With `alignment`, `bam::validate_bam_sex_truth_manifest_path(&Path, &Path)` returns `BamSexTruthValidationReport` or the existing acquisition/admission/observation error. The report fields, serde wire, persisted constants and CLI JSON/text remain unchanged. Preserve corpus-before-expected acquisition, exact expected sample order, complete normalized record equality, all four required call classes, nonnegative coverage and the original insufficiency reason/zero-confidence/status rules. Do not tighten unrelated confidence/method/enum grammar or alter domain thresholds, read/error order, chromosome-system or minimum-Y assumptions.
