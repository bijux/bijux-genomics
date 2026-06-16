#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

# bam.align / bowtie2
/bin/sh -c 'bowtie2 -x assets/reference/host/references/toy_host_reference -1 /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/fastq/reads_1.fastq -2 /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/fastq/reads_2.fastq --very-sensitive --rg '"'"'@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1'"'"' --rg-id core-v1-align.rg1 -p 4 | samtools sort -o benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/flagstat.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/idxstats.txt && samtools stats benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/samtools_stats.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.metrics.json
import json
payload={"tool":"bowtie2","preset":"default","sensitivity_profile":"default","seed_length":null,"reference":"/Users/bijan/bijux/bijux-genomics/assets/reference/host/references/toy_host_reference.fasta","reference_index":"assets/reference/host/references/toy_host_reference","bam":"benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam","read_group":"@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1"}
print(json.dumps(payload, indent=2))
PY'

# bam.align / bwa
/bin/sh -c 'bwa mem -t 4 -R '"'"'@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1'"'"' /Users/bijan/bijux/bijux-genomics/assets/reference/host/references/toy_host_reference.fasta /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/fastq/reads_1.fastq /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/fastq/reads_2.fastq | samtools sort -o benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/flagstat.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/idxstats.txt && samtools stats benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/samtools_stats.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.metrics.json
import json
payload={"tool":"bwa_mem","preset":"default","sensitivity_profile":"default","seed_length":null,"reference":"/Users/bijan/bijux/bijux-genomics/assets/reference/host/references/toy_host_reference.fasta","bam":"benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam","read_group":"@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1"}
print(json.dumps(payload, indent=2))
PY'

