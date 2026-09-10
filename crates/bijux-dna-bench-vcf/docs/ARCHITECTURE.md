# VCF adapter ownership

The CLI obtains the stage catalog and supported binding observations, admits
those inputs once, supplies the reference materialization operation and adapts
command arguments. Adapter families receive that explicit immutable input.
They do not locate the workspace or call cross-domain readiness collectors.

`adapters/` groups seven tool families. Each separates wire models, command
contracts, admission, fixture preparation, registry IO and publication. Tool
specific schemas and serialized field identities are retained. Shared catalog
and matrix wire types remain in the benchmark contract library; disabling its
optional FASTQ catalog keeps that normal dependency out of VCF compilation.

`fixtures/inputs.rs` handles governed VCF manifest paths and native indexing.
`fixtures/panel.rs` resolves reference metadata and writes explicitly classified
local marker files. Reference metadata resolution remains read-only in db-ref.
The reference FASTA writer is a typed operation supplied by composition, so the
adapter library has no normal dependency on the benchmark workflow owner.

Command validation and marker existence do not prove that a real scientific tool
can consume a fixture. Version-aware native payload admission and downstream
substitution require independent evidence. This boundary retains that limitation
rather than labeling synthetic panel markers as native panel artifacts.

Stage constructors live under each multi-stage family's `stages/` directory.
BCFtools, ANGSD, PLINK and descent dispatch to 26 separate stage implementations;
named `StageContract` fields connect command construction to report assembly.
Phasing and imputation isolate engine invocation from fixture preparation.
Imputation also owns a named output layout that derives paths and declarations;
EIGENSOFT command generation borrows a named collection of parameter/output files.
These private contracts preserve serialized public report fields while avoiding
large positional tuples and tool dispatch inside report publication.
