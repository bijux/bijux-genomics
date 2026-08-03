# LUNARC ancient-DNA damage validation

This report records runtime and stage qualification of the ngsBriggs,
mapDamage 2, AuthentiCT, AdDeam, and contamMix containers on public ancient-horse run
`ERR2112767`
(`Borly4_PAVH8_4978`, BioSample `SAMEA104233654`) against EquCab3.0. The source
alignment and reference were admitted by checksum. Every execution used an
immutable Apptainer image, a clean commit-addressed checkout, a governed output
root, and terminal Slurm accounting.

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
| expose the unconditional R probe | mapDamage 2.2.2 | `run-1047` | `3457852` | failed | 4s | table-only mode still invoked absent `Rscript` |
| expose reliance on plotting-derived tables | mapDamage 2.2.2 | `run-1048` | `3457858` | failed | 29s | native analysis passed but projection refused absent plot tables |
| validate native-table rate projection | mapDamage 2.2.2 | `run-1049` | `3457866` | passed | 30s | independently confirmed terminal rates |
| bound reference and runtime footprint | mapDamage 2.2.2 | `run-1050` | `3457867` | passed | 17s | same rates with a 56.8 MB image and 41,402 bytes of native tables |
| expose an acceptance-regex defect | Samtools 1.20 | `run-1090` | `3458076` | failed | 1s | valid SAM was not admitted |
| materialize the MD-tagged alignment stream | Samtools 1.20 | `run-1092` | `3458078` | passed | 1s | 86,363,583-byte checksummed SAM |
| measure terminal deamination | AuthentiCT 1.0.0 | `run-1093` | `3458079` | passed | 40s | 7,817-byte deamination table |
| expose an omitted BAM-index bind | AdDeam 1.0.0 | `run-1095` | `3458116` | failed | 1s | native profiling rejected an inaccessible BAI |
| generate terminal damage profiles | AdDeam 1.0.0 | `run-1096` | `3458117` | passed | 2s | native 5′ and 3′ substitution profiles for 244,634 mapped records |
| isolate mitochondrial evidence | Samtools 1.20 | `run-1107` | `3458129` | passed | 1s | 28 checksum-admitted `NC_001640.1` records |
| estimate mitochondrial mixture | contamMix 1.0.11 | `run-1108` | `3458130` | passed | 3s | three converged chains and native R diagnostics |

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

The mapDamage chain preserved each failure instead of weakening acceptance.
The first admitted image lacked the bundled `seqtk` executable; the corrected
image then exposed an upstream parser defect that invoked `Rscript` after
declaring plotting unavailable. A narrow source patch skips only that probe
when R-dependent plotting is disabled. The next run showed that the Bijux
projection wrongly depended on `5pCtoT_freq.txt` and `3pGtoA_freq.txt`, which
are created by the disabled R plotting path. The production wrapper now
aggregates `C>T / C` at the 5′ first position and `G>A / G` at the 3′ first
position directly from the primary `misincorporation.txt` table, matching the
upstream calculation.

`run-1049` passed but retained per-contig rows for 4,701 reference sequences,
expanding native tables to 148,817,327 bytes. The global stage metric already
aggregates those rows, so `run-1050` uses upstream
`--merge-reference-sequences`. This preserved the exact canonical result while
reducing native tables to 41,402 bytes and elapsed time from 30 to 17 seconds.
The final image also removes build-only compilers, package installers, and
headers after compiling `seqtk`, reducing the SIF from 175,280,128 to
56,786,944 bytes. A failed intermediate build remains recorded because it
proved that `gcc`, `make`, and zlib headers are real build dependencies.

AuthentiCT's documented interface reads SAM text with an `MD` tag. The source
corpus is a digest-pinned BAM, so `run-1092` materialized the complete SAM
stream with Samtools and retained the output as an upstream result. Preserved
`run-1090` produced valid content but failed admission because its version
regex assumed Samtools emitted only one line; the corrected contract accepts
the documented leading version line without weakening any execution check.
`run-1093` consumed that exact SAM digest with mapping-quality, read-length,
and base-quality cutoffs of 20, 30, and 20 respectively.

