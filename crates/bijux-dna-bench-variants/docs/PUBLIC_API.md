# Variant benchmark services

`inputs::VariantBenchmarkInputs::new` admits governed processing templates and
explicit QC binding rows before service effects. `filter`, `damage_filter`,
`postprocess` and `qc` export their workflow and existing report/metric schemas.
CLI composition supplies current repository declarations; owner tests supply the
same authored matrix with explicit report-format bindings.

Filtering and postprocessing preserve their native VCF stage behavior. Damage
filtering passes its remove-mode, non-UDG scientific policy explicitly without
mutating process-wide environment settings.

QC tool selection names a report dialect. All three supported formats (bcftools,
PLINK and PLINK2) use internal Bijux QC observations. They do not certify execution
of those external tools. `execution-provenance.json`, its ownership-inventory entry and
projected logs disclose this distinction. The fixed legacy stage-result output
list remains unchanged. Existing report schema fields are kept.

## Cohort statistics

`statistics::run_local_vcf_stats_smoke` takes admitted variant inputs, an explicit
repository root and tool identity. The governed `vcf.stats` declaration retains
`bcftools`, the cohort corpus/asset and the `stats_json` role. Its minimal authored
VCF supplies variant, SNP, indel, sample and transition/transversion truth; normalized
native statistics must agree before metrics and stage-result publication.

Statistics uses the variant artifact ownership protocol. Each successful execution
remains under `executions`; stable artifacts and stage-result addresses are projected
by the shared publisher. Unowned prior metrics are refused without replacement.
Report field names, schema identifiers and stable output locations remain compatible.

## Variant readiness

`readiness::{qc, stats, filter, damage_filter}` owns stage readiness assessment,
scientific smoke policy and report publication beside the variant execution services.
Callers supply admitted `VariantBenchmarkInputs`, an immutable `VcfEvidenceSnapshot`,
command publication addresses, repository root and report path. The owner verifies
snapshot corpus, asset, adapter, parser and schema agreement before native work.
QC binds each retained tool to its own admitted declaration.

`ReadinessCommandPaths` aliases the independent VCF evidence publication-address
record, shared with genotype readiness. Addresses do not establish command readiness;
that observation comes from the snapshot. Existing wire fields, scientific predicates,
active-scope selection and incomplete diagnostic publication remain compatible.
Coverage policy, scientific QC policy, wire projection and publication have separate
modules. These local reports do not establish remote or full tool substitution proof.
