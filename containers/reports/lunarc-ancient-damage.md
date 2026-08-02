# LUNARC ancient-DNA damage validation

This report records runtime and stage qualification of the ngsBriggs container
on public ancient-horse run `ERR2112767` (`Borly4_PAVH8_4978`, BioSample
`SAMEA104233654`) against EquCab3.0. The source alignment and reference were
admitted by checksum. Every execution used an immutable Apptainer image, a
clean commit-addressed checkout, a governed output root, and terminal Slurm
accounting.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-ancient-damage.json).
Raw inputs, images, logs, and outputs remain under the LUNARC study root read
from `containers/contracts/sites/lunarc.yml` and archived with each run.

## Execution chain

| Purpose | Tool | Run | Slurm job | Status | Elapsed | Principal result |
|---|---|---:|---:|---|---:|---|
| expose the upstream input mismatch | ngsBriggs 0.01 | `run-1043` | `3457815` | failed as expected | 5s | the paired/unmapped source BAM was rejected |
| expose an acceptance-regex defect | Samtools 1.20 | `run-1044` | `3457816` | failed | 3s | valid output was not admitted |
| derive the documented single-end input | Samtools 1.20 | `run-1045` | `3457817` | passed | 2s | 482,758-byte checksummed BAM |
| measure terminal damage | ngsBriggs 0.01 | `run-1046` | `3457823` | passed | 5s | governed JSON and three native result files |

The preparation command used `samtools view -b -F 5`: bit `0x1` removes paired
records and bit `0x4` removes unmapped records. This is an explicit adapter to
ngsBriggs' single-end input contract, not an undocumented mutation of the
source BAM. The derived BAM records its parent digest, command, image, Slurm
job, and evidence digest in the governed data catalog.

`run-1043` also showed that the raw upstream executable can return success for
a missing BAM. The production wrapper now validates BAM, reference, and FASTA
index inputs before execution and exposes the stable interface
`ngsbriggs analyze --bam … --reference … --json-out …`. In `run-1046`, the
negative check returned `66`, while version, help, and real execution passed in
the same job.

## Observed outputs

The admitted result contains:

- `damage.ngsbriggs.json`, conforming to `bijux.bam.damage.v1`;
- the native misincorporation matrix;
- inferred model parameters; and
- the inferred read-length distribution.

The terminal rates in this deliberately small admitted slice were
`C→T (5′) = 0.0` and `G→A (3′) = 0.0`. Those values demonstrate parsing and
contract closure only. They are not evidence that the sample lacks ancient-DNA
damage.

## Claim boundary

This evidence closes the ngsBriggs runtime and real-stage contracts on LUNARC.
It verifies immutable inputs, negative-input behavior, native output retention,
governed JSON conversion, and Slurm provenance. It does not establish damage
detection sensitivity, ancient-DNA authenticity, biological acceptance,
cross-tool agreement, benchmark variance, or release qualification.

The admitted BAM was produced by an external preprocessing pipeline and the
single-end compatibility slice is small. A scientific claim requires a
predeclared corpus, positive and negative controls, minimum informative-base
counts, acceptance thresholds, and comparison with another retained damage
profiler.
