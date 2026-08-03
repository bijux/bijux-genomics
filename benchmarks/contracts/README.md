# Scientific benchmark contracts

This directory owns the reviewed scientific policy that exists before a
benchmark is executed. It is separate from container contracts: container
contracts establish image identity and runtime confinement, while these files
define which real data, tools, measurements, truth strategies, and downstream
substitution probes make a stage comparison admissible.

`portfolio.toml` reserves one durable run identity for every retained FASTQ,
BAM, and VCF stage and binds each stage to its corpus and scientific profile
requirements. `stage-campaigns.toml` contains only executable campaigns whose
complete retained-tool closure, inputs, parameters, resource measurements,
quality metrics, interpretation, and limitations have been reviewed.

Run specifications and raw evidence do not belong here. Specifications are
published under the governed study result root, large execution evidence stays
under `STUDY/results/benchmarks/`, and small reviewed reports belong in
`benchmarks/reports/`.