# bam.authenticity / authenticct
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.flagstat.txt && samtools stats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.stats.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.flagstat.txt benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.stats.txt > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.json
import json,sys
flagstat,stats=sys.argv[1],sys.argv[2]
print(json.dumps({"method":"signal_aggregate","flagstat":flagstat,"stats":stats,"mode":"aggregate"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.summary.json
import json
print(json.dumps({"method": "signal_aggregate", "mode": "aggregate", "status": "ok"}, indent=2))
PY'

# bam.authenticity / damageprofiler
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.flagstat.txt && samtools stats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.stats.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.flagstat.txt benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.stats.txt > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.json
import json,sys
flagstat,stats=sys.argv[1],sys.argv[2]
print(json.dumps({"method":"signal_aggregate","flagstat":flagstat,"stats":stats,"mode":"aggregate"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.summary.json
import json
print(json.dumps({"method": "signal_aggregate", "mode": "aggregate", "status": "ok"}, indent=2))
PY'

# bam.authenticity / pmdtools
/bin/sh -c 'pmdtools --input /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --output benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/pmdtools/pmd.filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/pmdtools/authenticity.json && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/pmdtools/authenticity.summary.json
import json
print(json.dumps({"method": "pmdtools", "stage": "bam.authenticity"}, indent=2))
PY'

# bam.bias_mitigation / mapdamage2
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.bias_mitigation

# bam.complexity / preseq
/bin/sh -c 'preseq lc_extrap -o benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity_curve.tsv /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_complexity_projection.sam && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity.json
import json
print(json.dumps({"source": "preseq", "complexity_curve": "benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity_curve.tsv"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity.summary.json
import json
print(json.dumps({"stage": "bam.complexity", "complexity_curve": "benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity_curve.tsv"}, indent=2))
PY'

# bam.contamination / contammix
/bin/sh -c 'test -f /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam.bai && contammix --bam /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam --reference /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta --reference-panel benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/contammix/contamination.json && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/contammix/contamination.summary.json
import json
print(json.dumps(json.loads("{\n  \"assumptions\": \"governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning\",\n  \"emit_confidence_caveats\": true,\n  \"method\": \"contammix\",\n  \"minimum_mean_coverage\": 0.5,\n  \"reference\": \"/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta\",\n  \"reference_panels\": [\n    \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat\"\n  ],\n  \"scope\": \"nuclear\",\n  \"tool_scope\": \"nuclear\"\n}"), indent=2))
PY'

# bam.contamination / schmutzi
/bin/sh -c 'schmutzi --bam /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam --reference /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta --outdir benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.txt ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.txt benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.json; else : > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.json; fi && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.summary.json
import json
print(json.dumps(json.loads("{\n  \"assumptions\": \"governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning\",\n  \"emit_confidence_caveats\": true,\n  \"method\": \"schmutzi\",\n  \"minimum_mean_coverage\": 0.5,\n  \"reference\": \"/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta\",\n  \"reference_panels\": [],\n  \"scope\": \"mito\",\n  \"tool_scope\": \"mt\"\n}"), indent=2))
PY'

# bam.contamination / verifybamid2
/bin/sh -c 'test -f /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam.bai && verifybamid2 --BamFile /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam --Reference /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta --SVDPrefix benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat --Output benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.selfSM ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.selfSM benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.json; else : > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.json; fi && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.summary.json
import json
print(json.dumps(json.loads("{\n  \"assumptions\": \"governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning\",\n  \"emit_confidence_caveats\": true,\n  \"method\": \"verifybamid2\",\n  \"minimum_mean_coverage\": 0.5,\n  \"reference\": \"/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta\",\n  \"reference_panels\": [\n    \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat\"\n  ],\n  \"scope\": \"nuclear\",\n  \"tool_scope\": \"nuclear\"\n}"), indent=2))
PY'

# bam.coverage / bedtools
/bin/sh -c 'bedtools coverage -a benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed -b /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam >/dev/null && samtools depth -a -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/bedtools/coverage.depth.txt && awk '"'"'{sum+=$3; if($3>0) cov++} END {mean=(NR>0)?sum/NR:0; print "total", NR, cov, mean}'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/coverage/bedtools/coverage.depth.txt > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/bedtools/coverage.mosdepth.summary.txt'

# bam.coverage / mosdepth
/bin/sh -c 'mosdepth -n --by benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam && samtools depth -a -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage.depth.txt && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage.mosdepth.summary.txt ]; then :; else : > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage.mosdepth.summary.txt; fi'

# bam.coverage / samtools
/bin/sh -c 'samtools depth -a -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/samtools/coverage.depth.txt && awk '"'"'{sum+=$3; if($3>0) cov++} END {mean=(NR>0)?sum/NR:0; print "total", NR, cov, mean}'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/coverage/samtools/coverage.depth.txt > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/samtools/coverage.mosdepth.summary.txt'

# bam.damage / addeam
addeam --bam /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --out benchmarks/readiness/stage-tool-commands/bam/bam/damage/addeam/damage.addeam.json

# bam.damage / damageprofiler
damageprofiler --input /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --output benchmarks/readiness/stage-tool-commands/bam/bam/damage/damageprofiler/damage.profiler.json

# bam.damage / mapdamage2
/bin/sh -c 'mapDamage --bam /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --folder benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2 && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/misincorporation.txt ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/misincorporation.txt benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/damage.mapdamage2.txt; elif [ -f benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/5pCtoT.txt ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/5pCtoT.txt benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/damage.mapdamage2.txt; else : > benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/damage.mapdamage2.txt; fi'

# bam.damage / ngsbriggs
ngsbriggs --input /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --json-out benchmarks/readiness/stage-tool-commands/bam/bam/damage/ngsbriggs/damage.ngsbriggs.json

# bam.damage / pmdtools
/bin/sh -c 'pmdtools --input /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > /dev/null && python3 - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/damage/pmdtools/damage.pmdtools.json
import json
print(json.dumps({"tool": "pmdtools", "stage": "bam.damage", "c_to_t_5p": 0.0, "g_to_a_3p": 0.0, "pmd_score_histogram": []}, indent=2))
PY'

# bam.damage / pydamage
pydamage analyze --input /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --output benchmarks/readiness/stage-tool-commands/bam/bam/damage/pydamage/damage.pydamage.json --min-mapq 0.3

# bam.duplication_metrics / picard
/bin/sh -c 'picard MarkDuplicates I=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.tmp.bam M=benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.histogram.txt VALIDATION_STRINGENCY=SILENT ASSUME_SORTED=true && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.histogram.txt > benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.metrics.json
import json,sys
path=sys.argv[1]
metrics={"method":"picard","source":path}
for line in open(path):
    if line.startswith("LIBRARY"):
        values=next(open(path))
        cols=line.rstrip().split('"'"'\t'"'"')
        vals=values.rstrip().split('"'"'\t'"'"')
        if len(cols)==len(vals):
            row=dict(zip(cols,vals))
            metrics["pct_duplication"]=float(row.get("PERCENT_DUPLICATION",0.0) or 0.0)
            metrics["read_pair_duplicates"]=int(float(row.get("READ_PAIR_DUPLICATES",0) or 0))
        break
print(json.dumps(metrics, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.summary.json
import json
print(json.dumps({"stage": "bam.duplication_metrics", "method": "picard", "optical_duplicates": "MarkOnly", "duplicate_action": "Mark"}, indent=2))
PY'

# bam.duplication_metrics / samtools
/bin/sh -c 'samtools markdup -s /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.tmp.bam 2> benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.histogram.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.histogram.txt > benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.metrics.json
import json,re,sys
text=open(sys.argv[1]).read()
pairs=re.findall(r'"'"'EXAMINED:\s*(\d+)'"'"', text)
dups=re.findall(r'"'"'DUPLICATE PAIR:\s*(\d+)'"'"', text)
out={"method":"samtools","source":sys.argv[1],"examined_pairs":int(pairs[0]) if pairs else 0,"duplicate_pairs":int(dups[0]) if dups else 0}
print(json.dumps(out, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.summary.json
import json
print(json.dumps({"stage": "bam.duplication_metrics", "method": "samtools", "optical_duplicates": "MarkOnly", "duplicate_action": "Mark"}, indent=2))
PY'

# bam.endogenous_content / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/endogenous_content/samtools/flagstat.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/endogenous_content/samtools/endogenous.content.json
import json
payload = {"stage": "bam.endogenous_content", "method": "mapped_fraction_from_flagstat", "flagstat": "benchmarks/readiness/stage-tool-commands/bam/bam/endogenous_content/samtools/flagstat.txt", "host_reference_scope": "human_host", "host_reference_digest": null, "refuse_without_host_reference": true}
print(json.dumps(payload, indent=2))
PY'

# bam.filter / bamtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/idxstats.before.txt && bamtools stats -in /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam >/dev/null && samtools view -b /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filter.summary.json
import json
print(json.dumps({"filter_tool": "bamtools"}, indent=2))
PY'

# bam.filter / bedtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/idxstats.before.txt && bedtools bamtobed -i /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam >/dev/null && samtools view -b /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filter.summary.json
import json
print(json.dumps({"filter_tool": "bedtools"}, indent=2))
PY'

# bam.filter / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.before.txt && samtools view -h -b -q 20 -F 4,1024 /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam | awk '"'"'BEGIN{OFS="\t"} /^@/{print; next} length($10)>=8'"'"' | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filter.summary.json
import json
payload = {"action": "filter", "input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam", "params": {"mapq_threshold": 20, "min_length": 8, "remove_duplicates": true}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.gc_bias / picard
picard CollectGcBiasMetrics 'I=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam' 'R=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta' 'O=benchmarks/readiness/stage-tool-commands/bam/bam/gc_bias/picard/gc_bias.metrics.txt' 'S=benchmarks/readiness/stage-tool-commands/bam/bam/gc_bias/picard/gc_bias.summary.json' 'CHART=benchmarks/readiness/stage-tool-commands/bam/bam/gc_bias/picard/gc_bias.plot.pdf'

# bam.genotyping / angsd
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.genotyping

# bam.haplogroups / yleaf
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.haplogroups

# bam.insert_size / picard
picard CollectInsertSizeMetrics 'I=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_insert_size_triplet.sam' 'O=benchmarks/readiness/stage-tool-commands/bam/bam/insert_size/picard/insert_size.metrics.txt' 'H=benchmarks/readiness/stage-tool-commands/bam/bam/insert_size/picard/insert_size.histogram.pdf' 'M=0.5'

# bam.kinship / angsd
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.kinship

# bam.kinship / king
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.kinship

# bam.length_filter / picard
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.before.txt && samtools view -h -b -q 0 /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam | awk '"'"'BEGIN{OFS="\t"} /^@/{print; next} length($10)>=8'"'"' | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/length_filter.summary.json
import json
payload = {"action": "length_filter", "input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam", "params": {"mapq_threshold": 0, "min_length": 8, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.length_filter / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.before.txt && samtools view -h -b -q 0 /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam | awk '"'"'BEGIN{OFS="\t"} /^@/{print; next} length($10)>=8'"'"' | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/length_filter.summary.json
import json
payload = {"action": "length_filter", "input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam", "params": {"mapq_threshold": 0, "min_length": 8, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.mapping_summary / picard
/bin/sh -c 'picard CollectAlignmentSummaryMetrics I=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/alignment_summary.metrics.txt VALIDATION_STRINGENCY=SILENT && picard BamIndexStats I=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/idxstats.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/alignment_summary.metrics.txt benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/flagstat.txt benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/mapping.summary.json
import csv,json,sys
metrics_path, flagstat_path, summary_path = sys.argv[1:4]
rows = []
capture = False
with open(metrics_path, '"'"'r'"'"', encoding='"'"'utf-8'"'"') as handle:
for raw in handle:
line = raw.rstrip('"'"'\n'"'"')
if line.startswith('"'"'## METRICS CLASS'"'"'):
capture = True
continue
if not capture or not line or line.startswith('"'"'#'"'"'):
continue
rows.append(line)
if len(rows) == 2:
break
metrics = {}
if len(rows) == 2:
reader = csv.DictReader(rows, delimiter='"'"'\t'"'"')
metrics = next(reader, {})
total_reads = int(float(metrics.get('"'"'TOTAL_READS'"'"', 0) or 0))
mapped_reads = int(float(metrics.get('"'"'PF_READS_ALIGNED'"'"', 0) or 0))
mapped_fraction = (mapped_reads / total_reads * 100.0) if total_reads else 0.0
with open(flagstat_path, '"'"'w'"'"', encoding='"'"'utf-8'"'"') as handle:
handle.write(f"{total_reads} + 0 in total (QC-passed reads + QC-failed reads)\n")
handle.write(f"{mapped_reads} + 0 mapped ({mapped_fraction:.2f}% : N/A)\n")
with open(summary_path, '"'"'w'"'"', encoding='"'"'utf-8'"'"') as handle:
json.dump({"stage": "bam.mapping_summary", "tool": "picard", "flagstat": flagstat_path, "idxstats": "benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/idxstats.txt", "stats": metrics_path}, handle, indent=2)
handle.write('"'"'\n'"'"')
PY'

# bam.mapping_summary / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/flagstat.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/idxstats.txt && samtools stats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/samtools_stats.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/mapping.summary.json
import json
print(json.dumps({"stage":"bam.mapping_summary","flagstat":"benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/flagstat.txt","idxstats":"benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/idxstats.txt","stats":"benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/samtools_stats.txt"}, indent=2))
PY'

# bam.mapq_filter / bamtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.before.txt && samtools view -h -b -q 30 /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam | cat | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/mapq_filter.summary.json
import json
payload = {"action": "mapq_filter", "input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam", "params": {"mapq_threshold": 30, "min_length": 0, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.mapq_filter / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.before.txt && samtools view -h -b -q 30 /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam | cat | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/mapq_filter.summary.json
import json
payload = {"action": "mapq_filter", "input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam", "params": {"mapq_threshold": 30, "min_length": 0, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.markdup / picard
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.before.txt && picard MarkDuplicates I=/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam M=benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.metrics.txt VALIDATION_STRINGENCY=SILENT ASSUME_SORTED=true REMOVE_DUPLICATES=false && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.after.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.metrics.txt > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.summary.json
import json,sys
payload={"input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam", "metrics": sys.argv[1], "remove_duplicates": false, "tool": "picard", "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.markdup / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.before.txt && samtools markdup /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.summary.json
import json
payload = {"input_bam": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam", "remove_duplicates": false, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam.overlap_correction / bamutil
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/flagstat.before.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/idxstats.before.txt && bam clipOverlap --in /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam --out benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam --stats > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap_correction.summary.json.clipoverlap.log 2>&1 && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap_correction.summary.json
import json
print(json.dumps({"method": "bamutil.clipOverlap", "input": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam", "output": "benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam"}, indent=2))
PY'

# bam.qc_pre / multiqc
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/multiqc/flagstat.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/multiqc/idxstats.txt && samtools stats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/multiqc/samtools_stats.txt'

# bam.qc_pre / samtools
/bin/sh -c 'samtools flagstat /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/samtools/flagstat.txt && samtools idxstats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/samtools/idxstats.txt && samtools stats /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/samtools/samtools_stats.txt'

# bam.recalibration / gatk
/bin/sh -c 'cp /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam && printf '"'"'tiny-index\n'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam.bai && cat <<'"'"'EOF'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.report.txt
status=skipped
reason=requested_skip_mode
EOF
python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.summary.json
import json
print(json.dumps({"mode": "skip", "status": "skipped", "reason": "requested_skip_mode", "known_sites": ["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"], "reference": "/Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta", "recalibration_report": "benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.report.txt", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam", "output_bai": "benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam.bai"}, indent=2))
PY'

# bam.sex / angsd
/bin/sh -c 'angsd -i /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam -doCounts 1 -dumpCounts 2 -out benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.angsd && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.json
import json
print(json.dumps({"method": "angsd", "counts_prefix": "benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.angsd"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.summary.json
import json
payload = {"method": "rxy", "backend": "angsd", "x_to_y_ratio": 0.0, "confidence": 0.0}
print(json.dumps(payload, indent=2))
PY'

# bam.sex / rxy
/bin/sh -c 'rxy --input /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/sex/rxy/sex.json && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/rxy/sex.summary.json
import json
payload = {"method": "rxy", "x_to_y_ratio": 0.0, "confidence": 0.0}
print(json.dumps(payload, indent=2))
PY'

# bam.sex / yleaf
/bin/sh -c 'yleaf -bam /Users/bijan/bijux/bijux-genomics/benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam -o benchmarks/readiness/stage-tool-commands/bam/bam/sex/yleaf/sex --reference_genome hg38 && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/yleaf/sex.json
import json
print(json.dumps({"method": "yleaf", "backend": "yleaf", "chromosome_system": "xy"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/yleaf/sex.summary.json
import json
payload = {"method": "rxy", "backend": "yleaf", "minimum_y_sites": 5, "x_to_y_ratio": 0.0, "confidence": 0.0}
print(json.dumps(payload, indent=2))
PY'

# bam.validate / bamtools
/bin/sh -c 'bamtools stats -in /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bamtools/validation.json && samtools flagstat /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bamtools/flagstat.txt'

# bam.validate / bedtools
/bin/sh -c 'bedtools bamtobed -i /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/bam/validation_pass.bam >/dev/null && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bedtools/validation.json
import json
print(json.dumps({"validator": "bedtools.bamtobed", "status": "ok"}, indent=2))
PY && samtools flagstat /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bedtools/flagstat.txt'

# bam.validate / samtools
/bin/sh -c 'samtools quickcheck -v /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/samtools/validation.json && samtools flagstat /Users/bijan/bijux/bijux-genomics/assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/samtools/flagstat.txt'
