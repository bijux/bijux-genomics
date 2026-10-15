# bijux-dna-domain-fastq Architecture

`bijux-dna-domain-fastq` owns FASTQ typed domain truth and contract queries. Its
pure value kernels are separate from retained path-based library operations that
acquire inputs, publish files and normalize output names. Execution crates decide
when and how to run external tools.

## Layout

```text
src/
  artifacts/              stage report and manifest schemas
  banks/                  adapter, contaminant, and polyX banks plus selection
    adapter/              adapter bank models, validation, and preset resolution
    contaminant/          contaminant bank models, validation, and preset resolution
    polyx/                polyX bank models, validation, and preset resolution
    selection/            effective bank selection and provenance helpers
  bench/                  benchmark query context and repository contracts
  comparison_contract/    stage comparison artifact contracts
  execution_support/      manifest-backed execution support catalog
  integration_matrix/     stage/tool compatibility and benchmark scenarios
  invariants/             invariant specs, thresholds, and evaluation
    metrics/              metric invariant evaluators grouped by metric family
  metrics/                metric types, classes, specs, and deltas
    spec/                 metric specification catalog and classes
    types/                shared, tool, summary, and stage metric value types
  observer/               parser contracts and governed parser implementations
    contracts/            observer contract catalog and query helpers
    parse/                parser implementations grouped by report family
  params/                 descriptors, defaults, parsing, and effective params
    defaults/             shared, processing, profiling, and quality defaults
    descriptor/           stage parameter descriptors grouped by semantic family
    processing/           typed processing and transform parameter models
    quality/              typed quality, screen, trim, and validation parameter models
  pipeline_contract/      pipeline ordering, transitions, and dependency graph
    catalog/              criticality, modes, ordering, and transitions
    graph/                assembly, dependencies, and edge construction
  run/                    FASTQ input discovery and benchmark corpus helpers
  stage_tool_governance/  tool layout, readiness, maturity, and input layout policy
  stages/                 stage IDs, specs, ports, semantics, and contract JSON
    abundance_normalization.rs pure table-text normalization and observations
    abundance_normalization/   unit, ordering, and text-admission contracts
    primer_normalization.rs deterministic native primer-prefix computation on record values
    primer_normalization/   scientific computation contracts without filesystem setup
    contract/             contract catalog, export helpers, and path operations
      runtime/            retained input acquisition, local transforms and output publication
    otu_clustering.rs     pure native representative membership and aligned abundances
    otu_clustering/       order, threshold and record-retention contracts
    ports/                governed manifest ports and query helpers
    semantics/            semantic catalog and query helpers
  types/                  shared FASTQ domain value types
    record.rs             FASTQ record identity, content, and header pairing semantics
  lib.rs                  public facade and compatibility re-exports
```

## Ownership Rules

- `stages/` is the authority for FASTQ stage IDs, semantics, IO, and contract JSON.
- `stages/abundance_normalization` owns the borrowed-text normalization and table
  observation capability used by API artifact composition. It preserves six-decimal
  output, sorted sample grouping, duplicate row contributions, declared scaling,
  and existing malformed/non-finite observation semantics. API adapters retain
  acquisition, publication, execution, and benchmark persistence. The existing
  eight-decimal `stages::contract::normalize_abundance` path operation retains its
  separate behavior; this capability does not reconcile the two algorithms.
- `stages/primer_normalization` owns the pure native local-smoke prefix transform and its typed
  admission error. It keeps the two exact primer motifs, overlap threshold, and paired fraction
  semantics. Orientation, primer-bank, mismatch, and IUPAC metadata do not add algorithm behavior.
  The module and shared record value are crate-private. The existing path-based
  `stages::contract::normalize_primers` operation retains acquisition, publication, and report
  projection; this separation does not relocate its filesystem effects.
- `stages/otu_clustering` owns crate-private native local-smoke membership on
  admitted FASTQ records. It preserves input-order representative selection, first-match
  assignment, common-prefix identity and direct threshold comparison behavior. The
  existing `stages::contract::cluster_otus` operation retains filesystem acquisition,
  table/FASTA/FASTQ formatting, sequential publication and report projection. This
  kernel does not execute vsearch or replace its scientific clustering algorithm.
- `params/`, `metrics/`, and `invariants/` must stay aligned; changing one usually requires tests
  for the others.
- `banks/` owns bank content, preset resolution, provenance, and deterministic hashes.
- `observer/` may parse governed tool reports, but it must not execute tools or own runtime policy.
- `execution_support/`, `integration_matrix/`, and `stage_tool_governance/` expose domain readiness
  and compatibility metadata; they do not schedule execution.
- `run/` exposes FASTQ discovery and benchmark corpus descriptors. It also reexports
  the core input-assessment publication helper; writing that assessment is an explicit
  caller-selected filesystem effect, without runner orchestration.
- `stages::contract` retains path-based FASTQ/table/report acquisition, local
  transforms, directory creation, file publication and output renaming. These APIs
  are distinct from the pure borrowed-value kernels; they do not launch tools.

## Data Flow

1. Domain metadata and typed modules define IDs, artifacts, params, metrics, and invariants.
2. Contract query APIs expose stable views to planners, stages, benchmark tooling, and analyzers.
3. Parser and observer helpers normalize governed tool outputs into typed reports.
4. Tests lock parity with `domain/fastq/` manifests, snapshots, and semantic fixtures.
