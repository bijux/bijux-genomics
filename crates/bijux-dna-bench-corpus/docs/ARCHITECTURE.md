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

`panel_metadata` separates wire observations, reference authority admission, marker
planning, destination admission, materialization and exclusion. The optional
`panel-metadata` feature admits existing db-ref and infrastructure dependencies
without changing the default compilation closure. Markers preserve the existing
metadata-only labels and derivation observations; they are not executable panel
assets. All destinations are checked before writes. Existing exact markers are
reused, conflicting bytes are rejected, and atomic create never replaces a file.
Publication is not a multi-file transaction; callers must control the output
directory against concurrent ancestry changes.

## BAM assignment

`bam/assignment` pairs its original wire records with corpus routing.
Fixture acquisition reuses the BAM and damage owners directly. Governed configuration
admission precedes evidence-profile formatting and seven independent stage projections.
Sample/reference/path identity and genotyping/kinship contract checks stay beside
their consumed corpus schemas. Non-finite thresholds reject before publication.

Inventory, command coverage and compatibility are independent injected authorities.
Canonical domain BAM assignment rules remain in their domain owner; core supplies
stage identities. Core and infrastructure stay optional through `alignment`,
leaving default corpus compilation free of specialized workspace dependencies.
TSV publication uses infrastructure atomic replacement, with exact declared columns.

## BAM alignment fixture truth

`bam/truth` owns declared alignment-truth manifests and expected bundles. `model.rs`
retains the wire records, `admission.rs` checks identity, ordering and independent
aggregate invariants, and `observations.rs` reads declared files and consumes the
BAM domain's canonical native alignment summary. The facade compares complete
expected and observed records and returns the original validation report. It does
not launch tools, print output, write fixtures or orchestrate benchmarks. The CLI
fixture entrypoints delegate to this owner through their existing import path.

## VCF filter fixture truth

`vcf/filter_truth` admits declared filter and damage-filter expectations against
canonical parser and native VCF observations. Its private model, admission and
observation modules separate wire records, local reference/support checks and
evidence reads. The facade compares complete case records without publication
or execution. Domain parsing/interpretation and CLI presentation retain their
owners. [VCF filter truth](VCF_FILTER_TRUTH.md) records schemas, error ordering,
path normalization and the existing expected-case set/map semantics.

## VCF genotype fixture truth

`vcf/genotype_truth` owns the declared genotype manifest, expected case bundle
and read-only validation report. Private wire records, admission and observation
modules retain schema checks, reference admission and native evidence acquisition.
Observations reuse the VCF domain's canonical genotype summary and calling-boundary
rules; command JSON is metadata and its argv is never executed. The facade compares
complete expected and observed records. CLI entrypoints own presentation.

Manifest reference and evaluation admission precede expected-bundle acquisition;
bundle identity admission precedes VCF and command observations. Expected case
identity remains set-based and comparison remains map-based: ordering is ignored,
duplicate expected rows collapse to the last record, and duplicate manifest case
IDs reject. This service preserves those limitations rather than certifying
independent scientific provenance. It neither writes fixtures nor launches tools.

## VCF phasing/imputation fixture truth

`vcf/phasing_imputation_truth` separates the declared wire records, source and
supported-case admission, and native observations. The facade compares complete
expected and observed records; canonical VCF parsers and scientific output-truth
summaries remain in domain-vcf. CLI fixture routes retain presentation and whole-root
validation composition. This corpus service reads local declarations and evidence
without generating fixtures, launching tools or publishing products.

Manifest references and tool/stage/truth combinations are admitted before expected
bundle acquisition. Bundle identity admission precedes native metrics parsing, which
precedes output/truth VCF interpretation for each case. Expected identities remain
set-based and comparison remains map-based: order is ignored and the last duplicate
expected row wins; duplicate manifest IDs reject. Complete typed comparison and the
existing metrics-family preference preserve the original scientific record fields.

## VCF population fixture truth

`vcf/population_structure_truth` owns the declared manifest, expected bundle and
read-only population fixture validation report. Private wire, admission and
observation modules keep local reference/support checks beside their records.
Observations delegate plink2/eigensoft metrics parsing and metadata-aware PCA,
admixture and population summaries to domain-vcf. The facade compares complete
case records; CLI fixture commands retain presentation and whole-root composition.

Declared source references intentionally accept existing files and directories.
Per-case artifact and metadata admission precede support checks and expected-bundle
acquisition. Native metrics acquisition precedes scientific metadata joins.
Expected identities remain set-based and comparison remains map-based: ordering
is ignored, the last duplicate expected record wins, and duplicate manifest IDs
reject. Dynamic nested metrics and the existing metrics-family preference remain
unchanged. The service does not write fixtures, launch tools or establish new science.

