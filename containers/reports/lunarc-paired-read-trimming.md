# LUNARC paired-read trimming validation

## Question

Can the admitted Trim Galore 0.6.10 and Trimmomatic 0.39 Apptainer images
process the same non-fixture paired-end horse dataset on LUNARC while preserving
auditable input, command, output, scheduler, and failure evidence?

Yes. Both images completed their declared contracts. This result establishes a
real execution baseline; it does not establish that either parameterization is
optimal for ancient DNA, nor that one tool is scientifically superior.

## Evidence

| Tool | Run | Slurm job | Image SHA-256 prefix | Exit | Wall time |
|---|---|---:|---|---|---:|
| Trim Galore 0.6.10 | [run-1124](https://github.com/bijux/bijux-genomics/tree/main/containers/runs/run-1124-trim_galore) | `3458670` | `25567e0ec95d` | `0:0` | 8 s |
| Trimmomatic 0.39 | [run-1125](https://github.com/bijux/bijux-genomics/tree/main/containers/runs/run-1125-trimmomatic) | `3458671` | `329c9ec4abfd` | `0:0` | 5 s |

The input is the first 100,000 read pairs from ENA sample `ERR1021817`
(`PRJEB10854`). The two compressed FASTQ inputs are independently pinned as:

- read 1: `37e3200622685562d2fe7db41cd6b573eddfa23861719683026f1387e2bbeabc`;
- read 2: `14e65d2e2e43e8658bf04a9bf1f4ae90d2c41028b9ab0b86bafbfcc737d75fdf`.

Both runs used two CPUs, 4 GiB requested memory, disabled runtime networking,
job-local scratch, locale `C`, timezone `UTC`, and immutable repository commit
`5306d62788b314d1b0cfac8f6fa19ee55d45dd35`. Their evidence-lock SHA-256
values are `c0552c10ae812f856a9b05e80d34f269fd52dc75f00a77d6d7de3c5f27def537`
and `5dbeabae0b8edc03b6ed5e62ac3f5341f0d75e2701cbb985b291cb5bd488e534`.

## Observed output

The input contains 200,000 reads (100,000 pairs), 19,235,792 bases, and a mean
read length of 96.179 bases. Counts below were recomputed from the immutable
compressed outputs by counting FASTQ sequence lines; they are not inferred from
tool console prose.

| Tool | Paired reads retained | Pair retention | Retained bases | Base retention | Mean retained length |
|---|---:|---:|---:|---:|---:|
| Trim Galore | 199,996 | 99.998% | 19,135,124 | 99.477% | 95.678 bp |
| Trimmomatic | 182,040 | 91.020% | 17,166,622 | 89.243% | 94.301 bp |

Trim Galore ran its paired default behavior with two cores. Trimmomatic used
`SLIDINGWINDOW:4:20 MINLEN:30`. These are intentionally recorded operational
profiles, not matched algorithms. Their retention difference therefore cannot
be interpreted as an accuracy ranking.

The output pairs and their hashes are:

| Tool | Read 1 SHA-256 | Read 2 SHA-256 |
|---|---|---|
| Trim Galore | `3340613154aea4e0f70c9445e31bf03bfb38e25bb289ff50d90ead3f8e513783` | `4c186ac01312d6b579f3e1c3b6dd4dded0eeac0648736d39170d2512c25c04c2` |
| Trimmomatic | `660f03e061e43699ab153bacab06c898b6b1523eae78878f0468230c9e635bdb` | `0a6cf430f5a737ecbf1fb51a408d901f8050f71d5f8fed0f56a1abe0ab2b555f` |

## Recalculation

On LUNARC, the retained-read and retained-base observations were calculated
from each pair of output files with:

```console
gzip -cd READ_ONE.fastq.gz READ_TWO.fastq.gz \
  | awk 'NR % 4 == 2 { reads++; bases += length($0) } END { print reads, bases, bases / reads }'
```

The tracked run directories contain the exact commands, required outputs,
acceptance values, image and input identities, timestamps, and Slurm accounting.
Large FASTQ outputs remain under the governed study result root.

## Limits and next scientific check

This corpus slice proves execution on real sequence data but is too small and
too narrow to support a trimming recommendation. A scientific comparison must
predeclare adapter truth or a credible downstream mapping objective, include
modern and ancient libraries with different fragment-length distributions,
measure quality and adapter removal rather than retention alone, repeat timings,
and capture peak memory and I/O. The current LUNARC accounting lacks `MaxRSS`,
so the 5 s versus 8 s wall times are observations, not a benchmark claim.
