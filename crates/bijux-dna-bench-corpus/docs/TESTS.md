# Corpus validation tests

Run from the workspace root with build products under `artifacts/`:

```sh
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked --no-default-features --features alignment
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked --no-default-features --features variants
CARGO_TARGET_DIR=artifacts/target cargo test -p bijux-dna-bench-corpus --locked --all-features
```

The contracts cover manifest rejection, paired reads, taxonomy and abundance
observations, damage fixtures, BAM sample/header and cohort checks, and VCF
reference/sample/variant admission. BAM and VCF cases compile with their
respective features. Fixtures resolve the repository at runtime and allocate
scratch products through the test harness.

Dependency regressions reject CLI, benchmark aggregation, execution and runtime
edges, mandatory native dependencies, and implicitly enabled native defaults.
These tests establish declared local admission behavior. They do not prove
interchangeability of every external scientific tool or performance on HPC.

## BAM assignment contracts

Owner regressions reuse the existing governed fixture/configuration corpus to test
valid admission, foreign sample rejection, non-finite posterior rejection, reference
build substitution and pairwise relationship loss. Small local files test canonical
path identity, including Unix aliases. Injected source failures verify acquisition
ordering and preserve previous output. Duplicate compatibility stages cannot overwrite.

Three original CLI contracts and the original inline routing assertions retain text,
JSON and TSV coverage. Default, alignment and all-feature owner inventories remain
separate; assignment regressions require alignment, and alias coverage requires Unix.

## aDNA damage truth contracts

The alignment-feature contracts reuse an owned copy of the governed damage
manifest, expected bundle and declared source files. The damage truth case
checks the real two-case terminal/insufficiency projection, proves expected
identity rejection precedes alignment-header observation, then proves the
foreign header rejection and exact restored report. No shared fixture is
modified. The four existing CLI observed/expected JSON/text cases keep their
identities and presentation assertions. These are fixture admission controls,
not evidence that an external damage tool ran.

## aDNA contamination truth contracts

The alignment-feature oracle reads an owned copy of the governed six-case
contamination declaration and expected bundle. It checks all three tools and
three insufficient cases, preserves existing directory source membership,
proves unsupported tool rejection precedes a missing expected bundle, then
checks the confidence-interval ordering refusal and restored report/fixture
bytes. No governed source fixture is mutated. The four existing CLI observed
and expected JSON/text identities remain unchanged. These fixture contracts
do not establish that external contamination tools executed.

## Duplicate and insert fixture admission

The existing `contracts` target, with `alignment`, registers `bam::duplicate_insert_truth::duplicate_insert_truth_preserves_histogram_and_sample_order_admission`. It preserves the governed two-sample/three-bin report, refuses inconsistent expected arithmetic and reordered expected samples, and restores byte-identical private inputs/report. Existing CLI JSON/text cases remain unchanged in `fixtures_bam`. Default corpus boundaries must remain independent of `alignment`.

## Endogenous-count fixture controls

The existing alignment-gated `contracts` target registers `bam::endogenous_truth::endogenous_truth_preserves_count_fraction_and_admission_order`. It consumes immutable governed corpus inputs with private truth manifest/expected copies and restores their bytes and full report after meaningful admission controls. The four original CLI `fixtures_bam` JSON/text cases remain unchanged. Default corpus boundaries remain independent of the alignment feature.

## Coverage and gc-bias fixture controls

The existing alignment-gated `contracts` target registers `bam::coverage_gc_truth::coverage_gc_truth_preserves_weighted_regions_bins_and_admission`. It consumes immutable governed corpus inputs with private truth manifest/expected copies and restores their bytes and full report after meaningful admission controls. The four original CLI `fixtures_bam` JSON/text cases remain unchanged. Default corpus boundaries remain independent of the alignment feature.

## Haplogroup fixture controls

The existing alignment-gated `contracts` target registers `bam::haplogroup_truth::haplogroup_truth_preserves_marker_status_and_coverage_refusals`. It consumes immutable governed corpus inputs with private truth manifest/expected copies and restores their bytes and full report after meaningful admission controls. The four original CLI `fixtures_bam` JSON/text cases remain unchanged. Default corpus boundaries remain independent of the alignment feature.

## Sex-inference fixture controls

The existing alignment-gated `contracts` target registers `bam::sex_truth::sex_truth_preserves_call_classes_and_insufficiency_admission`. It consumes immutable governed corpus inputs with private truth manifest/expected copies and restores their bytes and full report after meaningful admission controls. The four original CLI `fixtures_bam` JSON/text cases remain unchanged. Default corpus boundaries remain independent of the alignment feature.