## VCF segment and demography fixture truth

`vcf/segments_demography_truth` owns declared fixture wire records, reference and
support admission, and read-only native observations. The facade compares complete
expected and observed ROH, IBD and demography case records. Segment parsers and
scientific interval, relatedness and inference rules remain in domain-vcf. CLI
fixture commands retain defaults, presentation and whole-root composition.

Source references intentionally accept existing files and directories. Case artifact
admission precedes tool/stage support and expected-bundle acquisition; expected case
identity admission precedes native parsing and canonical output-truth summaries.
Expected identities remain set-based and comparison remains map-based: ordering is
ignored and the final duplicate expected record wins; duplicate manifest IDs reject
after per-case admission. Arbitrary nested metrics, optional output families and
complete/insufficient observations remain unchanged. This service reads local
fixtures without launching tools, publishing products or establishing new science.

## aDNA damage fixture truth

`bam/damage_truth` owns the declared damage manifest, expected bundle admission
and complete expected/observed comparison. Its private models, declaration
admission and file observations follow the existing alignment truth boundary.
The BAM domain retains damage classification and UDG/insufficiency interpretation.
CLI entrypoints keep command defaults, JSON/text presentation and root-report
composition through their existing import path. This service reads declared local
files without running tools, generating fixtures or publishing new truth.

Manifest schema, identity, governed source membership and ordered case admission
precede expected-bundle acquisition. Bundle admission precedes alignment header
inspection and domain damage summaries. Manifest case IDs reject duplicates;
expected case identity remains set-based and the final duplicate expected row
wins during map comparison. Relative source and input paths use the supplied
repository root, while expected paths use the declaration directory. Absolute
paths and original display conventions remain supported.

## aDNA contamination fixture truth

`bam/contamination_truth` owns the declared contamination cases, expected
bundle admission and complete expected/observed comparison. Private models,
admission and observations separate persisted fixture wire from local reads.
The BAM domain retains tool-specific context, coverage and confidence-interval
interpretation. CLI validation and expected-validation entrypoints retain
command defaults, JSON/text presentation and root-report composition.

Manifest identity and governed source membership precede ordered case admission.
Source references retain the existing `exists` rule, accepting files or
directories. Tool, coverage, estimate/interval and input-path admission precede
expected-bundle acquisition; expected identity admission precedes alignment
header inspection and domain summary projection. Declared metric/context values
remain fixture inputs, not proof of newly executed contamination tools.

Manifest case IDs reject duplicates. Expected case identity remains set-based
and the final duplicate expected row wins during map comparison. Relative
source/input paths use the supplied root; expected paths use the declaration
directory. Absolute paths, original display strings and read/error order remain
unchanged. The service does not generate fixtures or publish products.

## Duplicate-family and insert-size fixture truth

`bam/duplicate_insert_truth` owns the complete declared fixture comparison: private typed wire records, corpus admission and histogram invariants, and local observations. The existing corpus loader and admission validate all declared BAM inputs before expected truth acquisition. Existing domain summaries retain duplicate geometry and insert statistics. The CLI keeps route/default/render/report duties through its stable import facade. No runtime tool execution or payload publication is added.

Expected sample order, complete records, integer arithmetic (including existing saturation choices), pair-count/optional-statistic rules, and read/error priority remain compatibility contracts. The four original CLI cases remain registered; the corpus oracle uses private declaration/expected copies and immutable governed payload inputs.

## Endogenous-count fixture ownership

`bam/endogenous_truth` owns private declaration/expected wire, count/fraction admission, complete comparison and local corpus observations. The existing domain calculator retains mapped/background/retained/endogenous and host-scope provenance. The existing Core count conversion is consumed directly; no CLI dependency or new numeric helper is added.

## Coverage and gc-bias fixture ownership

`bam/coverage_gc_truth` separates declaration admission, typed expected-summary arithmetic, complete identity-based comparison, local observations and wire models. Coverage and GC remain one existing bundle contract with two preserved observations. Existing domain coverage/GC calculators and Core numeric conversion are reused. The private identity trait and both implementations move with the comparator, with no public helper API.

## Haplogroup fixture ownership

`bam/haplogroup_truth` owns declaration admission, expected identity/marker/status rules, required outcome coverage, local observations and private normalized wire. Existing domain marker support and coverage gate calculations remain canonical. The CLI keeps routing, defaults, JSON/text and root-report projection.

## Sex-inference fixture ownership

`bam/sex_truth` owns typed fixture wire, declaration and expected admission, call-class coverage, local observations and complete comparison. The existing domain calculator retains reference context, X/Y/autosomal coverage, minimum-site gates and confidence/call thresholds. Private normalized summary records retain the public JSON wire without runtime-local payload paths. The CLI keeps routing/default/render/root-report responsibilities.