AdDeam's upstream Python driver invoked the native profiler twice and did not
propagate a failed native exit status. The admitted image applies a reviewed
source patch that executes each BAM once and raises on failure. That correction
made `run-1095` expose a missing BAI bind instead of reporting false success.
The failure remains archived. `run-1096` declares and mounts the checksum-
admitted BAM, BAI, and one-line BAM list, then executes classic combined-strand
profiling with mapping quality 20, minimum read length 30, ten terminal
positions, and a declared convergence configuration.

contamMix requires a mitochondrial alignment rather than the whole EquCab3
reference. `run-1107` used the admitted Samtools image to extract all 28
`NC_001640.1` records from the public ancient-horse BAM. The candidate panel is
a coordinate-homologous alignment of three public NCBI mitochondrial records:
the EquCab3 `NC_001640.1` consensus plus `OM869135.2` and `OM869133.2`, which
differ from that consensus at 48 and 97 positions. Their accessions, retrieval
URLs, lengths, ambiguity counts, substitutions, and SHA-256 digests are
recorded in the governed data catalog. `run-1108` used the upstream default
50,000 iterations across three chains with two worker threads and retained the
complete R workspace and diagnostic PDF.

## Observed outputs

The admitted result contains:

- `damage.ngsbriggs.json`, conforming to `bijux.bam.damage.v1`;
- the native misincorporation matrix;
- inferred model parameters; and
- the inferred read-length distribution.

The mapDamage result contains:

- `damage.mapdamage2.txt`, conforming to the stage parser contract;
- the native misincorporation matrix;
- the native nucleotide-composition table; and
- the native read-length distribution.

The AuthentiCT result contains the native position-wise deamination table with
derived and total counts plus confidence intervals. The rates at positions
`-1` and `1` were `0.0019827326573637187` and `0.003279650170648464` from
79/39,844 and 123/37,504 observations respectively. These are recorded
measurements, not authenticity or contamination acceptance decisions.

The AdDeam result contains the native 5′ and 3′ substitution matrices. The
terminal combined-strand rates were `C→T (5′) = 0.00806452` and
`G→A (3′) = 0.00694444` across 244,634 mapped records. These are observed
profile values under the declared convergence settings, not authenticity or
damage-detection acceptance decisions.

The contamMix posterior had a maximum-a-posteriori authentic fraction of
`0.2705522` and a 95% interval from `0.02686201` to `0.7980419`, with an
inferred error rate of `0.0054`. The Gelman diagnostic was `1.000006` with an
upper confidence limit of `1.000031`, below the upstream `1.1` convergence
threshold. This demonstrates that the container completes its real MCMC
workflow and produces converged chains. The interval is deliberately reported
in full: 28 mitochondrial reads and three modern-horse candidates do not
support a biological contamination estimate.

The terminal rates in this deliberately small admitted slice were
`C→T (5′) = 0.0` and `G→A (3′) = 0.0`. Those values demonstrate parsing and
contract closure only. They are not evidence that the sample lacks ancient-DNA
damage.

The mapDamage terminal rates were `C→T (5′) = 0.00361819` and
`G→A (3′) = 0.00340736`. An independent calculation from 147 of 40,628
informative C observations and 139 of 40,794 informative G observations
reproduced those values. Reference merging did not change either rate.

## Claim boundary

This evidence closes the ngsBriggs, mapDamage, AuthentiCT, AdDeam, and
contamMix runtime and real-stage contracts on LUNARC. It verifies immutable inputs,
negative-input behavior, native output retention, governed projection,
independently recalculated rates, and Slurm provenance. AuthentiCT's qualified
command measures its native
deamination surface; it does not run `deam2cont` or establish that the source
library satisfies AuthentiCT's single-stranded-library assumptions. The
evidence does not establish damage detection sensitivity, ancient-DNA
authenticity, biological acceptance, cross-tool agreement, benchmark variance,
or release qualification.

The admitted BAM was produced by an external preprocessing pipeline and the
single-end compatibility slice is small. A scientific claim requires a
predeclared corpus, positive and negative controls, minimum informative-base
counts, acceptance thresholds, and comparison with another retained damage
profiler. AdDeam's qualified command generates native damage matrices; it does
not establish that its early-convergence settings preserve every feature of a
full-corpus profile. contamMix's qualified command establishes runtime and
chain convergence only; the sparse mitochondrial slice and deliberately small
candidate panel prohibit an authenticity or contamination conclusion.
