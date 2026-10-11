# Workflow interface

`amplicon::render_amplicon_micro_pipeline` receives an explicit repository and
output report path. Core germline and aDNA services additionally receive typed
alignment fixture operations; germline also requires four admitted VCF tool bindings.
Constructors perform no fixture effects. Missing, foreign, repeated or malformed
bindings fail admission before workflow execution.

Reports retain family-specific schema identities, ordered stage rows and handoff
observations. Caller-selected report parents own only their declared generated
outputs. A matching prior report is required to retire old output files; source
references and unrelated files are preserved. Unclaimed artifacts and symbolic
output declarations fail closed.
