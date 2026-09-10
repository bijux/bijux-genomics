# Population services

Each family exposes `run_local_vcf_<family>_smoke`, its persisted report contract,
and typed rows where applicable. `PopulationBenchmarkInputs` validates declaration
identity and multiplicity before effects. Services validate required bindings,
corpus identity and expected artifacts before execution.

PCA/admixture/population structure use supported bindings; ROH, IBD and demography
use governed templates. IBD also consumes explicitly scoped active readiness
observations. Acquiring declarations from other domains remains a composition concern.
