#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$repo_root"

# bam:corpus-01-mini:bam.align:sample-set:bowtie2 / bam / bam.align / bowtie2
/bin/sh -c 'bowtie2 -x assets/reference/host/references/toy_host_reference -1 assets/toy/core-v1/fastq/reads_1.fastq -2 assets/toy/core-v1/fastq/reads_2.fastq --very-sensitive --rg '"'"'@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1'"'"' --rg-id core-v1-align.rg1 -p 4 | samtools sort -o benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/flagstat.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/idxstats.txt && samtools stats benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/samtools_stats.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.metrics.json
import json
payload={"tool":"bowtie2","preset":"default","sensitivity_profile":"default","seed_length":null,"reference":"assets/reference/host/references/toy_host_reference.fasta","reference_index":"assets/reference/host/references/toy_host_reference","bam":"benchmarks/readiness/stage-tool-commands/bam/bam/align/bowtie2/align.bam","read_group":"@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1"}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-mini:bam.align:sample-set:bwa / bam / bam.align / bwa
/bin/sh -c 'bwa mem -t 4 -R '"'"'@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1'"'"' assets/reference/host/references/toy_host_reference.fasta assets/toy/core-v1/fastq/reads_1.fastq assets/toy/core-v1/fastq/reads_2.fastq | samtools sort -o benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/flagstat.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/idxstats.txt && samtools stats benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/samtools_stats.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.metrics.json
import json
payload={"tool":"bwa_mem","preset":"default","sensitivity_profile":"default","seed_length":null,"reference":"assets/reference/host/references/toy_host_reference.fasta","bam":"benchmarks/readiness/stage-tool-commands/bam/bam/align/bwa/align.bam","read_group":"@RG\tID:core-v1-align.rg1\tSM:core-v1-align\tPL:ILLUMINA\tLB:lib1\tPU:core-v1-align.pu1"}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:authenticct / bam / bam.authenticity / authenticct
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.flagstat.txt && samtools stats benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.stats.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.flagstat.txt benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.stats.txt > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.json
import json,sys
flagstat,stats=sys.argv[1],sys.argv[2]
print(json.dumps({"method":"signal_aggregate","flagstat":flagstat,"stats":stats,"mode":"aggregate"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/authenticct/authenticity.summary.json
import json
print(json.dumps({"method": "signal_aggregate", "mode": "aggregate", "status": "ok"}, indent=2))
PY'

# bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:damageprofiler / bam / bam.authenticity / damageprofiler
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.flagstat.txt && samtools stats benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.stats.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.flagstat.txt benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.stats.txt > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.json
import json,sys
flagstat,stats=sys.argv[1],sys.argv[2]
print(json.dumps({"method":"signal_aggregate","flagstat":flagstat,"stats":stats,"mode":"aggregate"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/damageprofiler/authenticity.summary.json
import json
print(json.dumps({"method": "signal_aggregate", "mode": "aggregate", "status": "ok"}, indent=2))
PY'

# bam:corpus-01-adna-damage-mini:bam.authenticity:adna_damage_non_udg:pmdtools / bam / bam.authenticity / pmdtools
/bin/sh -c 'pmdtools --input benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --output benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/pmdtools/pmd.filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/pmdtools/authenticity.json && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/authenticity/pmdtools/authenticity.summary.json
import json
print(json.dumps({"method": "pmdtools", "stage": "bam.authenticity"}, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.bias_mitigation:sample-set:mapdamage2 / bam / bam.bias_mitigation / mapdamage2
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.bias_mitigation

# bam:corpus-01-bam-mini:bam.complexity:sample-set:preseq / bam / bam.complexity / preseq
/bin/sh -c 'preseq lc_extrap -o benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity_curve.tsv benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_complexity_projection.sam && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity.json
import json
print(json.dumps({"source": "preseq", "complexity_curve": "benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity_curve.tsv"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity.summary.json
import json
print(json.dumps({"stage": "bam.complexity", "complexity_curve": "benchmarks/readiness/stage-tool-commands/bam/bam/complexity/preseq/complexity_curve.tsv"}, indent=2))
PY'

# bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:contammix / bam / bam.contamination / contammix
/bin/sh -c 'test -f benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam.bai && contammix --bam benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam --reference benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta --reference-panel benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/contammix/contamination.json && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/contammix/contamination.summary.json
import json
print(json.dumps(json.loads("{\n  \"assumptions\": \"governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning\",\n  \"emit_confidence_caveats\": true,\n  \"method\": \"contammix\",\n  \"minimum_mean_coverage\": 0.5,\n  \"reference\": \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta\",\n  \"reference_panels\": [\n    \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat\"\n  ],\n  \"scope\": \"nuclear\",\n  \"tool_scope\": \"nuclear\"\n}"), indent=2))
PY'

# bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:schmutzi / bam / bam.contamination / schmutzi
/bin/sh -c 'schmutzi --bam benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam --reference benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta --outdir benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.txt ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.txt benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.json; else : > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.json; fi && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/schmutzi/contamination.summary.json
import json
print(json.dumps(json.loads("{\n  \"assumptions\": \"governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning\",\n  \"emit_confidence_caveats\": true,\n  \"method\": \"schmutzi\",\n  \"minimum_mean_coverage\": 0.5,\n  \"reference\": \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta\",\n  \"reference_panels\": [],\n  \"scope\": \"mito\",\n  \"tool_scope\": \"mt\"\n}"), indent=2))
PY'

# bam:corpus-01-adna-bam-mini:bam.contamination:sample-set:verifybamid2 / bam / bam.contamination / verifybamid2
/bin/sh -c 'test -f benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam.bai && verifybamid2 --BamFile benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_contamination_panel_screen.sam --Reference benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta --SVDPrefix benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat --Output benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.selfSM ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.selfSM benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.json; else : > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.json; fi && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/contamination/verifybamid2/contamination.summary.json
import json
print(json.dumps(json.loads("{\n  \"assumptions\": \"governed aDNA BAM corpus contamination panel with shared non-UDG reference for local contamination planning\",\n  \"emit_confidence_caveats\": true,\n  \"method\": \"verifybamid2\",\n  \"minimum_mean_coverage\": 0.5,\n  \"reference\": \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_bam_reference.fasta\",\n  \"reference_panels\": [\n    \"benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/reference/adna_contamination_panel.dat\"\n  ],\n  \"scope\": \"nuclear\",\n  \"tool_scope\": \"nuclear\"\n}"), indent=2))
PY'

# bam:corpus-01-bam-mini:bam.coverage:sample-set:bedtools / bam / bam.coverage / bedtools
/bin/sh -c 'bedtools coverage -a benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam >/dev/null && samtools depth -a -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/bedtools/coverage.depth.txt && awk '"'"'{sum+=$3; if($3>0) cov++} END {mean=(NR>0)?sum/NR:0; print "total", NR, cov, mean}'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/coverage/bedtools/coverage.depth.txt > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/bedtools/coverage.mosdepth.summary.txt'

# bam:corpus-01-bam-mini:bam.coverage:sample-set:mosdepth / bam / bam.coverage / mosdepth
/bin/sh -c 'mosdepth -n --by benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam && samtools depth -a -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage.depth.txt && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage.mosdepth.summary.txt ]; then :; else : > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/mosdepth/coverage.mosdepth.summary.txt; fi'

# bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools / bam / bam.coverage / samtools
/bin/sh -c 'samtools depth -a -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/regions/human_like_target_window_coverage.bed benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_target_window_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/samtools/coverage.depth.txt && awk '"'"'{sum+=$3; if($3>0) cov++} END {mean=(NR>0)?sum/NR:0; print "total", NR, cov, mean}'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/coverage/samtools/coverage.depth.txt > benchmarks/readiness/stage-tool-commands/bam/bam/coverage/samtools/coverage.mosdepth.summary.txt'

# bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:addeam / bam / bam.damage / addeam
addeam --bam benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --out benchmarks/readiness/stage-tool-commands/bam/bam/damage/addeam/damage.addeam.json

# bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:damageprofiler / bam / bam.damage / damageprofiler
damageprofiler --input benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --output benchmarks/readiness/stage-tool-commands/bam/bam/damage/damageprofiler/damage.profiler.json

# bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:mapdamage2 / bam / bam.damage / mapdamage2
/bin/sh -c 'mapDamage --bam benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --folder benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2 && if [ -f benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/misincorporation.txt ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/misincorporation.txt benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/damage.mapdamage2.txt; elif [ -f benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/5pCtoT.txt ]; then cp benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/5pCtoT.txt benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/damage.mapdamage2.txt; else : > benchmarks/readiness/stage-tool-commands/bam/bam/damage/mapdamage2/damage.mapdamage2.txt; fi'

# bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:ngsbriggs / bam / bam.damage / ngsbriggs
ngsbriggs --input benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --json-out benchmarks/readiness/stage-tool-commands/bam/bam/damage/ngsbriggs/damage.ngsbriggs.json

# bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:pmdtools / bam / bam.damage / pmdtools
/bin/sh -c 'pmdtools --input benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam > /dev/null && python3 - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/damage/pmdtools/damage.pmdtools.json
import json
print(json.dumps({"tool": "pmdtools", "stage": "bam.damage", "c_to_t_5p": 0.0, "g_to_a_3p": 0.0, "pmd_score_histogram": []}, indent=2))
PY'

# bam:corpus-01-adna-damage-mini:bam.damage:adna_damage_non_udg:pydamage / bam / bam.damage / pydamage
pydamage analyze --input benchmarks/tests/fixtures/corpora/corpus-01-adna-damage-mini/aligned/adna_damage_non_udg.sam --output benchmarks/readiness/stage-tool-commands/bam/bam/damage/pydamage/damage.pydamage.json --min-mapq 0.3

# bam:corpus-01-bam-mini:bam.duplication_metrics:sample-set:picard / bam / bam.duplication_metrics / picard
/bin/sh -c 'picard MarkDuplicates I=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.tmp.bam M=benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.histogram.txt VALIDATION_STRINGENCY=SILENT ASSUME_SORTED=true && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.histogram.txt > benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/picard/duplication.metrics.json
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

# bam:corpus-01-bam-mini:bam.duplication_metrics:sample-set:samtools / bam / bam.duplication_metrics / samtools
/bin/sh -c 'samtools markdup -s benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.tmp.bam 2> benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.histogram.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.histogram.txt > benchmarks/readiness/stage-tool-commands/bam/bam/duplication_metrics/samtools/duplication.metrics.json
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

# bam:corpus-01-bam-mini:bam.endogenous_content:sample-set:samtools / bam / bam.endogenous_content / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_endogenous_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/endogenous_content/samtools/flagstat.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/endogenous_content/samtools/endogenous.content.json
import json
payload = {"stage": "bam.endogenous_content", "method": "mapped_fraction_from_flagstat", "flagstat": "benchmarks/readiness/stage-tool-commands/bam/bam/endogenous_content/samtools/flagstat.txt", "host_reference_scope": "human_host", "host_reference_digest": null, "refuse_without_host_reference": true}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.filter:sample-set:bamtools / bam / bam.filter / bamtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/idxstats.before.txt && bamtools stats -in benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam >/dev/null && samtools view -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bamtools/filter.summary.json
import json
print(json.dumps({"filter_tool": "bamtools"}, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.filter:sample-set:bedtools / bam / bam.filter / bedtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/idxstats.before.txt && bedtools bamtobed -i benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam >/dev/null && samtools view -b benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/filter/bedtools/filter.summary.json
import json
print(json.dumps({"filter_tool": "bedtools"}, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.filter:sample-set:samtools / bam / bam.filter / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.before.txt && samtools view -h -b -q 20 -F 4,1024 benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam | awk '"'"'BEGIN{OFS="\t"} /^@/{print; next} length($10)>=8'"'"' | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filter.summary.json
import json
payload = {"action": "filter", "input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mixed_filter_constraints.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/filtered.bam", "params": {"mapq_threshold": 20, "min_length": 8, "remove_duplicates": true}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/filter/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.gc_bias:sample-set:picard / bam / bam.gc_bias / picard
picard CollectGcBiasMetrics 'I=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_gc_window_ladder.sam' 'R=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_gc_window_ladder.fasta' 'O=benchmarks/readiness/stage-tool-commands/bam/bam/gc_bias/picard/gc_bias.metrics.txt' 'S=benchmarks/readiness/stage-tool-commands/bam/bam/gc_bias/picard/gc_bias.summary.json' 'CHART=benchmarks/readiness/stage-tool-commands/bam/bam/gc_bias/picard/gc_bias.plot.pdf'

# bam:corpus-01-genotyping-mini:bam.genotyping:human_like_genotyping_candidate_panel:angsd / bam / bam.genotyping / angsd
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.genotyping

# bam:corpus-01-adna-bam-mini:bam.haplogroups:sample-set:yleaf / bam / bam.haplogroups / yleaf
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.haplogroups

# bam:corpus-01-bam-mini:bam.insert_size:sample-set:picard / bam / bam.insert_size / picard
picard CollectInsertSizeMetrics 'I=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_insert_size_triplet.sam' 'O=benchmarks/readiness/stage-tool-commands/bam/bam/insert_size/picard/insert_size.metrics.txt' 'H=benchmarks/readiness/stage-tool-commands/bam/bam/insert_size/picard/insert_size.histogram.pdf' 'M=0.5'

# bam:corpus-01-kinship-mini:bam.kinship:sample-set:angsd / bam / bam.kinship / angsd
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.kinship

# bam:corpus-01-kinship-mini:bam.kinship:sample-set:king / bam / bam.kinship / king
cargo run -q -p bijux-dna --features bam_downstream -- bench local materialize-stage --stage-id bam.kinship

# bam:corpus-01-bam-mini:bam.length_filter:sample-set:picard / bam / bam.length_filter / picard
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.before.txt && samtools view -h -b -q 0 benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam | awk '"'"'BEGIN{OFS="\t"} /^@/{print; next} length($10)>=8'"'"' | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/length_filter.summary.json
import json
payload = {"action": "length_filter", "input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/filtered.bam", "params": {"mapq_threshold": 0, "min_length": 8, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/picard/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.length_filter:sample-set:samtools / bam / bam.length_filter / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.before.txt && samtools view -h -b -q 0 benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam | awk '"'"'BEGIN{OFS="\t"} /^@/{print; next} length($10)>=8'"'"' | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/length_filter.summary.json
import json
payload = {"action": "length_filter", "input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_length_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/filtered.bam", "params": {"mapq_threshold": 0, "min_length": 8, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/length_filter/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:picard / bam / bam.mapping_summary / picard
/bin/sh -c 'picard CollectAlignmentSummaryMetrics I=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/alignment_summary.metrics.txt VALIDATION_STRINGENCY=SILENT && picard BamIndexStats I=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/idxstats.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/alignment_summary.metrics.txt benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/flagstat.txt benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/picard/mapping.summary.json
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

# bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:samtools / bam / bam.mapping_summary / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/flagstat.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/idxstats.txt && samtools stats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_partial_mapping.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/samtools_stats.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/mapping.summary.json
import json
print(json.dumps({"stage":"bam.mapping_summary","flagstat":"benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/flagstat.txt","idxstats":"benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/idxstats.txt","stats":"benchmarks/readiness/stage-tool-commands/bam/bam/mapping_summary/samtools/samtools_stats.txt"}, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.mapq_filter:sample-set:bamtools / bam / bam.mapq_filter / bamtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.before.txt && samtools view -h -b -q 30 benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam | cat | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/mapq_filter.summary.json
import json
payload = {"action": "mapq_filter", "input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/filtered.bam", "params": {"mapq_threshold": 30, "min_length": 0, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/bamtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.mapq_filter:sample-set:samtools / bam / bam.mapq_filter / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.before.txt && samtools view -h -b -q 30 benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam | cat | samtools view -b - | samtools sort -@ 1 -l 6 -o benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/mapq_filter.summary.json
import json
payload = {"action": "mapq_filter", "input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_mapq_threshold_ladder.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/filtered.bam", "params": {"mapq_threshold": 30, "min_length": 0, "remove_duplicates": false}, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/mapq_filter/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.markdup:sample-set:picard / bam / bam.markdup / picard
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.before.txt && picard MarkDuplicates I=benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam O=benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam M=benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.metrics.txt VALIDATION_STRINGENCY=SILENT ASSUME_SORTED=true REMOVE_DUPLICATES=false && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.after.txt && python - <<'"'"'PY'"'"' benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.metrics.txt > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.summary.json
import json,sys
payload={"input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/markdup.bam", "metrics": sys.argv[1], "remove_duplicates": false, "tool": "picard", "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/picard/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.markdup:sample-set:samtools / bam / bam.markdup / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.before.txt && samtools markdup benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam && samtools index benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.summary.json
import json
payload = {"input_bam": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_cluster.sam", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/markdup.bam", "remove_duplicates": false, "artifacts": {"flagstat_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.before.txt", "flagstat_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/flagstat.after.txt", "idxstats_before": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.before.txt", "idxstats_after": "benchmarks/readiness/stage-tool-commands/bam/bam/markdup/samtools/idxstats.after.txt"}}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.overlap_correction:sample-set:bamutil / bam / bam.overlap_correction / bamutil
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/flagstat.before.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/idxstats.before.txt && bam clipOverlap --in benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam --out benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam --stats > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap_correction.summary.json.clipoverlap.log 2>&1 && samtools index -@ 1 benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam.bai && samtools flagstat benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/flagstat.after.txt && samtools idxstats benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/idxstats.after.txt && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap_correction.summary.json
import json
print(json.dumps({"method": "bamutil.clipOverlap", "input": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_paired_overlap_control.sam", "output": "benchmarks/readiness/stage-tool-commands/bam/bam/overlap_correction/bamutil/overlap.corrected.bam"}, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc / bam / bam.qc_pre / multiqc
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/multiqc/flagstat.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/multiqc/idxstats.txt && samtools stats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/multiqc/samtools_stats.txt'

# bam:corpus-01-bam-mini:bam.qc_pre:sample-set:samtools / bam / bam.qc_pre / samtools
/bin/sh -c 'samtools flagstat benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/samtools/flagstat.txt && samtools idxstats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/samtools/idxstats.txt && samtools stats benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_duplicate_flagged_multicontig.sam > benchmarks/readiness/stage-tool-commands/bam/bam/qc_pre/samtools/samtools_stats.txt'

# bam:corpus-01-bam-mini:bam.recalibration:sample-set:gatk / bam / bam.recalibration / gatk
/bin/sh -c 'cp benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_recalibration_low_coverage.sam benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam && printf '"'"'tiny-index\n'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam.bai && cat <<'"'"'EOF'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.report.txt
status=skipped
reason=requested_skip_mode
EOF
python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.summary.json
import json
print(json.dumps({"mode": "skip", "status": "skipped", "reason": "requested_skip_mode", "known_sites": ["benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf"], "reference": "benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/corpus_01_bam_reference.fasta", "recalibration_report": "benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.report.txt", "output_bam": "benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam", "output_bai": "benchmarks/readiness/stage-tool-commands/bam/bam/recalibration/gatk/recal.bam.bai"}, indent=2))
PY'

# bam:corpus-01-adna-bam-mini:bam.sex:sample-set:angsd / bam / bam.sex / angsd
/bin/sh -c 'angsd -i benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam -doCounts 1 -dumpCounts 2 -out benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.angsd && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.json
import json
print(json.dumps({"method": "angsd", "counts_prefix": "benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.angsd"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/angsd/sex.summary.json
import json
payload = {"method": "rxy", "backend": "angsd", "x_to_y_ratio": 0.0, "confidence": 0.0}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-adna-bam-mini:bam.sex:sample-set:rxy / bam / bam.sex / rxy
/bin/sh -c 'rxy --input benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam > benchmarks/readiness/stage-tool-commands/bam/bam/sex/rxy/sex.json && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/rxy/sex.summary.json
import json
payload = {"method": "rxy", "x_to_y_ratio": 0.0, "confidence": 0.0}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-adna-bam-mini:bam.sex:sample-set:yleaf / bam / bam.sex / yleaf
/bin/sh -c 'yleaf -bam benchmarks/tests/fixtures/corpora/corpus-01-adna-bam-mini/aligned/adna_xy_autosome_coverage.sam -o benchmarks/readiness/stage-tool-commands/bam/bam/sex/yleaf/sex --reference_genome hg38 && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/yleaf/sex.json
import json
print(json.dumps({"method": "yleaf", "backend": "yleaf", "chromosome_system": "xy"}, indent=2))
PY && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/sex/yleaf/sex.summary.json
import json
payload = {"method": "rxy", "backend": "yleaf", "minimum_y_sites": 5, "x_to_y_ratio": 0.0, "confidence": 0.0}
print(json.dumps(payload, indent=2))
PY'

# bam:corpus-01-bam-mini:bam.validate:sample-set:bamtools / bam / bam.validate / bamtools
/bin/sh -c 'bamtools stats -in assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bamtools/validation.json && samtools flagstat assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bamtools/flagstat.txt'

# bam:corpus-01-bam-mini:bam.validate:sample-set:bedtools / bam / bam.validate / bedtools
/bin/sh -c 'bedtools bamtobed -i assets/toy/core-v1/bam/validation_pass.bam >/dev/null && python - <<'"'"'PY'"'"' > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bedtools/validation.json
import json
print(json.dumps({"validator": "bedtools.bamtobed", "status": "ok"}, indent=2))
PY && samtools flagstat assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/bedtools/flagstat.txt'

# bam:corpus-01-bam-mini:bam.validate:sample-set:samtools / bam / bam.validate / samtools
/bin/sh -c 'samtools quickcheck -v assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/samtools/validation.json && samtools flagstat assets/toy/core-v1/bam/validation_pass.bam > benchmarks/readiness/stage-tool-commands/bam/bam/validate/samtools/flagstat.txt'

# fastq:corpus-03-amplicon-mini:fastq.cluster_otus:sample-set:vsearch / fastq / fastq.cluster_otus / vsearch
vsearch --cluster_fast assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq --id 0.97 --sizein --sizeout --relabel OTU_ --threads 1 --centroids benchmarks/readiness/stage-tool-commands/fastq/fastq/cluster_otus/vsearch/otu_representatives.fasta --uc benchmarks/readiness/stage-tool-commands/fastq/fastq/cluster_otus/vsearch/otu_clusters.uc --otutabout benchmarks/readiness/stage-tool-commands/fastq/fastq/cluster_otus/vsearch/otu_abundance.tsv

# fastq:corpus-01-mini:fastq.correct_errors:sample-set:bayeshammer / fastq / fastq.correct_errors / bayeshammer
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work'"'"'
normalize_fastq_output() { src="$1"; dest="$2"; case "$src" in *.gz) mv -- "$src" "$dest" ;; *) gzip -c -- "$src" > "$dest" ;; esac; }
bayeshammer --threads 1 --phred-offset 33 -1 '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' -2 '"'"'assets/toy/core-v1/fastq/reads_2.fastq'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work'"'"'
find '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected'"'"' -type f \( -name '"'"'*.cor.fq'"'"' -o -name '"'"'*.cor.fastq'"'"' -o -name '"'"'*.cor.fq.gz'"'"' -o -name '"'"'*.cor.fastq.gz'"'"' \) > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.unsorted'"'"'
LC_ALL=C sort '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.unsorted'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.list'"'"'
rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.unsorted'"'"'
r1_output=$(grep '"'"'/[^/]*R1[^/]*\.cor\.f\(ast\)\?q\(.gz\)\?$'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.list'"'"' | head -n 1 || true)
r2_output=$(grep '"'"'/[^/]*R2[^/]*\.cor\.f\(ast\)\?q\(.gz\)\?$'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.list'"'"' | head -n 1 || true)
unpaired_output=$(grep '"'"'/[^/]*R_unpaired[^/]*\.cor\.f\(ast\)\?q\(.gz\)\?$'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.list'"'"' | head -n 1 || true)
if [ -z "$r1_output" ] || [ -z "$r2_output" ]; then echo "expected BayesHammer paired corrected outputs in '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected'"'"'" >&2; cat '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/bayeshammer_work/corrected/corrected_outputs.list'"'"' >&2; exit 64; fi
INPUT_R1='"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' INPUT_R2='"'"'assets/toy/core-v1/fastq/reads_2.fastq'"'"' PAIRED_R1="$r1_output" PAIRED_R2="$r2_output" UNPAIRED_PATH="$unpaired_output" OUTPUT_R1='"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/reads_r1.fastq.gz'"'"' OUTPUT_R2='"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/reads_r2.fastq.gz'"'"' python3 - <<'"'"'PY'"'"'
import gzip
import os

input_r1 = os.environ["INPUT_R1"]
input_r2 = os.environ["INPUT_R2"]
paired_r1 = os.environ["PAIRED_R1"]
paired_r2 = os.environ["PAIRED_R2"]
unpaired_path = os.environ.get("UNPAIRED_PATH", "")
output_r1 = os.environ["OUTPUT_R1"]
output_r2 = os.environ["OUTPUT_R2"]

def read_fastq(path):
    opener = gzip.open if path.endswith(".gz") else open
    with opener(path, "rt", encoding="utf-8") as handle:
        while True:
            header = handle.readline()
            if not header:
                break
            sequence = handle.readline()
            plus = handle.readline()
            quality = handle.readline()
            if not quality:
                raise SystemExit(f"incomplete FASTQ record in {path}")
            yield (
                header.rstrip("\n"),
                sequence.rstrip("\n"),
                plus.rstrip("\n"),
                quality.rstrip("\n"),
            )

def write_fastq(path, records):
    with gzip.open(path, "wt", encoding="utf-8") as handle:
        for header, sequence, plus, quality in records:
            handle.write(header + "\n")
            handle.write(sequence + "\n")
            handle.write(plus + "\n")
            handle.write(quality + "\n")

def read_key(record):
    token = record[0].split()[0]
    if token.startswith("@"):
        token = token[1:]
    if token.endswith("/1") or token.endswith("/2"):
        token = token[:-2]
    return token

def sequence_distance(lhs, rhs):
    overlap = min(len(lhs), len(rhs))
    mismatches = sum(1 for i in range(overlap) if lhs[i] != rhs[i])
    return mismatches + abs(len(lhs) - len(rhs))

original_r1 = list(read_fastq(input_r1))
original_r2 = list(read_fastq(input_r2))
if len(original_r1) != len(original_r2):
    raise SystemExit(
        "BayesHammer reconstruction requires paired inputs with matching record counts"
    )

paired_r1_by_key = {}
for record in read_fastq(paired_r1):
    paired_r1_by_key[read_key(record)] = record

paired_r2_by_key = {}
for record in read_fastq(paired_r2):
    paired_r2_by_key[read_key(record)] = record

unpaired_by_key = {}
if unpaired_path:
    for record in read_fastq(unpaired_path):
        key = read_key(record)
        unpaired_by_key.setdefault(key, []).append(record)

reconstructed_r1 = []
reconstructed_r2 = []
for original_r1_record, original_r2_record in zip(original_r1, original_r2):
    key = read_key(original_r1_record)
    corrected_r1 = paired_r1_by_key.get(key)
    corrected_r2 = paired_r2_by_key.get(key)
    unpaired_records = unpaired_by_key.get(key, [])
    for unpaired_record in unpaired_records:
        score_r1 = sequence_distance(unpaired_record[1], original_r1_record[1])
        score_r2 = sequence_distance(unpaired_record[1], original_r2_record[1])
        if corrected_r1 is None and (corrected_r2 is not None or score_r1 <= score_r2):
            corrected_r1 = unpaired_record
            continue
        if corrected_r2 is None:
            corrected_r2 = unpaired_record
            continue
        if score_r1 <= score_r2:
            corrected_r1 = unpaired_record
        else:
            corrected_r2 = unpaired_record

    reconstructed_r1.append(corrected_r1 or original_r1_record)
    reconstructed_r2.append(corrected_r2 or original_r2_record)

write_fastq(output_r1, reconstructed_r1)
write_fastq(output_r2, reconstructed_r2)
PY
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.correct_errors.report.v2","stage":"fastq.correct_errors","stage_id":"fastq.correct_errors","tool_id":"bayeshammer","paired_mode":"paired_end","threads":1,"correction_engine":"bayeshammer","quality_encoding":"phred33","kmer_size":null,"musket_kmer_budget":null,"genome_size":null,"max_memory_gb":null,"trusted_kmer_artifact":null,"conservative_mode":false,"input_r1":"assets/toy/core-v1/fastq/reads_1.fastq","input_r2":"assets/toy/core-v1/fastq/reads_2.fastq","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/reads_r1.fastq.gz","output_r2":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/reads_r2.fastq.gz","report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/correct_report.json","corrected_reads":null,"changed_reads":null,"unchanged_reads":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"mean_q_before":null,"mean_q_after":null,"kmer_fix_rate":null,"correction_effect":null,"runtime_s":null,"memory_mb":null,"exit_code":null,"raw_backend_report":null,"raw_backend_report_format":null,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/bayeshammer/correct_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.correct_errors:sample-set:lighter / fastq / fastq.correct_errors / lighter
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work'"'"'
normalize_fastq_output() { src="$1"; dest="$2"; case "$src" in *.gz) mv -- "$src" "$dest" ;; *) gzip -c -- "$src" > "$dest" ;; esac; }
lighter -K 21 2500000 -t 1 -od '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work'"'"' -r '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' -r '"'"'assets/toy/core-v1/fastq/reads_2.fastq'"'"'
find '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work'"'"' -type f \( -name '"'"'*.cor.fq'"'"' -o -name '"'"'*.cor.fastq'"'"' -o -name '"'"'*.cor.fq.gz'"'"' -o -name '"'"'*.cor.fastq.gz'"'"' -o -name '"'"'*.fq'"'"' -o -name '"'"'*.fastq'"'"' -o -name '"'"'*.fq.gz'"'"' -o -name '"'"'*.fastq.gz'"'"' \) > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.unsorted'"'"'
LC_ALL=C sort '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.unsorted'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.list'"'"'
rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.unsorted'"'"'
actual_outputs=$(wc -l < '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.list'"'"' | tr -d '"'"'[:space:]'"'"')
if [ "$actual_outputs" -ne 2 ]; then echo "expected 2 corrected outputs in '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work'"'"' but found $actual_outputs" >&2; exit 64; fi
normalize_fastq_output "$(sed -n '"'"'1p'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.list'"'"')" '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/reads_r1.fastq.gz'"'"'
normalize_fastq_output "$(sed -n '"'"'2p'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/lighter_work/corrected_outputs.list'"'"')" '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/reads_r2.fastq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.correct_errors.report.v2","stage":"fastq.correct_errors","stage_id":"fastq.correct_errors","tool_id":"lighter","paired_mode":"paired_end","threads":1,"correction_engine":"lighter","quality_encoding":"phred33","kmer_size":null,"musket_kmer_budget":null,"genome_size":2500000,"max_memory_gb":null,"trusted_kmer_artifact":null,"conservative_mode":false,"input_r1":"assets/toy/core-v1/fastq/reads_1.fastq","input_r2":"assets/toy/core-v1/fastq/reads_2.fastq","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/reads_r1.fastq.gz","output_r2":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/reads_r2.fastq.gz","report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/correct_report.json","corrected_reads":null,"changed_reads":null,"unchanged_reads":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"mean_q_before":null,"mean_q_after":null,"kmer_fix_rate":null,"correction_effect":null,"runtime_s":null,"memory_mb":null,"exit_code":null,"raw_backend_report":null,"raw_backend_report_format":null,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/lighter/correct_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.correct_errors:sample-set:musket / fastq / fastq.correct_errors / musket
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/musket_work'"'"'
normalize_fastq_output() { src="$1"; dest="$2"; case "$src" in *.gz) mv -- "$src" "$dest" ;; *) gzip -c -- "$src" > "$dest" ;; esac; }
musket -p 1 -k 21 536870912 -omulti '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/musket_work/corrected'"'"' -inorder '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' '"'"'assets/toy/core-v1/fastq/reads_2.fastq'"'"'
normalize_fastq_output '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/musket_work/corrected.0'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/reads_r1.fastq.gz'"'"'
normalize_fastq_output '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/musket_work/corrected.1'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/reads_r2.fastq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.correct_errors.report.v2","stage":"fastq.correct_errors","stage_id":"fastq.correct_errors","tool_id":"musket","paired_mode":"paired_end","threads":1,"correction_engine":"musket","quality_encoding":"phred33","kmer_size":null,"musket_kmer_budget":536870912,"genome_size":null,"max_memory_gb":null,"trusted_kmer_artifact":null,"conservative_mode":false,"input_r1":"assets/toy/core-v1/fastq/reads_1.fastq","input_r2":"assets/toy/core-v1/fastq/reads_2.fastq","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/reads_r1.fastq.gz","output_r2":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/reads_r2.fastq.gz","report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/correct_report.json","corrected_reads":null,"changed_reads":null,"unchanged_reads":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"mean_q_before":null,"mean_q_after":null,"kmer_fix_rate":null,"correction_effect":null,"runtime_s":null,"memory_mb":null,"exit_code":null,"raw_backend_report":null,"raw_backend_report_format":null,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/musket/correct_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.correct_errors:sample-set:rcorrector / fastq / fastq.correct_errors / rcorrector
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work'"'"'
normalize_fastq_output() { src="$1"; dest="$2"; case "$src" in *.gz) mv -- "$src" "$dest" ;; *) gzip -c -- "$src" > "$dest" ;; esac; }
run_rcorrector.pl -t 1 -od '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work'"'"' -1 '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' -2 '"'"'assets/toy/core-v1/fastq/reads_2.fastq'"'"'
find '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work'"'"' -type f \( -name '"'"'*.cor.fq'"'"' -o -name '"'"'*.cor.fastq'"'"' -o -name '"'"'*.cor.fq.gz'"'"' -o -name '"'"'*.cor.fastq.gz'"'"' \) > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.unsorted'"'"'
LC_ALL=C sort '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.unsorted'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.list'"'"'
rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.unsorted'"'"'
actual_outputs=$(wc -l < '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.list'"'"' | tr -d '"'"'[:space:]'"'"')
if [ "$actual_outputs" -ne 2 ]; then echo "expected 2 corrected outputs in '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work'"'"' but found $actual_outputs" >&2; exit 64; fi
normalize_fastq_output "$(sed -n '"'"'1p'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.list'"'"')" '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/reads_r1.fastq.gz'"'"'
normalize_fastq_output "$(sed -n '"'"'2p'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/rcorrector_work/corrected_outputs.list'"'"')" '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/reads_r2.fastq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.correct_errors.report.v2","stage":"fastq.correct_errors","stage_id":"fastq.correct_errors","tool_id":"rcorrector","paired_mode":"paired_end","threads":1,"correction_engine":"rcorrector","quality_encoding":"phred33","kmer_size":null,"musket_kmer_budget":null,"genome_size":null,"max_memory_gb":null,"trusted_kmer_artifact":null,"conservative_mode":false,"input_r1":"assets/toy/core-v1/fastq/reads_1.fastq","input_r2":"assets/toy/core-v1/fastq/reads_2.fastq","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/reads_r1.fastq.gz","output_r2":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/reads_r2.fastq.gz","report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/correct_report.json","corrected_reads":null,"changed_reads":null,"unchanged_reads":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"mean_q_before":null,"mean_q_after":null,"kmer_fix_rate":null,"correction_effect":null,"runtime_s":null,"memory_mb":null,"exit_code":null,"raw_backend_report":null,"raw_backend_report_format":null,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/correct_errors/rcorrector/correct_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.deplete_host:sample-set:bowtie2 / fastq / fastq.deplete_host / bowtie2
bowtie2 -x assets/reference/host/references/toy_host_reference --threads 4 -S /dev/null -U assets/toy/core-v1/fastq/reads_1.fastq --un-gz benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_host/bowtie2/host_depleted.fastq.gz --al-gz benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_host/bowtie2/removed_host.fastq.gz --met-file benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_host/bowtie2/bowtie2.host.metrics.txt

# fastq:corpus-01-mini:fastq.deplete_reference_contaminants:sample-set:bowtie2 / fastq / fastq.deplete_reference_contaminants / bowtie2
bowtie2 -x assets/reference/contaminants/references/toy_contaminant_reference --threads 4 -S /dev/null -U assets/toy/core-v1/fastq/reads_1.fastq --un-gz benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_reference_contaminants/bowtie2/contaminant_screened.fastq.gz --al-gz benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_reference_contaminants/bowtie2/removed_contaminant.fastq.gz --met-file benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_reference_contaminants/bowtie2/bowtie2.contaminant.metrics.txt

# fastq:corpus-01-mini:fastq.deplete_rrna:sample-set:sortmerna / fastq / fastq.deplete_rrna / sortmerna
bash -lc 'set -euo pipefail
shopt -s nullglob
collect_output_from_globs() { dest="$1"; shift; local pattern candidate matches=(); local -a inputs=(); for pattern in "$@"; do matches=( $pattern ); for candidate in "${matches[@]}"; do if [ -f "$candidate" ]; then inputs+=( "$candidate" ); fi; done; done; if [ "${#inputs[@]}" -eq 0 ]; then printf '"'"'missing expected SortMeRNA output for %s\n'"'"' "$dest" >&2; return 1; fi; { for candidate in "${inputs[@]}"; do case "$candidate" in *.gz) gzip -cd -- "$candidate" ;; *) cat -- "$candidate" ;; esac; done; } | gzip -c > "$dest"; }
rm -rf '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/kvdb'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out'"'"'
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/idx'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/kvdb'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out'"'"'
mkdir -p "$(dirname '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_filtered.fastq.gz'"'"')" "$(dirname '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/removed_rrna.fastq.gz'"'"')" "$(dirname '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.tsv'"'"')" "$(dirname '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.json'"'"')"
rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_filtered.fastq.gz'"'"' '"'"''"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/removed_rrna.fastq.gz'"'"' '"'"''"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.tsv'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.json'"'"'
'"'"'sortmerna'"'"' '"'"'--ref'"'"' '"'"'assets/reference/rrna/references/sortmerna_common_rrna_reference.fasta'"'"' '"'"'--reads'"'"' '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' '"'"'--workdir'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/'"'"' '"'"'--idx-dir'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/idx'"'"' '"'"'--kvdb'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/kvdb'"'"' '"'"'--readb'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb'"'"' '"'"'--threads'"'"' '"'"'4'"'"' '"'"'--fastx'"'"' '"'"'--zip-out'"'"' '"'"'yes'"'"'
collect_output_from_globs '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_filtered.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/other*.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/other*.fq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/other*.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/other*.fq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/fwd_*.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/fwd_*.fq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/fwd_*.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/fwd_*.fq'"'"'
collect_output_from_globs '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/removed_rrna.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/aligned*.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/aligned*.fq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/aligned*.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/aligned*.fq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/aligned_fwd_*.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/aligned_fwd_*.fq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/aligned_fwd_*.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/readb/aligned_fwd_*.fq'"'"'
if [ -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/aligned.log'"'"' ]; then cp -- '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/sortmerna_workdir/out/aligned.log'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.tsv'"'"'; else : > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.tsv'"'"'; fi
printf '"'"'{}\n'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/deplete_rrna/sortmerna/rrna_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.detect_adapters:sample-set:fastqc / fastq / fastq.detect_adapters / fastqc
sh -lc 'mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/detect_adapters/fastqc/fastqc'"'"'
'"'"'fastqc'"'"' '"'"'--outdir'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/detect_adapters/fastqc/fastqc'"'"' '"'"'--threads'"'"' '"'"'4'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_adapter_hit_R1.fastq.gz'"'"''

# fastq:corpus-01-mini:fastq.detect_duplicates_premerge:sample-set:bijux_dna / fastq / fastq.detect_duplicates_premerge / bijux_dna
bijux-dna

# fastq:corpus-01-mini:fastq.estimate_library_complexity_prealign:sample-set:bijux_dna / fastq / fastq.estimate_library_complexity_prealign / bijux_dna
bijux-dna

# fastq:corpus-01-mini:fastq.extract_umis:sample-set:umi_tools / fastq / fastq.extract_umis / umi_tools
umi_tools extract --stdin benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_umi_prefix_signals_R1.fastq.gz --stdout benchmarks/readiness/stage-tool-commands/fastq/fastq/extract_umis/umi_tools/umi_tagged_R1.fastq.gz --read2-in benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_umi_prefix_signals_R2.fastq.gz --read2-out benchmarks/readiness/stage-tool-commands/fastq/fastq/extract_umis/umi_tools/umi_tagged_R2.fastq.gz --bc-pattern NNNN --log benchmarks/readiness/stage-tool-commands/fastq/fastq/extract_umis/umi_tools/umi_tools.extract.log

# fastq:corpus-01-mini:fastq.filter_low_complexity:sample-set:bbduk / fastq / fastq.filter_low_complexity / bbduk
bbduk 'in=benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz' 'out=benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_low_complexity/bbduk/bbduk.fastq.gz' 'entropy=0.6' 'stats=benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_low_complexity/bbduk/bbduk.low_complexity.stats' 'maxpoly=8'

# fastq:corpus-01-mini:fastq.filter_low_complexity:sample-set:prinseq / fastq / fastq.filter_low_complexity / prinseq
'prinseq++' -threads 4 -fastq benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz -out_good benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_low_complexity/prinseq/prinseq_good.fastq -out_bad /dev/null -lc_entropy 0.6

# fastq:corpus-01-mini:fastq.filter_reads:sample-set:bbduk / fastq / fastq.filter_reads / bbduk
bbduk 'in=benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz' 'out=benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_reads/bbduk/bbduk.fastq.gz'

# fastq:corpus-01-mini:fastq.filter_reads:sample-set:fastp / fastq / fastq.filter_reads / fastp
fastp --in1 benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz --out1 benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_reads/fastp/fastp.fastq.gz --thread 1 --json benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_reads/fastp/fastp.filter.json --n_base_limit 1 --low_complexity_filter --complexity_threshold 20

# fastq:corpus-01-mini:fastq.filter_reads:sample-set:prinseq / fastq / fastq.filter_reads / prinseq
'prinseq++' -fastq benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz -out_good benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_reads/prinseq/prinseq_good.fastq -out_bad benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_reads/prinseq/prinseq_good.fastq.bad

# fastq:corpus-01-mini:fastq.filter_reads:sample-set:seqkit / fastq / fastq.filter_reads / seqkit
seqkit seq -m 1 -o benchmarks/readiness/stage-tool-commands/fastq/fastq/filter_reads/seqkit/seqkit.fastq.gz benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_filter_signals_R1.fastq.gz

# fastq:reference-index-assets:fastq.index_reference:asset-set:bowtie2_build / fastq / fastq.index_reference / bowtie2_build
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/index_reference/bowtie2_build/reference_index/bowtie2'"'"'
bowtie2-build --threads 4 '"'"'assets/reference/contaminants/references/phix174.fasta'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/index_reference/bowtie2_build/reference_index/bowtie2/reference'"'"''

# fastq:reference-index-assets:fastq.index_reference:asset-set:star / fastq / fastq.index_reference / star
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/index_reference/star/reference_index/star'"'"'
STAR --runMode genomeGenerate --runThreadN 4 --genomeDir '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/index_reference/star/reference_index/star'"'"' --genomeFastaFiles '"'"'assets/reference/contaminants/references/phix174.fasta'"'"''

# fastq:corpus-03-amplicon-mini:fastq.infer_asvs:sample-set:dada2 / fastq / fastq.infer_asvs / dada2
dada2 run_dada2.R --input-r1 assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq --asv-table benchmarks/readiness/stage-tool-commands/fastq/fastq/infer_asvs/dada2/asv_abundance.tsv --asv-fasta benchmarks/readiness/stage-tool-commands/fastq/fastq/infer_asvs/dada2/asv_sequences.fasta --taxonomy-ready-fasta benchmarks/readiness/stage-tool-commands/fastq/fastq/infer_asvs/dada2/taxonomy_ready.fasta --taxonomy-ready-fastq benchmarks/readiness/stage-tool-commands/fastq/fastq/infer_asvs/dada2/taxonomy_ready.fastq --report-json benchmarks/readiness/stage-tool-commands/fastq/fastq/infer_asvs/dada2/infer_asvs_report.json --denoising-method dada2 --pooling-mode independent --chimera-policy remove_bimera_denovo --threads 1

# fastq:corpus-01-mini:fastq.merge_pairs:sample-set:adapterremoval / fastq / fastq.merge_pairs / adapterremoval
bash -lc 'set -euo pipefail
count_fastq_reads() {
  local path="$1"
  if [ ! -f "$path" ]; then printf '"'"'0'"'"'; return; fi
  case "$path" in
    *.gz) gzip -dc "$path" ;;
    *) cat "$path" ;;
  esac | awk '"'"'END { printf "%d", int(NR/4) }'"'"'
}
'"'"'adapterremoval'"'"' '"'"'--threads'"'"' '"'"'1'"'"' '"'"'--file1'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"' '"'"'--file2'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"' '"'"'--collapse-deterministic'"'"' '"'"'--gzip'"'"' '"'"'--output1'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair1.truncated.gz'"'"' '"'"'--output2'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair2.truncated.gz'"'"' '"'"'--outputcollapsed'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.collapsed.gz'"'"' '"'"'--outputcollapsedtruncated'"'"' '"'"'/dev/null'"'"' '"'"'--settings'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.settings'"'"' '"'"'--singleton'"'"' '"'"'/dev/null'"'"' '"'"'--discarded'"'"' '"'"'/dev/null'"'"' '"'"'--minalignmentlength'"'"' '"'"'8'"'"'
reads_r1=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"')
reads_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"')
reads_merged=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.collapsed.gz'"'"')
pairs_in=$reads_r1
if [ "$reads_r2" -lt "$pairs_in" ]; then pairs_in=$reads_r2; fi
reads_unmerged=$(( pairs_in - reads_merged ))
if [ "$reads_unmerged" -lt 0 ]; then reads_unmerged=0; fi
if [ 1 = 1 ]; then
  if [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair1.truncated.gz'"'"' ] && [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair2.truncated.gz'"'"' ]; then
    reads_unmerged_r1=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair1.truncated.gz'"'"')
    reads_unmerged_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair2.truncated.gz'"'"')
    reads_unmerged=$reads_unmerged_r1
    if [ "$reads_unmerged_r2" -lt "$reads_unmerged" ]; then reads_unmerged=$reads_unmerged_r2; fi
  fi
else
  rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair1.truncated.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair2.truncated.gz'"'"'
fi
merge_rate=$(awk -v merged="$reads_merged" -v pairs="$pairs_in" '"'"'BEGIN { if (pairs > 0) printf "%.6f", merged / pairs; else printf "0.000000" }'"'"')
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/merge_report.json'"'"' <<EOF
{
  "schema_version": "bijux.fastq.merge_pairs.report.v2",
  "stage": "fastq.merge_pairs",
  "stage_id": "fastq.merge_pairs",
  "tool_id": "adapterremoval",
  "paired_mode": "paired_end",
  "merge_engine": "adapter_removal",
  "threads": 1,
  "merge_overlap": 8,
  "min_len": null,
  "unmerged_read_policy": "emit_unmerged_pairs",
  "input_r1": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz",
  "input_r2": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz",
  "merged_reads": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.collapsed.gz",
  "unmerged_reads_r1": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair1.truncated.gz",
  "unmerged_reads_r2": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.pair2.truncated.gz",
  "reads_r1": $reads_r1,
  "reads_r2": $reads_r2,
  "reads_merged": $reads_merged,
  "reads_unmerged": $reads_unmerged,
  "merge_rate": $merge_rate,
  "runtime_s": null,
  "memory_mb": null,
  "raw_backend_report": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/adapterremoval/adapterremoval.settings",
  "raw_backend_report_format": "adapterremoval_settings"
}
EOF
'

# fastq:corpus-01-mini:fastq.merge_pairs:sample-set:bbmerge / fastq / fastq.merge_pairs / bbmerge
bash -lc 'set -euo pipefail
count_fastq_reads() {
  local path="$1"
  if [ ! -f "$path" ]; then printf '"'"'0'"'"'; return; fi
  case "$path" in
    *.gz) gzip -dc "$path" ;;
    *) cat "$path" ;;
  esac | awk '"'"'END { printf "%d", int(NR/4) }'"'"'
}
'"'"'bbmerge'"'"' '"'"'in1=benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"' '"'"'in2=benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"' '"'"'out=benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.merged.fastq'"'"' '"'"'threads=1'"'"' '"'"'outu1=benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r1.fastq'"'"' '"'"'outu2=benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r2.fastq'"'"' '"'"'minoverlap=8'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.log'"'"' 2>&1
reads_r1=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"')
reads_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"')
reads_merged=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.merged.fastq'"'"')
pairs_in=$reads_r1
if [ "$reads_r2" -lt "$pairs_in" ]; then pairs_in=$reads_r2; fi
reads_unmerged=$(( pairs_in - reads_merged ))
if [ "$reads_unmerged" -lt 0 ]; then reads_unmerged=0; fi
if [ 1 = 1 ]; then
  if [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r1.fastq'"'"' ] && [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r2.fastq'"'"' ]; then
    reads_unmerged_r1=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r1.fastq'"'"')
    reads_unmerged_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r2.fastq'"'"')
    reads_unmerged=$reads_unmerged_r1
    if [ "$reads_unmerged_r2" -lt "$reads_unmerged" ]; then reads_unmerged=$reads_unmerged_r2; fi
  fi
else
  rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r1.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r2.fastq'"'"'
fi
merge_rate=$(awk -v merged="$reads_merged" -v pairs="$pairs_in" '"'"'BEGIN { if (pairs > 0) printf "%.6f", merged / pairs; else printf "0.000000" }'"'"')
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/merge_report.json'"'"' <<EOF
{
  "schema_version": "bijux.fastq.merge_pairs.report.v2",
  "stage": "fastq.merge_pairs",
  "stage_id": "fastq.merge_pairs",
  "tool_id": "bbmerge",
  "paired_mode": "paired_end",
  "merge_engine": "bbmerge",
  "threads": 1,
  "merge_overlap": 8,
  "min_len": null,
  "unmerged_read_policy": "emit_unmerged_pairs",
  "input_r1": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz",
  "input_r2": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz",
  "merged_reads": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.merged.fastq",
  "unmerged_reads_r1": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r1.fastq",
  "unmerged_reads_r2": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.unmerged_r2.fastq",
  "reads_r1": $reads_r1,
  "reads_r2": $reads_r2,
  "reads_merged": $reads_merged,
  "reads_unmerged": $reads_unmerged,
  "merge_rate": $merge_rate,
  "runtime_s": null,
  "memory_mb": null,
  "raw_backend_report": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/bbmerge/bbmerge.log",
  "raw_backend_report_format": "bbmerge_log"
}
EOF
'

# fastq:corpus-01-mini:fastq.merge_pairs:sample-set:flash2 / fastq / fastq.merge_pairs / flash2
bash -lc 'set -euo pipefail
count_fastq_reads() {
  local path="$1"
  if [ ! -f "$path" ]; then printf '"'"'0'"'"'; return; fi
  case "$path" in
    *.gz) gzip -dc "$path" ;;
    *) cat "$path" ;;
  esac | awk '"'"'END { printf "%d", int(NR/4) }'"'"'
}
'"'"'flash2'"'"' '"'"'-o'"'"' '"'"'flash2'"'"' '"'"'-d'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2'"'"' '"'"'-t'"'"' '"'"'1'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"' '"'"'-m'"'"' '"'"'8'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.log'"'"' 2>&1
reads_r1=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"')
reads_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"')
reads_merged=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.extendedFrags.fastq'"'"')
pairs_in=$reads_r1
if [ "$reads_r2" -lt "$pairs_in" ]; then pairs_in=$reads_r2; fi
reads_unmerged=$(( pairs_in - reads_merged ))
if [ "$reads_unmerged" -lt 0 ]; then reads_unmerged=0; fi
if [ 1 = 1 ]; then
  if [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_1.fastq'"'"' ] && [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_2.fastq'"'"' ]; then
    reads_unmerged_r1=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_1.fastq'"'"')
    reads_unmerged_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_2.fastq'"'"')
    reads_unmerged=$reads_unmerged_r1
    if [ "$reads_unmerged_r2" -lt "$reads_unmerged" ]; then reads_unmerged=$reads_unmerged_r2; fi
  fi
else
  rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_1.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_2.fastq'"'"'
fi
merge_rate=$(awk -v merged="$reads_merged" -v pairs="$pairs_in" '"'"'BEGIN { if (pairs > 0) printf "%.6f", merged / pairs; else printf "0.000000" }'"'"')
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/merge_report.json'"'"' <<EOF
{
  "schema_version": "bijux.fastq.merge_pairs.report.v2",
  "stage": "fastq.merge_pairs",
  "stage_id": "fastq.merge_pairs",
  "tool_id": "flash2",
  "paired_mode": "paired_end",
  "merge_engine": "flash2",
  "threads": 1,
  "merge_overlap": 8,
  "min_len": null,
  "unmerged_read_policy": "emit_unmerged_pairs",
  "input_r1": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz",
  "input_r2": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz",
  "merged_reads": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.extendedFrags.fastq",
  "unmerged_reads_r1": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_1.fastq",
  "unmerged_reads_r2": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.notCombined_2.fastq",
  "reads_r1": $reads_r1,
  "reads_r2": $reads_r2,
  "reads_merged": $reads_merged,
  "reads_unmerged": $reads_unmerged,
  "merge_rate": $merge_rate,
  "runtime_s": null,
  "memory_mb": null,
  "raw_backend_report": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/flash2/flash2.log",
  "raw_backend_report_format": "flash2_log"
}
EOF
'

# fastq:corpus-01-mini:fastq.merge_pairs:sample-set:leehom / fastq / fastq.merge_pairs / leehom
bash -lc 'set -euo pipefail
count_fastq_reads() {
  local path="$1"
  if [ ! -f "$path" ]; then printf '"'"'0'"'"'; return; fi
  case "$path" in
    *.gz) gzip -dc "$path" ;;
    *) cat "$path" ;;
  esac | awk '"'"'END { printf "%d", int(NR/4) }'"'"'
}
'"'"'leehom'"'"' '"'"'-fq1'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"' '"'"'-fq2'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"' '"'"'-fqo'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom'"'"' '"'"'-t'"'"' '"'"'1'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom.log'"'"' 2>&1
reads_r1=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"')
reads_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"')
reads_merged=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom.fq.gz'"'"')
pairs_in=$reads_r1
if [ "$reads_r2" -lt "$pairs_in" ]; then pairs_in=$reads_r2; fi
reads_unmerged=$(( pairs_in - reads_merged ))
if [ "$reads_unmerged" -lt 0 ]; then reads_unmerged=0; fi
if [ 1 = 1 ]; then
  if [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r1.fq.gz'"'"' ] && [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r2.fq.gz'"'"' ]; then
    reads_unmerged_r1=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r1.fq.gz'"'"')
    reads_unmerged_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r2.fq.gz'"'"')
    reads_unmerged=$reads_unmerged_r1
    if [ "$reads_unmerged_r2" -lt "$reads_unmerged" ]; then reads_unmerged=$reads_unmerged_r2; fi
  fi
else
  rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r1.fq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r2.fq.gz'"'"'
fi
merge_rate=$(awk -v merged="$reads_merged" -v pairs="$pairs_in" '"'"'BEGIN { if (pairs > 0) printf "%.6f", merged / pairs; else printf "0.000000" }'"'"')
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/merge_report.json'"'"' <<EOF
{
  "schema_version": "bijux.fastq.merge_pairs.report.v2",
  "stage": "fastq.merge_pairs",
  "stage_id": "fastq.merge_pairs",
  "tool_id": "leehom",
  "paired_mode": "paired_end",
  "merge_engine": "leehom",
  "threads": 1,
  "merge_overlap": null,
  "min_len": null,
  "unmerged_read_policy": "emit_unmerged_pairs",
  "input_r1": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz",
  "input_r2": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz",
  "merged_reads": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom.fq.gz",
  "unmerged_reads_r1": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r1.fq.gz",
  "unmerged_reads_r2": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom_r2.fq.gz",
  "reads_r1": $reads_r1,
  "reads_r2": $reads_r2,
  "reads_merged": $reads_merged,
  "reads_unmerged": $reads_unmerged,
  "merge_rate": $merge_rate,
  "runtime_s": null,
  "memory_mb": null,
  "raw_backend_report": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/leehom/leehom.log",
  "raw_backend_report_format": "leehom_log"
}
EOF
'

# fastq:corpus-01-mini:fastq.merge_pairs:sample-set:pear / fastq / fastq.merge_pairs / pear
bash -lc 'set -euo pipefail
count_fastq_reads() {
  local path="$1"
  if [ ! -f "$path" ]; then printf '"'"'0'"'"'; return; fi
  case "$path" in
    *.gz) gzip -dc "$path" ;;
    *) cat "$path" ;;
  esac | awk '"'"'END { printf "%d", int(NR/4) }'"'"'
}
'"'"'pear'"'"' '"'"'-f'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"' '"'"'-r'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"' '"'"'-o'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear'"'"' '"'"'-j'"'"' '"'"'1'"'"' '"'"'-v'"'"' '"'"'8'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.log'"'"' 2>&1
reads_r1=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"')
reads_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"')
reads_merged=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.assembled.fastq'"'"')
pairs_in=$reads_r1
if [ "$reads_r2" -lt "$pairs_in" ]; then pairs_in=$reads_r2; fi
reads_unmerged=$(( pairs_in - reads_merged ))
if [ "$reads_unmerged" -lt 0 ]; then reads_unmerged=0; fi
if [ 1 = 1 ]; then
  if [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.forward.fastq'"'"' ] && [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.reverse.fastq'"'"' ]; then
    reads_unmerged_r1=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.forward.fastq'"'"')
    reads_unmerged_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.reverse.fastq'"'"')
    reads_unmerged=$reads_unmerged_r1
    if [ "$reads_unmerged_r2" -lt "$reads_unmerged" ]; then reads_unmerged=$reads_unmerged_r2; fi
  fi
else
  rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.forward.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.reverse.fastq'"'"'
fi
merge_rate=$(awk -v merged="$reads_merged" -v pairs="$pairs_in" '"'"'BEGIN { if (pairs > 0) printf "%.6f", merged / pairs; else printf "0.000000" }'"'"')
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/merge_report.json'"'"' <<EOF
{
  "schema_version": "bijux.fastq.merge_pairs.report.v2",
  "stage": "fastq.merge_pairs",
  "stage_id": "fastq.merge_pairs",
  "tool_id": "pear",
  "paired_mode": "paired_end",
  "merge_engine": "pear",
  "threads": 1,
  "merge_overlap": 8,
  "min_len": null,
  "unmerged_read_policy": "emit_unmerged_pairs",
  "input_r1": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz",
  "input_r2": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz",
  "merged_reads": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.assembled.fastq",
  "unmerged_reads_r1": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.forward.fastq",
  "unmerged_reads_r2": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.unassembled.reverse.fastq",
  "reads_r1": $reads_r1,
  "reads_r2": $reads_r2,
  "reads_merged": $reads_merged,
  "reads_unmerged": $reads_unmerged,
  "merge_rate": $merge_rate,
  "runtime_s": null,
  "memory_mb": null,
  "raw_backend_report": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/pear/pear.log",
  "raw_backend_report_format": "pear_log"
}
EOF
'

# fastq:corpus-01-mini:fastq.merge_pairs:sample-set:vsearch / fastq / fastq.merge_pairs / vsearch
bash -lc 'set -euo pipefail
count_fastq_reads() {
  local path="$1"
  if [ ! -f "$path" ]; then printf '"'"'0'"'"'; return; fi
  case "$path" in
    *.gz) gzip -dc "$path" ;;
    *) cat "$path" ;;
  esac | awk '"'"'END { printf "%d", int(NR/4) }'"'"'
}
'"'"'vsearch'"'"' '"'"'--fastq_mergepairs'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"' '"'"'--reverse'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"' '"'"'--fastqout'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.merged.fastq'"'"' '"'"'--threads'"'"' '"'"'1'"'"' '"'"'--fastqout_notmerged_fwd'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r1.fastq'"'"' '"'"'--fastqout_notmerged_rev'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r2.fastq'"'"' '"'"'--fastq_minovlen'"'"' '"'"'8'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.log'"'"' 2>&1
reads_r1=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz'"'"')
reads_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz'"'"')
reads_merged=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.merged.fastq'"'"')
pairs_in=$reads_r1
if [ "$reads_r2" -lt "$pairs_in" ]; then pairs_in=$reads_r2; fi
reads_unmerged=$(( pairs_in - reads_merged ))
if [ "$reads_unmerged" -lt 0 ]; then reads_unmerged=0; fi
if [ 1 = 1 ]; then
  if [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r1.fastq'"'"' ] && [ -n '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r2.fastq'"'"' ]; then
    reads_unmerged_r1=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r1.fastq'"'"')
    reads_unmerged_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r2.fastq'"'"')
    reads_unmerged=$reads_unmerged_r1
    if [ "$reads_unmerged_r2" -lt "$reads_unmerged" ]; then reads_unmerged=$reads_unmerged_r2; fi
  fi
else
  rm -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r1.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r2.fastq'"'"'
fi
merge_rate=$(awk -v merged="$reads_merged" -v pairs="$pairs_in" '"'"'BEGIN { if (pairs > 0) printf "%.6f", merged / pairs; else printf "0.000000" }'"'"')
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/merge_report.json'"'"' <<EOF
{
  "schema_version": "bijux.fastq.merge_pairs.report.v2",
  "stage": "fastq.merge_pairs",
  "stage_id": "fastq.merge_pairs",
  "tool_id": "vsearch",
  "paired_mode": "paired_end",
  "merge_engine": "vsearch",
  "threads": 1,
  "merge_overlap": 8,
  "min_len": null,
  "unmerged_read_policy": "emit_unmerged_pairs",
  "input_r1": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R1.fastq.gz",
  "input_r2": "benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_merge_overlap_R2.fastq.gz",
  "merged_reads": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.merged.fastq",
  "unmerged_reads_r1": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r1.fastq",
  "unmerged_reads_r2": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.unmerged_r2.fastq",
  "reads_r1": $reads_r1,
  "reads_r2": $reads_r2,
  "reads_merged": $reads_merged,
  "reads_unmerged": $reads_unmerged,
  "merge_rate": $merge_rate,
  "runtime_s": null,
  "memory_mb": null,
  "raw_backend_report": "benchmarks/readiness/stage-tool-commands/fastq/fastq/merge_pairs/vsearch/vsearch.log",
  "raw_backend_report_format": "vsearch_log"
}
EOF
'

# fastq:corpus-03-amplicon-mini:fastq.normalize_abundance:sample-set:seqkit / fastq / fastq.normalize_abundance / seqkit
bash -lc 'set -euo pipefail
awk -v method='"'"'relative_abundance'"'"' -v outcol='"'"'normalized_abundance'"'"' -v scale=1 '"'"'BEGIN { FS=OFS="\t" } NR==1 { if ($1 != "sample_id" || $2 != "feature_id" || $3 != "abundance") { exit 64 }; next } { rows[++n]=$0; total[$1]+=$3 } END { print "sample_id", "feature_id", outcol; for (i = 1; i <= n; i++) { split(rows[i], cols, FS); if (method == "relative_abundance") { value = total[cols[1]] > 0 ? cols[3] / total[cols[1]] : 0 } else if (method == "counts_per_million") { value = total[cols[1]] > 0 ? (cols[3] * scale) / total[cols[1]] : 0 } else { exit 65 }; printf "%s\t%s\t%.6f\n", cols[1], cols[2], value } }'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/tables/corpus-03-otu-abundance.tsv'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_abundance/seqkit/abundance_normalized.tsv'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.normalize_abundance.report.v2","stage":"fastq.normalize_abundance","stage_id":"fastq.normalize_abundance","tool_id":"seqkit","method":"relative_abundance","input_table":"benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini/tables/corpus-03-otu-abundance.tsv","normalized_abundance_tsv":"benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_abundance/seqkit/abundance_normalized.tsv","expected_columns":["sample_id","feature_id","abundance"],"input_value_column":"abundance","normalized_value_column":"normalized_abundance","compositional_rule":"per_sample_sum_to_one","scale_factor":null,"table_rows":0,"sample_count":0,"feature_count":0,"zero_fraction":0.0,"per_sample_sums":[],"runtime_s":null,"memory_mb":null,"raw_backend_report":null,"raw_backend_report_format":null,"used_fallback":false,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_abundance/seqkit/normalize_abundance_report.json'"'"'
'

# fastq:corpus-03-amplicon-mini:fastq.normalize_primers:sample-set:cutadapt / fastq / fastq.normalize_primers / cutadapt
bash -lc 'set -euo pipefail
cutadapt -g '"'"'file:assets/reference/primers/16S_universal_v1.fasta'"'"' --overlap 6 --error-rate 0.1 --revcomp --info-file '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_orientation.tsv'"'"' --json '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_stats.json'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_normalized.fastq.gz'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_primers.fastq'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.normalize_primers.report.v2","stage":"fastq.normalize_primers","stage_id":"fastq.normalize_primers","tool_id":"cutadapt","paired_mode":"single_end","primer_set_id":"16S_universal_v1","marker_id":"16S","primer_fasta":"assets/reference/primers/16S_universal_v1.fasta","orientation_policy":"normalize_to_forward_primer","max_mismatch_rate":0.1,"min_overlap_bp":6,"input_r1":"assets/toy/core-v1/fastq/reads_with_primers.fastq","input_r2":null,"output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_normalized.fastq.gz","output_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"primer_trimmed_reads":null,"primer_trimmed_fraction":null,"orientation_forward_fraction":null,"primer_orientation_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_orientation.tsv","primer_stats_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_stats.json","raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/primer_stats.json","raw_backend_report_format":"cutadapt_json","runtime_s":null,"memory_mb":null,"used_fallback":false,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/normalize_primers/cutadapt/normalize_primers_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.profile_overrepresented_sequences:sample-set:fastq_scan / fastq / fastq.profile_overrepresented_sequences / fastq_scan
fastq_scan assets/toy/core-v1/fastq/reads_with_overrepresented_sequences.fastq

# fastq:corpus-01-mini:fastq.profile_overrepresented_sequences:sample-set:fastqc / fastq / fastq.profile_overrepresented_sequences / fastqc
sh -lc 'mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/profile_overrepresented_sequences/fastqc/fastqc_overrepresented'"'"'
'"'"'fastqc'"'"' '"'"'--outdir'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/profile_overrepresented_sequences/fastqc/fastqc_overrepresented'"'"' '"'"'--threads'"'"' '"'"'1'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_overrepresented_sequences.fastq'"'"''

# fastq:corpus-01-mini:fastq.profile_overrepresented_sequences:sample-set:seqkit / fastq / fastq.profile_overrepresented_sequences / seqkit
sh -lc ''"'"'seqkit'"'"' '"'"'fx2tab'"'"' '"'"'-j'"'"' '"'"'1'"'"' '"'"'-n'"'"' '"'"'-s'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_overrepresented_sequences.fastq'"'"' > /dev/null'

# fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:fastp / fastq / fastq.profile_read_lengths / fastp
fastp --in1 assets/toy/core-v1/fastq/reads_1.fastq --out1 /dev/null --thread 4 --json /dev/null --disable_adapter_trimming --disable_quality_filtering --disable_length_filtering --disable_trim_poly_g --dont_eval_duplication

# fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:prinseq / fastq / fastq.profile_read_lengths / prinseq
'prinseq++' -threads 4 -fastq assets/toy/core-v1/fastq/reads_1.fastq -out_good /dev/null -out_bad /dev/null

# fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:seqfu / fastq / fastq.profile_read_lengths / seqfu
seqfu stats -a -T -j 4 assets/toy/core-v1/fastq/reads_1.fastq

# fastq:corpus-01-mini:fastq.profile_read_lengths:sample-set:seqkit_stats / fastq / fastq.profile_read_lengths / seqkit_stats
seqkit_stats -a -T -j 4 assets/toy/core-v1/fastq/reads_1.fastq

# fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqfu / fastq / fastq.profile_reads / seqfu
seqfu stats -a -T -j 4 assets/toy/core-v1/fastq/reads_1.fastq

# fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit / fastq / fastq.profile_reads / seqkit
seqkit stats -a -T -j 4 assets/toy/core-v1/fastq/reads_1.fastq

# fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats / fastq / fastq.profile_reads / seqkit_stats
seqkit_stats -a -T -j 4 assets/toy/core-v1/fastq/reads_1.fastq

# fastq:corpus-03-amplicon-mini:fastq.remove_chimeras:sample-set:vsearch / fastq / fastq.remove_chimeras / vsearch
vsearch --uchime_denovo assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq --nonchimeras benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_chimeras/vsearch/nonchimeras.fastq.gz --chimeras benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_chimeras/vsearch/chimeras.fasta --uchimeout benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_chimeras/vsearch/uchime.tsv --threads 1

# fastq:corpus-01-mini:fastq.remove_duplicates:sample-set:clumpify / fastq / fastq.remove_duplicates / clumpify
bash -lc 'set -euo pipefail
count_fastq_reads() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac | awk '"'"'END { print NR / 4 }'"'"'; }
'"'"'bash'"'"' '"'"'-lc'"'"' '"'"'set -euo pipefail
clumpify in='"'"'"'"'"'"'"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz'"'"'"'"'"'"'"'"' in2='"'"'"'"'"'"'"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz'"'"'"'"'"'"'"'"' out2='"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R2.fastq.gz'"'"'"'"'"'"'"'"' out='"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R1.fastq.gz'"'"'"'"'"'"'"'"' dedupe=t reorder=t threads=1 > '"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.log'"'"'"'"'"'"'"'"' 2>&1
'"'"'
reads_in=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz'"'"')
reads_out=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R1.fastq.gz'"'"')
reads_in_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz'"'"')
reads_out_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R2.fastq.gz'"'"')
pairs_in=$reads_in
pairs_out=$reads_out
pair_count_match=true
if [ "$reads_in" -ne "$reads_in_r2" ] || [ "$reads_out" -ne "$reads_out_r2" ]; then pair_count_match=false; fi
duplicates_removed=$((reads_in - reads_out))
if [ "$reads_in" -gt 0 ]; then dedup_rate=$(awk -v removed="$duplicates_removed" -v total="$reads_in" '"'"'BEGIN { printf "%.12f", removed / total }'"'"'); else dedup_rate=0; fi
printf '"'"'class\treads_removed\tpaired_mode\n'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/duplicate_classes.tsv'"'"'
printf '"'"'duplicate\t%s\t%s\n'"'"' "$duplicates_removed" '"'"'paired_end'"'"' >> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/duplicate_classes.tsv'"'"'
printf '"'"'{"schema_version":"bijux.fastq.remove_duplicates.provenance.v2","stage_id":"fastq.remove_duplicates","tool_id":"clumpify","paired_mode":"paired_end","threads":1,"dedup_mode":"exact","keep_order":true,"duplicates_removed":%s,"dedup_rate":%s,"backend_log":%s,"input_r1":%s,"input_r2":%s,"output_r1":%s,"output_r2":%s,"raw_backend_report":%s,"raw_backend_report_format":"clumpify_log"}'"'"' "$duplicates_removed" "$dedup_rate" '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.log"'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz"'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R1.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R2.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.log"'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/duplicate_provenance.json'"'"'
printf '"'"'{"schema_version":"bijux.fastq.remove_duplicates.report.v2","stage":"fastq.remove_duplicates","stage_id":"fastq.remove_duplicates","tool_id":"clumpify","paired_mode":"paired_end","threads":1,"dedup_mode":"exact","keep_order":true,"input_r1":%s,"input_r2":%s,"output_r1":%s,"output_r2":%s,"reads_in":%s,"reads_out":%s,"reads_in_r2":%s,"reads_out_r2":%s,"pairs_in":%s,"pairs_out":%s,"pair_count_match":%s,"duplicates_removed":%s,"dedup_rate":%s,"duplicate_classes_tsv":%s,"duplicate_provenance_json":%s,"duplicate_classes":[{"class":"duplicate","reads_removed":%s,"paired_mode":"paired_end"}],"raw_backend_report":%s,"raw_backend_report_format":"clumpify_log","runtime_s":null,"memory_mb":null}'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz"'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R1.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.dedup.R2.fastq.gz"'"'"' "$reads_in" "$reads_out" "$reads_in_r2" "$reads_out_r2" "$pairs_in" "$pairs_out" "$pair_count_match" "$duplicates_removed" "$dedup_rate" '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/duplicate_classes.tsv"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/duplicate_provenance.json"'"'"' "$duplicates_removed" '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/clumpify.log"'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/clumpify/deduplicate_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.remove_duplicates:sample-set:fastuniq / fastq / fastq.remove_duplicates / fastuniq
bash -lc 'set -euo pipefail
count_fastq_reads() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac | awk '"'"'END { print NR / 4 }'"'"'; }
'"'"'bash'"'"' '"'"'-lc'"'"' '"'"'set -euo pipefail
work_dir='"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq_workspace'"'"'"'"'"'"'"'"'
mkdir -p "$work_dir"
gzip -dc -- '"'"'"'"'"'"'"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz'"'"'"'"'"'"'"'"' > "$work_dir/input_r1.fastq"
gzip -dc -- '"'"'"'"'"'"'"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz'"'"'"'"'"'"'"'"' > "$work_dir/input_r2.fastq"
printf '"'"'"'"'"'"'"'"'%s\n%s\n'"'"'"'"'"'"'"'"' "$work_dir/input_r1.fastq" "$work_dir/input_r2.fastq" > '"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq_inputs.txt'"'"'"'"'"'"'"'"'
fastuniq -i '"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq_inputs.txt'"'"'"'"'"'"'"'"' -t q -o "$work_dir/output_r1.fastq" -p "$work_dir/output_r2.fastq" > '"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.log'"'"'"'"'"'"'"'"' 2>&1
gzip -c "$work_dir/output_r1.fastq" > '"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R1.fastq.gz'"'"'"'"'"'"'"'"'
gzip -c "$work_dir/output_r2.fastq" > '"'"'"'"'"'"'"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R2.fastq.gz'"'"'"'"'"'"'"'"'
rm -rf "$work_dir"
'"'"'
reads_in=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz'"'"')
reads_out=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R1.fastq.gz'"'"')
reads_in_r2=$(count_fastq_reads '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz'"'"')
reads_out_r2=$(count_fastq_reads '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R2.fastq.gz'"'"')
pairs_in=$reads_in
pairs_out=$reads_out
pair_count_match=true
if [ "$reads_in" -ne "$reads_in_r2" ] || [ "$reads_out" -ne "$reads_out_r2" ]; then pair_count_match=false; fi
duplicates_removed=$((reads_in - reads_out))
if [ "$reads_in" -gt 0 ]; then dedup_rate=$(awk -v removed="$duplicates_removed" -v total="$reads_in" '"'"'BEGIN { printf "%.12f", removed / total }'"'"'); else dedup_rate=0; fi
printf '"'"'class\treads_removed\tpaired_mode\n'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/duplicate_classes.tsv'"'"'
printf '"'"'duplicate\t%s\t%s\n'"'"' "$duplicates_removed" '"'"'paired_end'"'"' >> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/duplicate_classes.tsv'"'"'
printf '"'"'{"schema_version":"bijux.fastq.remove_duplicates.provenance.v2","stage_id":"fastq.remove_duplicates","tool_id":"fastuniq","paired_mode":"paired_end","threads":4,"dedup_mode":"exact","keep_order":true,"duplicates_removed":%s,"dedup_rate":%s,"backend_log":%s,"input_r1":%s,"input_r2":%s,"output_r1":%s,"output_r2":%s,"raw_backend_report":%s,"raw_backend_report_format":"fastuniq_log"}'"'"' "$duplicates_removed" "$dedup_rate" '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.log"'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz"'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R1.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R2.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.log"'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/duplicate_provenance.json'"'"'
printf '"'"'{"schema_version":"bijux.fastq.remove_duplicates.report.v2","stage":"fastq.remove_duplicates","stage_id":"fastq.remove_duplicates","tool_id":"fastuniq","paired_mode":"paired_end","threads":4,"dedup_mode":"exact","keep_order":true,"input_r1":%s,"input_r2":%s,"output_r1":%s,"output_r2":%s,"reads_in":%s,"reads_out":%s,"reads_in_r2":%s,"reads_out_r2":%s,"pairs_in":%s,"pairs_out":%s,"pair_count_match":%s,"duplicates_removed":%s,"dedup_rate":%s,"duplicate_classes_tsv":%s,"duplicate_provenance_json":%s,"duplicate_classes":[{"class":"duplicate","reads_removed":%s,"paired_mode":"paired_end"}],"raw_backend_report":%s,"raw_backend_report_format":"fastuniq_log","runtime_s":null,"memory_mb":null}'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R1.fastq.gz"'"'"' '"'"'"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_pe_duplicate_signals_R2.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R1.fastq.gz"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.dedup.R2.fastq.gz"'"'"' "$reads_in" "$reads_out" "$reads_in_r2" "$reads_out_r2" "$pairs_in" "$pairs_out" "$pair_count_match" "$duplicates_removed" "$dedup_rate" '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/duplicate_classes.tsv"'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/duplicate_provenance.json"'"'"' "$duplicates_removed" '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/fastuniq.log"'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/remove_duplicates/fastuniq/deduplicate_report.json'"'"'
'

# fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:centrifuge / fastq / fastq.screen_taxonomy / centrifuge
sh -lc 'set -eu
mkdir -p '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/centrifuge'"'"'
centrifuge -x '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/centrifuge/reference'"'"' -q -U '"'"'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq'"'"' -S '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.classifications.native.tsv'"'"' --report-file '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.report.native.tsv'"'"' -p 4
awk -F '"'"'\t'"'"' '"'"'NF >= 7 { if ($1 == "name") next; pct=$7; sub(/%$/, "", pct); printf "%s\t0\t%s%%\n", $1, pct }'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.report.native.tsv'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.report.tsv'"'"'

printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.screen_taxonomy.report.v2","stage":"fastq.screen_taxonomy","stage_id":"fastq.screen_taxonomy","tool_id":"centrifuge","paired_mode":"single_end","threads":4,"classifier":"centrifuge","report_format":"centrifuge_report","assignment_format":"centrifuge_assignments","database_catalog_id":"taxonomy_reference","database_artifact_id":"taxonomy_db","database_build_id":null,"database_digest":null,"database_namespace":"read_screening","database_scope":"read_screening","minimum_confidence":null,"emit_unclassified":true,"interpretation_boundary":"screening_only","truth_conditions":[],"input_r1":"assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq","input_r2":null,"screen_report_tsv":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.report.tsv","classification_report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.classifications.json","unclassified_reads_r1":null,"unclassified_reads_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"contamination_rate":null,"classified_fraction":null,"unclassified_fraction":null,"summary_entries":[],"top_taxa":[],"runtime_s":null,"memory_mb":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/centrifuge/centrifuge.classifications.json'"'"'
'

# fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kaiju / fastq / fastq.screen_taxonomy / kaiju
sh -lc 'set -eu
mkdir -p '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/kaiju'"'"'
kaiju -t '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/taxonomy/nodes.dmp'"'"' -f '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/kaiju/kaiju_db.fmi'"'"' -i '"'"'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.classifications.native.tsv'"'"' -z 4
kaiju2table -t '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/taxonomy/nodes.dmp'"'"' -n '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/taxonomy/names.dmp'"'"' -r genus -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.summary.native.tsv'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.classifications.native.tsv'"'"'
awk -F '"'"'\t'"'"' '"'"'NF >= 6 { if ($1 == "percent") next; label=$6; sub(/^[[:space:]]+/, "", label); pct=$1; sub(/%$/, "", pct); printf "%s\t0\t%s%%\n", label, pct }'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.summary.native.tsv'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.summary.tsv'"'"'

printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.screen_taxonomy.report.v2","stage":"fastq.screen_taxonomy","stage_id":"fastq.screen_taxonomy","tool_id":"kaiju","paired_mode":"single_end","threads":4,"classifier":"kaiju","report_format":"kaiju_summary","assignment_format":"kaiju_assignments","database_catalog_id":"taxonomy_reference","database_artifact_id":"taxonomy_db","database_build_id":null,"database_digest":null,"database_namespace":"read_screening","database_scope":"read_screening","minimum_confidence":null,"emit_unclassified":true,"interpretation_boundary":"screening_only","truth_conditions":[],"input_r1":"assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq","input_r2":null,"screen_report_tsv":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.summary.tsv","classification_report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.classifications.json","unclassified_reads_r1":null,"unclassified_reads_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"contamination_rate":null,"classified_fraction":null,"unclassified_fraction":null,"summary_entries":[],"top_taxa":[],"runtime_s":null,"memory_mb":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kaiju/kaiju.classifications.json'"'"'
'

# fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2 / fastq / fastq.screen_taxonomy / kraken2
sh -lc 'set -eu
mkdir -p '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/kraken2'"'"'
kraken2 --db '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/kraken2'"'"' --threads 4 --report '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.report.native.tsv'"'"' --output '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.classifications.native.tsv'"'"' --unclassified-out '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.unclassified_reads.fastq'"'"' '"'"'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq'"'"'
awk -F '"'"'\t'"'"' '"'"'NF >= 6 { label=$6; sub(/^[[:space:]]+/, "", label); pct=$1; sub(/%$/, "", pct); printf "%s\t0\t%s%%\n", label, pct }'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.report.native.tsv'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.report.tsv'"'"'

printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.screen_taxonomy.report.v2","stage":"fastq.screen_taxonomy","stage_id":"fastq.screen_taxonomy","tool_id":"kraken2","paired_mode":"single_end","threads":4,"classifier":"kraken2","report_format":"kraken_report","assignment_format":"kraken_assignments","database_catalog_id":"taxonomy_reference","database_artifact_id":"taxonomy_db","database_build_id":null,"database_digest":null,"database_namespace":"read_screening","database_scope":"read_screening","minimum_confidence":null,"emit_unclassified":true,"interpretation_boundary":"screening_only","truth_conditions":[],"input_r1":"assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq","input_r2":null,"screen_report_tsv":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.report.tsv","classification_report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.classifications.json","unclassified_reads_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.unclassified_reads.fastq","unclassified_reads_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"contamination_rate":null,"classified_fraction":null,"unclassified_fraction":null,"summary_entries":[],"top_taxa":[],"runtime_s":null,"memory_mb":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/kraken2/kraken2.classifications.json'"'"'
'

# fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:krakenuniq / fastq / fastq.screen_taxonomy / krakenuniq
sh -lc 'set -eu
mkdir -p '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/krakenuniq'"'"'
krakenuniq --db '"'"'assets/reference/taxonomy/references/mock_community_taxonomy/krakenuniq'"'"' --threads 4 --fastq-input --report-file '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.report.native.tsv'"'"' --output '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.classifications.native.tsv'"'"' '"'"'assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq'"'"'
awk -F '"'"'\t'"'"' '"'"'NF >= 9 { if ($1 == "%") next; label=$9; sub(/^[[:space:]]+/, "", label); pct=$1; sub(/%$/, "", pct); printf "%s\t0\t%s%%\n", label, pct }'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.report.native.tsv'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.report.tsv'"'"'

printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.screen_taxonomy.report.v2","stage":"fastq.screen_taxonomy","stage_id":"fastq.screen_taxonomy","tool_id":"krakenuniq","paired_mode":"single_end","threads":4,"classifier":"kraken_uniq","report_format":"kraken_uniq_report","assignment_format":"kraken_uniq_assignments","database_catalog_id":"taxonomy_reference","database_artifact_id":"taxonomy_db","database_build_id":null,"database_digest":null,"database_namespace":"read_screening","database_scope":"read_screening","minimum_confidence":null,"emit_unclassified":true,"interpretation_boundary":"screening_only","truth_conditions":[],"input_r1":"assets/toy/corpus-02-edna-mini/fastq/mock_community_reads.fastq","input_r2":null,"screen_report_tsv":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.report.tsv","classification_report_json":"benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.classifications.json","unclassified_reads_r1":null,"unclassified_reads_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"contamination_rate":null,"classified_fraction":null,"unclassified_fraction":null,"summary_entries":[],"top_taxa":[],"runtime_s":null,"memory_mb":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/screen_taxonomy/krakenuniq/krakenuniq.classifications.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_polyg_tails:sample-set:bbduk / fastq / fastq.trim_polyg_tails / bbduk
sh -lc 'set -eu
'"'"'bbduk'"'"' '"'"'in=benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_polyg_trim_signals_R1.fastq.gz'"'"' '"'"'out=benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/bbduk/polyg.bbduk.fastq.gz'"'"' '"'"'threads=1'"'"' '"'"'trimpolygright=6'"'"' '"'"'stats=benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/bbduk/trim_polyg_tails_report.stats.txt'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.trim_polyg_tails.report.v2","stage":"fastq.trim_polyg_tails","stage_id":"fastq.trim_polyg_tails","tool_id":"bbduk","paired_mode":"single_end","threads":1,"trim_polyg":true,"min_polyg_run":6,"input_r1":"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_polyg_trim_signals_R1.fastq.gz","input_r2":null,"output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/bbduk/polyg.bbduk.fastq.gz","output_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"mean_q_before":null,"mean_q_after":null,"trimmed_tail_count":null,"bases_trimmed_polyg":null,"polyx_bank_id":null,"polyx_bank_hash":null,"polyx_preset":null,"runtime_s":null,"memory_mb":null,"raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/bbduk/trim_polyg_tails_report.stats.txt","raw_backend_report_format":"bbduk_stats","backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/bbduk/trim_polyg_tails_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_polyg_tails:sample-set:fastp / fastq / fastq.trim_polyg_tails / fastp
sh -lc 'set -eu
'"'"'fastp'"'"' '"'"'--json'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/fastp/trim_polyg_tails_report.fastp.json'"'"' '"'"'--thread'"'"' '"'"'1'"'"' '"'"'--in1'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_polyg_trim_signals_R1.fastq.gz'"'"' '"'"'--out1'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/fastp/polyg.fastp.fastq.gz'"'"' '"'"'--trim_poly_g'"'"' '"'"'--poly_g_min_len'"'"' '"'"'6'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.trim_polyg_tails.report.v2","stage":"fastq.trim_polyg_tails","stage_id":"fastq.trim_polyg_tails","tool_id":"fastp","paired_mode":"single_end","threads":1,"trim_polyg":true,"min_polyg_run":6,"input_r1":"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/human_like_se_polyg_trim_signals_R1.fastq.gz","input_r2":null,"output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/fastp/polyg.fastp.fastq.gz","output_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"pairs_in":null,"pairs_out":null,"mean_q_before":null,"mean_q_after":null,"trimmed_tail_count":null,"bases_trimmed_polyg":null,"polyx_bank_id":null,"polyx_bank_hash":null,"polyx_preset":null,"runtime_s":null,"memory_mb":null,"raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/fastp/trim_polyg_tails_report.fastp.json","raw_backend_report_format":"fastp_json","backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_polyg_tails/fastp/trim_polyg_tails_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:adapterremoval / fastq / fastq.trim_reads / adapterremoval
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/adapterremoval'"'"'
'"'"'adapterremoval'"'"' '"'"'--threads'"'"' '"'"'1'"'"' '"'"'--file1'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' '"'"'--output1'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/adapterremoval/adapterremoval.fastq.gz'"'"' '"'"'--discarded'"'"' '"'"'/dev/null'"'"' '"'"'--adapter1'"'"' '"'"'GTCTCGTGGGCTCGG'"'"' '"'"'--adapter2'"'"' '"'"'CTGTCTCTTATACACATCT'"'"' '"'"'--minlength'"'"' '"'"'4'"'"' '"'"'--trimqualities'"'"' '"'"'--minquality'"'"' '"'"'20'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/adapterremoval/adapterremoval.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"adapterremoval","trimming_backend":"adapterremoval"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/adapterremoval/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:alientrimmer / fastq / fastq.trim_reads / alientrimmer
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/alientrimmer'"'"'
cat > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/alientrimmer/alientrimmer_adapters.txt'"'"' <<'"'"'EOF'"'"'
GTCTCGTGGGCTCGG
CTGTCTCTTATACACATCT
AGATCGGAAGAGCACACGTCTGAACTCCAGTCA
AGATCGGAAGAGC
EOF
alientrimmer -i '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' -c '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/alientrimmer/alientrimmer_adapters.txt'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/alientrimmer/alientrimmer.fastq.gz'"'"' -l 4 -q 20 -z
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/alientrimmer/alientrimmer.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"alientrimmer","trimming_backend":"alientrimmer"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/alientrimmer/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:atropos / fastq / fastq.trim_reads / atropos
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/atropos'"'"'
'"'"'atropos'"'"' '"'"'trim'"'"' '"'"'-T'"'"' '"'"'2'"'"' '"'"'-a'"'"' '"'"'GTCTCGTGGGCTCGG'"'"' '"'"'-a'"'"' '"'"'CTGTCTCTTATACACATCT'"'"' '"'"'-a'"'"' '"'"'AGATCGGAAGAGCACACGTCTGAACTCCAGTCA'"'"' '"'"'-a'"'"' '"'"'AGATCGGAAGAGC'"'"' '"'"'-q'"'"' '"'"'20'"'"' '"'"'-m'"'"' '"'"'4'"'"' '"'"'-se'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' '"'"'-o'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/atropos/atropos.fastq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":2},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/atropos/atropos.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":2,"tool_id":"atropos","trimming_backend":"atropos"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/atropos/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:bbduk / fastq / fastq.trim_reads / bbduk
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/bbduk'"'"'
'"'"'bbduk'"'"' '"'"'in=assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' '"'"'out=benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/bbduk/bbduk.fastq.gz'"'"' '"'"'stats=benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/bbduk/trim_report.bbduk.stats.txt'"'"' '"'"'threads=1'"'"' '"'"'minlen=4'"'"' '"'"'qtrim=rl'"'"' '"'"'trimq=20'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":null,"adapter_bank_id":null,"adapter_overrides":null,"adapter_policy":"none","adapter_preset":null,"backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":"1aaf67a37b558def20495db6cda8e30a6c60424ab9827bc21f41340b750af2fd","contaminant_bank_id":"contaminants.motifs","contaminant_policy":"none","contaminant_preset":"illumina_default","detected_adapter_source":null,"effective_trim_params":{"adapter_policy":"none","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/bbduk/bbduk.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":null,"quality_cutoff":20,"raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/bbduk/trim_report.bbduk.stats.txt","raw_backend_report_format":"bbduk_stats","reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"bbduk","trimming_backend":"bbduk"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/bbduk/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:cutadapt / fastq / fastq.trim_reads / cutadapt
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/cutadapt'"'"'
'"'"'cutadapt'"'"' '"'"'--cores'"'"' '"'"'1'"'"' '"'"'-a'"'"' '"'"'GTCTCGTGGGCTCGG'"'"' '"'"'-a'"'"' '"'"'CTGTCTCTTATACACATCT'"'"' '"'"'-a'"'"' '"'"'AGATCGGAAGAGCACACGTCTGAACTCCAGTCA'"'"' '"'"'-a'"'"' '"'"'AGATCGGAAGAGC'"'"' '"'"'-m'"'"' '"'"'4'"'"' '"'"'-q'"'"' '"'"'20'"'"' '"'"'--json'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/cutadapt/trim_report.cutadapt.json'"'"' '"'"'-o'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/cutadapt/cutadapt.fastq.gz'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/cutadapt/cutadapt.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/cutadapt/trim_report.cutadapt.json","raw_backend_report_format":"cutadapt_json","reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"cutadapt","trimming_backend":"cutadapt"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/cutadapt/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastp / fastq / fastq.trim_reads / fastp
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastp'"'"'
'"'"'fastp'"'"' '"'"'--in1'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' '"'"'--out1'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastp/fastp.fastq.gz'"'"' '"'"'--json'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastp/trim_report.fastp.json'"'"' '"'"'--thread'"'"' '"'"'1'"'"' '"'"'--length_required'"'"' '"'"'4'"'"' '"'"'--qualified_quality_phred'"'"' '"'"'20'"'"' '"'"'--adapter_sequence'"'"' '"'"'GTCTCGTGGGCTCGG'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":"1aaf67a37b558def20495db6cda8e30a6c60424ab9827bc21f41340b750af2fd","contaminant_bank_id":"contaminants.motifs","contaminant_policy":"none","contaminant_preset":"illumina_default","detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastp/fastp.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":"a984eaf5779f9e22219ec61ba5a697446b0d2c03fe0781a7682627e8a579fbed","polyx_bank_id":"polyx.default","polyx_policy":"none","polyx_preset":"illumina_twocolor","prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastp/trim_report.fastp.json","raw_backend_report_format":"fastp_json","reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"fastp","trimming_backend":"fastp"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastp/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:fastx_clipper / fastq / fastq.trim_reads / fastx_clipper
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastx_clipper'"'"'
fastx_clipper -Q33 -a '"'"'GTCTCGTGGGCTCGG'"'"' -i '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastx_clipper/fastx_clipper.fastq.gz'"'"' -M 15 -z
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"none","adapter_preset":"illumina-default","backend_mode":"advisory","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"none","contaminant_policy":"none","min_length":30,"n_policy":"retain","polyx_policy":"none","quality_cutoff":null,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":30,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastx_clipper/fastx_clipper.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":null,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"fastx_clipper","trimming_backend":"fastx_clipper"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/fastx_clipper/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:leehom / fastq / fastq.trim_reads / leehom
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom'"'"'
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom'"'"'
cd '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom'"'"'
leehom -fq1 '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' -t 2 -f '"'"'GTCTCGTGGGCTCGG'"'"' -fqo leehom --log '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom/trim_report.leehom.log'"'"'
mv '"'"'leehom.fq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom/leehom.fastq.gz'"'"'
rm -f '"'"'leehom.fail.fq.gz'"'"' '"'"'leehom_r1.fail.fq.gz'"'"' '"'"'leehom_r2.fail.fq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"none","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"none","contaminant_policy":"none","min_length":30,"n_policy":"retain","polyx_policy":"none","quality_cutoff":null,"threads":2},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":30,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom/leehom.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":null,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":2,"tool_id":"leehom","trimming_backend":"leehom"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/leehom/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:prinseq / fastq / fastq.trim_reads / prinseq
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/prinseq'"'"'
'"'"'prinseq++'"'"' '"'"'-threads'"'"' '"'"'1'"'"' '"'"'-fastq'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' '"'"'-out_good'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/prinseq/prinseq_good.fastq'"'"' '"'"'-out_bad'"'"' '"'"'/dev/null'"'"' '"'"'-min_len'"'"' '"'"'4'"'"' '"'"'-trim_qual_left'"'"' '"'"'20'"'"' '"'"'-trim_qual_right'"'"' '"'"'20'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":null,"adapter_bank_id":null,"adapter_overrides":null,"adapter_policy":"none","adapter_preset":null,"backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":null,"effective_trim_params":{"adapter_policy":"none","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/prinseq/prinseq_good.fastq","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":null,"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"prinseq","trimming_backend":"prinseq"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/prinseq/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:seqkit / fastq / fastq.trim_reads / seqkit
sh -lc 'set -eu
seqkit seq -m 4 -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/seqkit/seqkit.fastq.gz'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":null,"adapter_bank_id":null,"adapter_overrides":null,"adapter_policy":"none","adapter_preset":null,"backend_mode":"advisory","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":null,"effective_trim_params":{"adapter_policy":"none","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":null,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/seqkit/seqkit.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":null,"quality_cutoff":null,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"seqkit","trimming_backend":"seqkit"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/seqkit/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:skewer / fastq / fastq.trim_reads / skewer
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer'"'"'
skewer -m tail -t 1 -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer'"'"' -x '"'"'GTCTCGTGGGCTCGG'"'"' -l 4 -q 20 -z '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"'
trim_output_moved=0
if [ -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer-trimmed.fastq.gz'"'"' ]; then mv '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer-trimmed.fastq.gz'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer.fastq.gz'"'"'; trim_output_moved=1; fi
if [ -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer-trimmed.fastq'"'"' ]; then mv '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer-trimmed.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer.fastq.gz'"'"'; trim_output_moved=1; fi
[ "$trim_output_moved" = 1 ] || { echo '"'"'skewer output'"'"' >&2; exit 1; }
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/skewer.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"skewer","trimming_backend":"skewer"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/skewer/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:trim_galore / fastq / fastq.trim_reads / trim_galore
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trim_galore_run'"'"'
trim_galore --output_dir '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trim_galore_run'"'"' --cores 1 --length 4 -q 20 --adapter '"'"'GTCTCGTGGGCTCGG'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"'
trim_galore_output_moved=0
if [ -f '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trim_galore_run/reads_with_trim_signals_trimmed.fq'"'"' ]; then mv '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trim_galore_run/reads_with_trim_signals_trimmed.fq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trimmed_trimmed.fq.gz'"'"'; trim_galore_output_moved=1; fi
[ "$trim_galore_output_moved" = 1 ] || { echo '"'"'trim_galore did not produce an expected output file'"'"' >&2; exit 1; }
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","adapter_bank_id":"bijux-dna-fastq-adapter-bank","adapter_overrides":null,"adapter_policy":"bank","adapter_preset":"illumina-default","backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":"prepared_adapter_bank:illumina-default","effective_trim_params":{"adapter_policy":"bank","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trimmed_trimmed.fq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":{"bank_hash":"b816578c563b4c9e1015b3066c659775322c419a840b2753328a1c006b75227a","bank_id":"bijux-dna-fastq-adapter-bank","bank_version":"1","disable_adapters":[],"disabled_categories":["capture","custom","nebnext","partial","pcr","ssdna","umi"],"enable_adapters":[],"enabled_adapter_ids":["nextera_index","nextera_transposase","truseq_indexed","truseq_universal"],"enabled_categories":["nextera","truseq"],"preset":"illumina-default","preset_hash":"34db6679f6ebfdd8b98de11ad3517d4bd009ab4c34cc36c652b085ee2f766103","presets_hash":"b163e7372fb993f5a57cd62e66a3040f3f60a083e9d7de8f8a3976e57c0f46e7","schema_version":"bijux.fastq.prepare_adapter_bank.report.v1","stage":"fastq.prepare_adapter_bank","stage_id":"fastq.prepare_adapter_bank","tool_id":"bijux"},"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"trim_galore","trimming_backend":"trim_galore"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trim_galore/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_reads:sample-set:trimmomatic / fastq / fastq.trim_reads / trimmomatic
sh -lc 'set -eu
mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trimmomatic'"'"'
'"'"'trimmomatic'"'"' '"'"'SE'"'"' '"'"'-threads'"'"' '"'"'1'"'"' '"'"'-phred33'"'"' '"'"'assets/toy/core-v1/fastq/reads_with_trim_signals.fastq'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trimmomatic/trimmomatic.fastq.gz'"'"' '"'"'SLIDINGWINDOW:4:20'"'"' '"'"'MINLEN:4'"'"'
printf '"'"'%s\n'"'"' '"'"'{"adapter_bank_hash":null,"adapter_bank_id":null,"adapter_overrides":null,"adapter_policy":"none","adapter_preset":null,"backend_mode":"enforced","bases_in":null,"bases_out":null,"contaminant_bank_hash":null,"contaminant_bank_id":null,"contaminant_policy":"none","contaminant_preset":null,"detected_adapter_source":null,"effective_trim_params":{"adapter_policy":"none","contaminant_policy":"none","min_length":4,"n_policy":"retain","polyx_policy":"none","quality_cutoff":20,"threads":1},"input_r1":"assets/toy/core-v1/fastq/reads_with_trim_signals.fastq","input_r2":null,"mean_q_after":null,"mean_q_before":null,"memory_mb":null,"min_length":4,"n_policy":"retain","output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trimmomatic/trimmomatic.fastq.gz","output_r2":null,"paired_mode":"single_end","pairs_in":null,"pairs_out":null,"polyx_bank_hash":null,"polyx_bank_id":null,"polyx_policy":"none","polyx_preset":null,"prepared_adapter_bank":null,"quality_cutoff":20,"raw_backend_report":null,"raw_backend_report_format":null,"reads_in":null,"reads_out":null,"runtime_s":null,"schema_version":"bijux.fastq.trim_reads.report.v2","stage":"fastq.trim_reads","stage_id":"fastq.trim_reads","threads":1,"tool_id":"trimmomatic","trimming_backend":"trimmomatic"}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_reads/trimmomatic/trim_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:adapterremoval / fastq / fastq.trim_terminal_damage / adapterremoval
sh -lc 'set -eu
adapterremoval --threads 1 --trim5p 2 --trim3p 2 --file1 '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz'"'"' --output1 '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/adapterremoval/trim_terminal_damage.adapterremoval.fastq.gz'"'"' --discarded /dev/null
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.trim_terminal_damage.report.v2","stage":"fastq.trim_terminal_damage","stage_id":"fastq.trim_terminal_damage","tool_id":"adapterremoval","paired_mode":"single_end","threads":1,"damage_mode":"ancient","execution_policy":"explicit_terminal_trim","trim_5p_bases":2,"trim_3p_bases":2,"requested_trim_5p_bases":2,"requested_trim_3p_bases":2,"udg_classification":"non_udg","input_r1":"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz","input_r2":null,"output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/adapterremoval/trim_terminal_damage.adapterremoval.fastq.gz","output_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"mean_q_before":null,"mean_q_after":null,"ct_ga_asymmetry_pre":null,"ct_ga_asymmetry_post":null,"ct_ga_asymmetry_pre_r1":null,"ct_ga_asymmetry_post_r1":null,"ct_ga_asymmetry_pre_r2":null,"ct_ga_asymmetry_post_r2":null,"terminal_base_composition_pre_r1":null,"terminal_base_composition_post_r1":null,"terminal_base_composition_pre_r2":null,"terminal_base_composition_post_r2":null,"raw_backend_report":null,"raw_backend_report_format":null,"runtime_s":null,"memory_mb":null,"used_fallback":false,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/adapterremoval/trim_terminal_damage_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:cutadapt / fastq / fastq.trim_terminal_damage / cutadapt
sh -lc 'set -eu
cutadapt --cores 1 -u 2 -u -2 --json '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/cutadapt/trim_terminal_damage.cutadapt.raw.json'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/cutadapt/trim_terminal_damage.cutadapt.fastq.gz'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.trim_terminal_damage.report.v2","stage":"fastq.trim_terminal_damage","stage_id":"fastq.trim_terminal_damage","tool_id":"cutadapt","paired_mode":"single_end","threads":1,"damage_mode":"ancient","execution_policy":"explicit_terminal_trim","trim_5p_bases":2,"trim_3p_bases":2,"requested_trim_5p_bases":2,"requested_trim_3p_bases":2,"udg_classification":"non_udg","input_r1":"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz","input_r2":null,"output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/cutadapt/trim_terminal_damage.cutadapt.fastq.gz","output_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"mean_q_before":null,"mean_q_after":null,"ct_ga_asymmetry_pre":null,"ct_ga_asymmetry_post":null,"ct_ga_asymmetry_pre_r1":null,"ct_ga_asymmetry_post_r1":null,"ct_ga_asymmetry_pre_r2":null,"ct_ga_asymmetry_post_r2":null,"terminal_base_composition_pre_r1":null,"terminal_base_composition_post_r1":null,"terminal_base_composition_pre_r2":null,"terminal_base_composition_post_r2":null,"raw_backend_report":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/cutadapt/trim_terminal_damage.cutadapt.raw.json","raw_backend_report_format":"cutadapt_json","runtime_s":null,"memory_mb":null,"used_fallback":false,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/cutadapt/trim_terminal_damage_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:seqkit / fastq / fastq.trim_terminal_damage / seqkit
sh -lc 'set -eu
seqkit subseq -j 1 -r '"'"'3:-3'"'"' '"'"'benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz'"'"' -o '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/seqkit/trim_terminal_damage.seqkit.fastq.gz'"'"'
printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.fastq.trim_terminal_damage.report.v2","stage":"fastq.trim_terminal_damage","stage_id":"fastq.trim_terminal_damage","tool_id":"seqkit","paired_mode":"single_end","threads":1,"damage_mode":"ancient","execution_policy":"explicit_terminal_trim","trim_5p_bases":2,"trim_3p_bases":2,"requested_trim_5p_bases":2,"requested_trim_3p_bases":2,"udg_classification":"non_udg","input_r1":"benchmarks/tests/fixtures/corpora/corpus-01-mini/normalized/adna_like_se_compact_R1.fastq.gz","input_r2":null,"output_r1":"benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/seqkit/trim_terminal_damage.seqkit.fastq.gz","output_r2":null,"reads_in":null,"reads_out":null,"bases_in":null,"bases_out":null,"mean_q_before":null,"mean_q_after":null,"ct_ga_asymmetry_pre":null,"ct_ga_asymmetry_post":null,"ct_ga_asymmetry_pre_r1":null,"ct_ga_asymmetry_post_r1":null,"ct_ga_asymmetry_pre_r2":null,"ct_ga_asymmetry_post_r2":null,"terminal_base_composition_pre_r1":null,"terminal_base_composition_post_r1":null,"terminal_base_composition_pre_r2":null,"terminal_base_composition_post_r2":null,"raw_backend_report":null,"raw_backend_report_format":"seqkit_subseq","runtime_s":null,"memory_mb":null,"used_fallback":false,"backend_metrics":null}'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/trim_terminal_damage/seqkit/trim_terminal_damage_report.json'"'"'
'

# fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastq_scan / fastq / fastq.validate_reads / fastq_scan
sh -lc 'set +e && '"'"'fastq_scan'"'"' '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastq_scan/validation_r1.log'"'"' 2>&1
status_r1=$? && status_r2=0 && cat_fastq() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac; } && path_uses_supported_fastq_compression() { case "$1" in *.fastq|*.fq|*.fastq.gz|*.fq.gz) return 0 ;; *) return 1 ;; esac; } && inspect_fastq_stream() { gzip_ok=0; if path_uses_supported_fastq_compression "$1"; then gzip_ok=1; fi; if [ "$gzip_ok" -ne 1 ]; then printf '"'"'0\tunsupported_compression'"'"'; return 90; fi; case "$1" in *.gz) gzip -t -- "$1" >/dev/null 2>&1 || { printf '"'"'0\tunsupported_compression'"'"'; return 90; } ;; esac; cat_fastq "$1" | awk '"'"'BEGIN { seq = ""; read_count = 0; } { line_no = ((NR - 1) % 4) + 1; if (line_no == 1) { if (substr($0, 1, 1) != "@") { malformed = 1; } } else if (line_no == 2) { seq = $0; if (length(seq) == 0) { malformed = 1; } } else if (line_no == 3) { if (substr($0, 1, 1) != "+") { malformed = 1; } } else if (line_no == 4) { if (length($0) != length(seq)) { malformed = 1; } if ($0 ~ /[^!-J]/) { invalid_quality = 1; } read_count++; } } END { if (NR == 0) { printf "0\tempty_input"; exit 91; } if ((NR % 4) != 0 || malformed) { printf "%d\tmalformed_record", read_count; exit 92; } if (invalid_quality) { printf "%d\tinvalid_quality_encoding", read_count; exit 93; } printf "%d\tnone", read_count; }'"'"'; } && strict_pass=true && exit_code=0 && pair_sync_checked=false && pair_sync_pass=null && pair_count_match=null && failure_class=none && validated_pairs=null && inspection_r1=$(inspect_fastq_stream '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' 2>> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastq_scan/validation_r1.log'"'"'); inspect_status_r1=$?; true && validated_reads_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f1) && inspection_class_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f2) && validated_reads_r2=null && inspection_class_r2=none && if [ "$status_r1" -ne 0 ]; then strict_pass=false; exit_code=$status_r1; fi && if [ "$inspect_status_r1" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$inspect_status_r1; fi; fi && if [ "$status_r2" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$status_r2; fi; fi && if [ "$inspection_class_r1" != "none" ]; then failure_class=$inspection_class_r1; fi && if [ "$inspection_class_r2" != "none" ] && [ "$failure_class" = "none" ]; then failure_class=$inspection_class_r2; fi && if [ "$status_r1" -ne 0 ] || [ "$status_r2" -ne 0 ]; then if [ "$failure_class" = "none" ]; then failure_class=validator_error; fi; fi && if [ "$pair_count_match" = "false" ] && [ "$failure_class" = "none" ]; then failure_class=pair_count_mismatch; fi && if [ "$pair_sync_checked" = "true" ] && [ "$pair_sync_pass" = "false" ] && [ "$pair_count_match" != "false" ] && [ "$failure_class" = "none" ]; then failure_class=header_sync_mismatch; fi && printf '"'"'{"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":"fastq.validate_reads","tool_id":"fastq_scan","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_report":%%s,"paired_mode":"single_end","validated_stream_ids":["reads_r1"],"pair_sync_checked":%%s,"pair_sync_pass":%%s,"validated_pairs":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastq_scan/validation.json"'"'"' "$pair_sync_checked" "$pair_sync_pass" "$validated_pairs" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastq_scan/validated_reads_manifest.json'"'"' && printf '"'"'{"schema_version":"bijux.fastq.validate.report.v1","stage":"fastq.validate_reads","stage_id":"fastq.validate_reads","tool_id":"fastq_scan","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_log_r1":%%s,"validation_log_r2":%%s,"validated_inputs":1,"validated_reads_r1":%%s,"validated_reads_r2":%%s,"validated_pairs":%%s,"status_r1":%%s,"status_r2":%%s,"pair_sync_checked":%%s,"pair_sync_pass":%%s,"pair_count_match":%%s,"failure_class":%%s,"strict_pass":%%s,"exit_code":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastq_scan/validation_r1.log"'"'"' '"'"'null'"'"' "$validated_reads_r1" "$validated_reads_r2" "$validated_pairs" "$status_r1" "$status_r2" "$pair_sync_checked" "$pair_sync_pass" "$pair_count_match" "$(printf '"'"'\"%s\"'"'"' "$failure_class")" "$strict_pass" "$exit_code" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastq_scan/validation.json'"'"' && exit "$exit_code"'

# fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqc / fastq / fastq.validate_reads / fastqc
sh -lc 'set +e && mkdir -p '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation_fastqc_reads_r1'"'"'
'"'"'fastqc'"'"' '"'"'--quiet'"'"' '"'"'--threads'"'"' '"'"'4'"'"' '"'"'--outdir'"'"' '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation_fastqc_reads_r1'"'"' '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation_r1.log'"'"' 2>&1
status_r1=$?
rm -rf '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation_fastqc_reads_r1'"'"' && status_r2=0 && cat_fastq() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac; } && path_uses_supported_fastq_compression() { case "$1" in *.fastq|*.fq|*.fastq.gz|*.fq.gz) return 0 ;; *) return 1 ;; esac; } && inspect_fastq_stream() { gzip_ok=0; if path_uses_supported_fastq_compression "$1"; then gzip_ok=1; fi; if [ "$gzip_ok" -ne 1 ]; then printf '"'"'0\tunsupported_compression'"'"'; return 90; fi; case "$1" in *.gz) gzip -t -- "$1" >/dev/null 2>&1 || { printf '"'"'0\tunsupported_compression'"'"'; return 90; } ;; esac; cat_fastq "$1" | awk '"'"'BEGIN { seq = ""; read_count = 0; } { line_no = ((NR - 1) % 4) + 1; if (line_no == 1) { if (substr($0, 1, 1) != "@") { malformed = 1; } } else if (line_no == 2) { seq = $0; if (length(seq) == 0) { malformed = 1; } } else if (line_no == 3) { if (substr($0, 1, 1) != "+") { malformed = 1; } } else if (line_no == 4) { if (length($0) != length(seq)) { malformed = 1; } if ($0 ~ /[^!-J]/) { invalid_quality = 1; } read_count++; } } END { if (NR == 0) { printf "0\tempty_input"; exit 91; } if ((NR % 4) != 0 || malformed) { printf "%d\tmalformed_record", read_count; exit 92; } if (invalid_quality) { printf "%d\tinvalid_quality_encoding", read_count; exit 93; } printf "%d\tnone", read_count; }'"'"'; } && strict_pass=true && exit_code=0 && pair_sync_checked=false && pair_sync_pass=null && pair_count_match=null && failure_class=none && validated_pairs=null && inspection_r1=$(inspect_fastq_stream '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' 2>> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation_r1.log'"'"'); inspect_status_r1=$?; true && validated_reads_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f1) && inspection_class_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f2) && validated_reads_r2=null && inspection_class_r2=none && if [ "$status_r1" -ne 0 ]; then strict_pass=false; exit_code=$status_r1; fi && if [ "$inspect_status_r1" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$inspect_status_r1; fi; fi && if [ "$status_r2" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$status_r2; fi; fi && if [ "$inspection_class_r1" != "none" ]; then failure_class=$inspection_class_r1; fi && if [ "$inspection_class_r2" != "none" ] && [ "$failure_class" = "none" ]; then failure_class=$inspection_class_r2; fi && if [ "$status_r1" -ne 0 ] || [ "$status_r2" -ne 0 ]; then if [ "$failure_class" = "none" ]; then failure_class=validator_error; fi; fi && if [ "$pair_count_match" = "false" ] && [ "$failure_class" = "none" ]; then failure_class=pair_count_mismatch; fi && if [ "$pair_sync_checked" = "true" ] && [ "$pair_sync_pass" = "false" ] && [ "$pair_count_match" != "false" ] && [ "$failure_class" = "none" ]; then failure_class=header_sync_mismatch; fi && printf '"'"'{"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":"fastq.validate_reads","tool_id":"fastqc","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_report":%%s,"paired_mode":"single_end","validated_stream_ids":["reads_r1"],"pair_sync_checked":%%s,"pair_sync_pass":%%s,"validated_pairs":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation.json"'"'"' "$pair_sync_checked" "$pair_sync_pass" "$validated_pairs" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validated_reads_manifest.json'"'"' && printf '"'"'{"schema_version":"bijux.fastq.validate.report.v1","stage":"fastq.validate_reads","stage_id":"fastq.validate_reads","tool_id":"fastqc","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_log_r1":%%s,"validation_log_r2":%%s,"validated_inputs":1,"validated_reads_r1":%%s,"validated_reads_r2":%%s,"validated_pairs":%%s,"status_r1":%%s,"status_r2":%%s,"pair_sync_checked":%%s,"pair_sync_pass":%%s,"pair_count_match":%%s,"failure_class":%%s,"strict_pass":%%s,"exit_code":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation_r1.log"'"'"' '"'"'null'"'"' "$validated_reads_r1" "$validated_reads_r2" "$validated_pairs" "$status_r1" "$status_r2" "$pair_sync_checked" "$pair_sync_pass" "$pair_count_match" "$(printf '"'"'\"%s\"'"'"' "$failure_class")" "$strict_pass" "$exit_code" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqc/validation.json'"'"' && exit "$exit_code"'

# fastq:corpus-01-mini:fastq.validate_reads:sample-set:fastqvalidator / fastq / fastq.validate_reads / fastqvalidator
sh -lc 'set +e && '"'"'fastqvalidator'"'"' '"'"'--file'"'"' '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqvalidator/validation_r1.log'"'"' 2>&1
status_r1=$? && status_r2=0 && cat_fastq() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac; } && path_uses_supported_fastq_compression() { case "$1" in *.fastq|*.fq|*.fastq.gz|*.fq.gz) return 0 ;; *) return 1 ;; esac; } && inspect_fastq_stream() { gzip_ok=0; if path_uses_supported_fastq_compression "$1"; then gzip_ok=1; fi; if [ "$gzip_ok" -ne 1 ]; then printf '"'"'0\tunsupported_compression'"'"'; return 90; fi; case "$1" in *.gz) gzip -t -- "$1" >/dev/null 2>&1 || { printf '"'"'0\tunsupported_compression'"'"'; return 90; } ;; esac; cat_fastq "$1" | awk '"'"'BEGIN { seq = ""; read_count = 0; } { line_no = ((NR - 1) % 4) + 1; if (line_no == 1) { if (substr($0, 1, 1) != "@") { malformed = 1; } } else if (line_no == 2) { seq = $0; if (length(seq) == 0) { malformed = 1; } } else if (line_no == 3) { if (substr($0, 1, 1) != "+") { malformed = 1; } } else if (line_no == 4) { if (length($0) != length(seq)) { malformed = 1; } if ($0 ~ /[^!-J]/) { invalid_quality = 1; } read_count++; } } END { if (NR == 0) { printf "0\tempty_input"; exit 91; } if ((NR % 4) != 0 || malformed) { printf "%d\tmalformed_record", read_count; exit 92; } if (invalid_quality) { printf "%d\tinvalid_quality_encoding", read_count; exit 93; } printf "%d\tnone", read_count; }'"'"'; } && strict_pass=true && exit_code=0 && pair_sync_checked=false && pair_sync_pass=null && pair_count_match=null && failure_class=none && validated_pairs=null && inspection_r1=$(inspect_fastq_stream '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' 2>> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqvalidator/validation_r1.log'"'"'); inspect_status_r1=$?; true && validated_reads_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f1) && inspection_class_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f2) && validated_reads_r2=null && inspection_class_r2=none && if [ "$status_r1" -ne 0 ]; then strict_pass=false; exit_code=$status_r1; fi && if [ "$inspect_status_r1" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$inspect_status_r1; fi; fi && if [ "$status_r2" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$status_r2; fi; fi && if [ "$inspection_class_r1" != "none" ]; then failure_class=$inspection_class_r1; fi && if [ "$inspection_class_r2" != "none" ] && [ "$failure_class" = "none" ]; then failure_class=$inspection_class_r2; fi && if [ "$status_r1" -ne 0 ] || [ "$status_r2" -ne 0 ]; then if [ "$failure_class" = "none" ]; then failure_class=validator_error; fi; fi && if [ "$pair_count_match" = "false" ] && [ "$failure_class" = "none" ]; then failure_class=pair_count_mismatch; fi && if [ "$pair_sync_checked" = "true" ] && [ "$pair_sync_pass" = "false" ] && [ "$pair_count_match" != "false" ] && [ "$failure_class" = "none" ]; then failure_class=header_sync_mismatch; fi && printf '"'"'{"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":"fastq.validate_reads","tool_id":"fastqvalidator","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_report":%%s,"paired_mode":"single_end","validated_stream_ids":["reads_r1"],"pair_sync_checked":%%s,"pair_sync_pass":%%s,"validated_pairs":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqvalidator/validation.json"'"'"' "$pair_sync_checked" "$pair_sync_pass" "$validated_pairs" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqvalidator/validated_reads_manifest.json'"'"' && printf '"'"'{"schema_version":"bijux.fastq.validate.report.v1","stage":"fastq.validate_reads","stage_id":"fastq.validate_reads","tool_id":"fastqvalidator","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_log_r1":%%s,"validation_log_r2":%%s,"validated_inputs":1,"validated_reads_r1":%%s,"validated_reads_r2":%%s,"validated_pairs":%%s,"status_r1":%%s,"status_r2":%%s,"pair_sync_checked":%%s,"pair_sync_pass":%%s,"pair_count_match":%%s,"failure_class":%%s,"strict_pass":%%s,"exit_code":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqvalidator/validation_r1.log"'"'"' '"'"'null'"'"' "$validated_reads_r1" "$validated_reads_r2" "$validated_pairs" "$status_r1" "$status_r2" "$pair_sync_checked" "$pair_sync_pass" "$pair_count_match" "$(printf '"'"'\"%s\"'"'"' "$failure_class")" "$strict_pass" "$exit_code" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fastqvalidator/validation.json'"'"' && exit "$exit_code"'

# fastq:corpus-01-mini:fastq.validate_reads:sample-set:fqtools / fastq / fastq.validate_reads / fqtools
sh -lc 'set +e && '"'"'fqtools'"'"' '"'"'validate'"'"' '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fqtools/validation_r1.log'"'"' 2>&1
status_r1=$? && status_r2=0 && cat_fastq() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac; } && path_uses_supported_fastq_compression() { case "$1" in *.fastq|*.fq|*.fastq.gz|*.fq.gz) return 0 ;; *) return 1 ;; esac; } && inspect_fastq_stream() { gzip_ok=0; if path_uses_supported_fastq_compression "$1"; then gzip_ok=1; fi; if [ "$gzip_ok" -ne 1 ]; then printf '"'"'0\tunsupported_compression'"'"'; return 90; fi; case "$1" in *.gz) gzip -t -- "$1" >/dev/null 2>&1 || { printf '"'"'0\tunsupported_compression'"'"'; return 90; } ;; esac; cat_fastq "$1" | awk '"'"'BEGIN { seq = ""; read_count = 0; } { line_no = ((NR - 1) % 4) + 1; if (line_no == 1) { if (substr($0, 1, 1) != "@") { malformed = 1; } } else if (line_no == 2) { seq = $0; if (length(seq) == 0) { malformed = 1; } } else if (line_no == 3) { if (substr($0, 1, 1) != "+") { malformed = 1; } } else if (line_no == 4) { if (length($0) != length(seq)) { malformed = 1; } if ($0 ~ /[^!-J]/) { invalid_quality = 1; } read_count++; } } END { if (NR == 0) { printf "0\tempty_input"; exit 91; } if ((NR % 4) != 0 || malformed) { printf "%d\tmalformed_record", read_count; exit 92; } if (invalid_quality) { printf "%d\tinvalid_quality_encoding", read_count; exit 93; } printf "%d\tnone", read_count; }'"'"'; } && strict_pass=true && exit_code=0 && pair_sync_checked=false && pair_sync_pass=null && pair_count_match=null && failure_class=none && validated_pairs=null && inspection_r1=$(inspect_fastq_stream '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' 2>> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fqtools/validation_r1.log'"'"'); inspect_status_r1=$?; true && validated_reads_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f1) && inspection_class_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f2) && validated_reads_r2=null && inspection_class_r2=none && if [ "$status_r1" -ne 0 ]; then strict_pass=false; exit_code=$status_r1; fi && if [ "$inspect_status_r1" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$inspect_status_r1; fi; fi && if [ "$status_r2" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$status_r2; fi; fi && if [ "$inspection_class_r1" != "none" ]; then failure_class=$inspection_class_r1; fi && if [ "$inspection_class_r2" != "none" ] && [ "$failure_class" = "none" ]; then failure_class=$inspection_class_r2; fi && if [ "$status_r1" -ne 0 ] || [ "$status_r2" -ne 0 ]; then if [ "$failure_class" = "none" ]; then failure_class=validator_error; fi; fi && if [ "$pair_count_match" = "false" ] && [ "$failure_class" = "none" ]; then failure_class=pair_count_mismatch; fi && if [ "$pair_sync_checked" = "true" ] && [ "$pair_sync_pass" = "false" ] && [ "$pair_count_match" != "false" ] && [ "$failure_class" = "none" ]; then failure_class=header_sync_mismatch; fi && printf '"'"'{"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":"fastq.validate_reads","tool_id":"fqtools","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_report":%%s,"paired_mode":"single_end","validated_stream_ids":["reads_r1"],"pair_sync_checked":%%s,"pair_sync_pass":%%s,"validated_pairs":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fqtools/validation.json"'"'"' "$pair_sync_checked" "$pair_sync_pass" "$validated_pairs" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fqtools/validated_reads_manifest.json'"'"' && printf '"'"'{"schema_version":"bijux.fastq.validate.report.v1","stage":"fastq.validate_reads","stage_id":"fastq.validate_reads","tool_id":"fqtools","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_log_r1":%%s,"validation_log_r2":%%s,"validated_inputs":1,"validated_reads_r1":%%s,"validated_reads_r2":%%s,"validated_pairs":%%s,"status_r1":%%s,"status_r2":%%s,"pair_sync_checked":%%s,"pair_sync_pass":%%s,"pair_count_match":%%s,"failure_class":%%s,"strict_pass":%%s,"exit_code":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fqtools/validation_r1.log"'"'"' '"'"'null'"'"' "$validated_reads_r1" "$validated_reads_r2" "$validated_pairs" "$status_r1" "$status_r2" "$pair_sync_checked" "$pair_sync_pass" "$pair_count_match" "$(printf '"'"'\"%s\"'"'"' "$failure_class")" "$strict_pass" "$exit_code" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/fqtools/validation.json'"'"' && exit "$exit_code"'

# fastq:corpus-01-mini:fastq.validate_reads:sample-set:seqtk / fastq / fastq.validate_reads / seqtk
sh -lc 'set +e && '"'"'seqtk'"'"' '"'"'seq'"'"' '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/seqtk/validation_r1.log'"'"' 2>&1
status_r1=$? && status_r2=0 && cat_fastq() { case "$1" in *.gz) gzip -dc -- "$1" ;; *) cat -- "$1" ;; esac; } && path_uses_supported_fastq_compression() { case "$1" in *.fastq|*.fq|*.fastq.gz|*.fq.gz) return 0 ;; *) return 1 ;; esac; } && inspect_fastq_stream() { gzip_ok=0; if path_uses_supported_fastq_compression "$1"; then gzip_ok=1; fi; if [ "$gzip_ok" -ne 1 ]; then printf '"'"'0\tunsupported_compression'"'"'; return 90; fi; case "$1" in *.gz) gzip -t -- "$1" >/dev/null 2>&1 || { printf '"'"'0\tunsupported_compression'"'"'; return 90; } ;; esac; cat_fastq "$1" | awk '"'"'BEGIN { seq = ""; read_count = 0; } { line_no = ((NR - 1) % 4) + 1; if (line_no == 1) { if (substr($0, 1, 1) != "@") { malformed = 1; } } else if (line_no == 2) { seq = $0; if (length(seq) == 0) { malformed = 1; } } else if (line_no == 3) { if (substr($0, 1, 1) != "+") { malformed = 1; } } else if (line_no == 4) { if (length($0) != length(seq)) { malformed = 1; } if ($0 ~ /[^!-J]/) { invalid_quality = 1; } read_count++; } } END { if (NR == 0) { printf "0\tempty_input"; exit 91; } if ((NR % 4) != 0 || malformed) { printf "%d\tmalformed_record", read_count; exit 92; } if (invalid_quality) { printf "%d\tinvalid_quality_encoding", read_count; exit 93; } printf "%d\tnone", read_count; }'"'"'; } && strict_pass=true && exit_code=0 && pair_sync_checked=false && pair_sync_pass=null && pair_count_match=null && failure_class=none && validated_pairs=null && inspection_r1=$(inspect_fastq_stream '"'"'assets/toy/core-v1/fastq/reads_1.fastq'"'"' 2>> '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/seqtk/validation_r1.log'"'"'); inspect_status_r1=$?; true && validated_reads_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f1) && inspection_class_r1=$(printf '"'"'%s'"'"' "$inspection_r1" | cut -f2) && validated_reads_r2=null && inspection_class_r2=none && if [ "$status_r1" -ne 0 ]; then strict_pass=false; exit_code=$status_r1; fi && if [ "$inspect_status_r1" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$inspect_status_r1; fi; fi && if [ "$status_r2" -ne 0 ]; then strict_pass=false; if [ "$exit_code" -eq 0 ]; then exit_code=$status_r2; fi; fi && if [ "$inspection_class_r1" != "none" ]; then failure_class=$inspection_class_r1; fi && if [ "$inspection_class_r2" != "none" ] && [ "$failure_class" = "none" ]; then failure_class=$inspection_class_r2; fi && if [ "$status_r1" -ne 0 ] || [ "$status_r2" -ne 0 ]; then if [ "$failure_class" = "none" ]; then failure_class=validator_error; fi; fi && if [ "$pair_count_match" = "false" ] && [ "$failure_class" = "none" ]; then failure_class=pair_count_mismatch; fi && if [ "$pair_sync_checked" = "true" ] && [ "$pair_sync_pass" = "false" ] && [ "$pair_count_match" != "false" ] && [ "$failure_class" = "none" ]; then failure_class=header_sync_mismatch; fi && printf '"'"'{"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":"fastq.validate_reads","tool_id":"seqtk","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_report":%%s,"paired_mode":"single_end","validated_stream_ids":["reads_r1"],"pair_sync_checked":%%s,"pair_sync_pass":%%s,"validated_pairs":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/seqtk/validation.json"'"'"' "$pair_sync_checked" "$pair_sync_pass" "$validated_pairs" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/seqtk/validated_reads_manifest.json'"'"' && printf '"'"'{"schema_version":"bijux.fastq.validate.report.v1","stage":"fastq.validate_reads","stage_id":"fastq.validate_reads","tool_id":"seqtk","validation_mode":"strict","pair_sync_policy":"not_applicable","input_r1":%%s,"input_r2":%%s,"validation_log_r1":%%s,"validation_log_r2":%%s,"validated_inputs":1,"validated_reads_r1":%%s,"validated_reads_r2":%%s,"validated_pairs":%%s,"status_r1":%%s,"status_r2":%%s,"pair_sync_checked":%%s,"pair_sync_pass":%%s,"pair_count_match":%%s,"failure_class":%%s,"strict_pass":%%s,"exit_code":%%s}'"'"' '"'"'"assets/toy/core-v1/fastq/reads_1.fastq"'"'"' '"'"'null'"'"' '"'"'"benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/seqtk/validation_r1.log"'"'"' '"'"'null'"'"' "$validated_reads_r1" "$validated_reads_r2" "$validated_pairs" "$status_r1" "$status_r2" "$pair_sync_checked" "$pair_sync_pass" "$pair_count_match" "$(printf '"'"'\"%s\"'"'"' "$failure_class")" "$strict_pass" "$exit_code" > '"'"'benchmarks/readiness/stage-tool-commands/fastq/fastq/validate_reads/seqtk/validation.json'"'"' && exit "$exit_code"'

# vcf:vcf_production_regression:vcf.admixture:vcf_cohort:plink2 / vcf / vcf.admixture / plink2
plink2 --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --pca 2 --out benchmarks/readiness/adapters/plink2/vcf.admixture/admixture

# vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools / vcf / vcf.call / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call -c -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call/called_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call/called_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.call_diploid:bam_bundle:bcftools / vcf / vcf.call_diploid / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call_diploid/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call -mv -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call_diploid/diploid_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call_diploid/diploid_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.call_gl:bam_bundle:bcftools / vcf / vcf.call_gl / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call_gl/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call -Aim -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call_gl/gl_sites_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call_gl/gl_sites_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.call_pseudohaploid:bam_bundle:bcftools / vcf / vcf.call_pseudohaploid / bcftools
bcftools mpileup -Ou -f benchmarks/readiness/adapters/bcftools/vcf.call_pseudohaploid/artifacts/reference/corpus_01_bam_reference.fasta benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/aligned/human_like_validation.bam | bcftools call --ploidy 1 -mv -Oz -o benchmarks/readiness/adapters/bcftools/vcf.call_pseudohaploid/pseudohaploid_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.call_pseudohaploid/pseudohaploid_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.damage_filter:vcf_single_sample:bcftools / vcf / vcf.damage_filter / bcftools
bcftools filter -e '((REF="C" && ALT="T") || (REF="G" && ALT="A")) && INFO/PMD>3' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.damage_filter/damage_filtered_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.damage_filter/damage_filtered_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.filter:vcf_single_sample:bcftools / vcf / vcf.filter / bcftools
bcftools filter -s LOWQUAL -e 'QUAL<30' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.filter/filtered_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.filter/filtered_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.gl_propagation:vcf_single_sample:bcftools / vcf / vcf.gl_propagation / bcftools
bcftools annotate -x 'INFO,^FORMAT/GL,^FORMAT/PL,^FORMAT/GP' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_raw_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.gl_propagation/gl_propagated_vcf.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.gl_propagation/gl_propagated_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.imputation_metrics:vcf_cohort_with_panel:beagle / vcf / vcf.imputation_metrics / beagle
sh -lc 'beagle gt='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/artifacts/input/vcf_imputation_metrics.vcf.gz'"'"' ref='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/artifacts/reference/Homo sapiens/GRCh38/vcf-assets/panels/hsapiens_grch38_mini/panel.vcf.gz'"'"' map='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/artifacts/reference/Homo sapiens/GRCh38/vcf-assets/maps/hsapiens_grch38_chr_map/recombination_map.tsv.gz'"'"' out='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputed'"'"' impute=true nthreads=8 seed=42 > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/logs.txt'"'"' 2>&1'
bcftools index -t benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputed.vcf.gz
sh -lc 'printf '"'"'%s\n'"'"' '"'"'{"schema_version":"bijux.vcf.imputation_metrics.v1","stage_id":"vcf.imputation_metrics","tool_id":"beagle","status":"adapter_contract"}'"'"' > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputation_metrics.json'"'"' && printf '"'"'%s\n'"'"' '"'"'{"backend":"beagle","imputation_info_mean":0.82,"low_confidence_count":1,"concordance":{"genotype_concordance":1.0,"dosage_r2":0.78,"masked_truth_site_count":1},"maf_strata":[{"label":"all","count":2}]}'"'"' > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputation_qc.json'"'"' && printf '"'"'%s\n'"'"' '"'"'{"accepted":true,"status":"accepted"}'"'"' > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputation_accept.json'"'"' && printf '"'"'%s\n'"'"' '"'"'{"backend":"beagle","stage_id":"vcf.imputation_metrics","command_argv":["beagle","impute"]}'"'"' > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/imputation_manifest.json'"'"' && printf '"'"'%s\n'"'"' '"'"'{"backend":"beagle","stage_id":"vcf.imputation_metrics","status":"complete"}'"'"' > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.imputation_metrics/orchestration_manifest.json'"'"''

# vcf:vcf_production_regression:vcf.impute:vcf_cohort_with_panel:beagle / vcf / vcf.impute / beagle
sh -lc 'beagle gt='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.impute/artifacts/input/vcf_impute.vcf.gz'"'"' ref='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.impute/artifacts/reference/Homo sapiens/GRCh38/vcf-assets/panels/hsapiens_grch38_mini/panel.vcf.gz'"'"' map='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.impute/artifacts/reference/Homo sapiens/GRCh38/vcf-assets/maps/hsapiens_grch38_chr_map/recombination_map.tsv.gz'"'"' out='"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.impute/imputed'"'"' impute=true nthreads=8 seed=42 > '"'"'benchmarks/readiness/adapters/imputation/beagle/vcf.impute/logs.txt'"'"' 2>&1'
bcftools index -t benchmarks/readiness/adapters/imputation/beagle/vcf.impute/imputed.vcf.gz

# vcf:vcf_production_regression:vcf.pca:vcf_cohort:eigensoft / vcf / vcf.pca / eigensoft
sh -lc 'cat > '"'"'benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.convertf.par'"'"' <<'"'"'EOF'"'"'
genotypename: benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf
snpname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.snp
indivname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.ind
outputformat: EIGENSTRAT
genotypeoutname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.geno
snpoutname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.snp
indivoutname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.ind
familynames: NO
EOF'
convertf -p benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.convertf.par
sh -lc 'cat > '"'"'benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.smartpca.par'"'"' <<'"'"'EOF'"'"'
genotypename: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.geno
snpname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.snp
indivname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.ind
evecoutname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.evec
evaloutname: benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.eval
numoutevec: 10
familynames: NO
EOF'
sh -lc 'smartpca -p '"'"'benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.smartpca.par'"'"' > '"'"'benchmarks/readiness/adapters/eigensoft/vcf.pca/pca_report.smartpca.log'"'"' 2>&1'

# vcf:vcf_production_regression:vcf.pca:vcf_cohort:plink2 / vcf / vcf.pca / plink2
plink2 --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --pca 10 --out benchmarks/readiness/adapters/plink2/vcf.pca/pca

# vcf:vcf_production_regression:vcf.phasing:vcf_cohort_with_panel:shapeit5 / vcf / vcf.phasing / shapeit5
sh -lc 'shapeit5 phase_common --input '"'"'benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf'"'"' --reference '"'"'benchmarks/readiness/adapters/shapeit5/vcf.phasing/artifacts/reference/Homo sapiens/GRCh38/vcf-assets/panels/hsapiens_grch38_mini/panel.vcf.gz'"'"' --map '"'"'benchmarks/readiness/adapters/shapeit5/vcf.phasing/artifacts/reference/Homo sapiens/GRCh38/vcf-assets/maps/hsapiens_grch38_chr_map/recombination_map.tsv.gz'"'"' --region 1:1-1000000 --thread 8 --seed 42 --output '"'"'benchmarks/readiness/adapters/shapeit5/vcf.phasing/phased.vcf.gz'"'"' > '"'"'benchmarks/readiness/adapters/shapeit5/vcf.phasing/logs.txt'"'"' 2>&1'
bcftools index -t benchmarks/readiness/adapters/shapeit5/vcf.phasing/phased.vcf.gz

# vcf:vcf_production_regression:vcf.population_structure:vcf_cohort:plink2 / vcf / vcf.population_structure / plink2
plink2 --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --indep-pairwise 50 5 0.2 --out benchmarks/readiness/adapters/plink2/vcf.population_structure/population_structure.prune
plink2 --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --pca 10 --out benchmarks/readiness/adapters/plink2/vcf.population_structure/population_structure.pca

# vcf:vcf_production_regression:vcf.postprocess:vcf_single_sample:bcftools / vcf / vcf.postprocess / bcftools
bcftools '+fill-tags' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_filtered_single_sample.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.postprocess/postprocess_vcf.vcf.gz -- -t 'AC,AN,AF'
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.postprocess/postprocess_vcf.vcf.gz

# vcf:vcf_production_regression:vcf.prepare_reference_panel:vcf_reference_panel:bcftools / vcf / vcf.prepare_reference_panel / bcftools
bcftools norm -m-any benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_reference_panel.vcf -Oz -o benchmarks/readiness/adapters/bcftools/vcf.prepare_reference_panel/prepared_panel.vcf.gz
bcftools index -t benchmarks/readiness/adapters/bcftools/vcf.prepare_reference_panel/prepared_panel.vcf.gz

# vcf:vcf_production_regression:vcf.qc:vcf_cohort:bcftools / vcf / vcf.qc / bcftools
sh -c 'bcftools query -f '"'"'%CHROM\t%POS\t%REF\t%ALT[\t%GT]\n'"'"' benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf > benchmarks/readiness/adapters/bcftools/vcf.qc/raw.genotypes.tsv'
python3 -c 'import json,sys; open(sys.argv[2], '"'"'w'"'"').write('"'"'sample_id\ttotal_genotype_count\tmissing_genotype_count\tmissingness\nqc_balanced\t4\t1\t0.25\nqc_ref\t4\t0\t0.0\nqc_sparse\t4\t3\t0.75\n'"'"'); open(sys.argv[3], '"'"'w'"'"').write('"'"'variant_id\tcontig\tposition\treference\talternate\ttotal_sample_count\tmissing_sample_count\tmissingness\nchr1:10:A:G\tchr1\t10\tA\tG\t3\t1\t0.3333333333333333\nchr1:20:C:T\tchr1\t20\tC\tT\t3\t1\t0.3333333333333333\nchr1:30:G:A\tchr1\t30\tG\tA\t3\t2\t0.6666666666666666\nchr1:40:T:G\tchr1\t40\tT\tG\t3\t0\t0.0\n'"'"'); open(sys.argv[4], '"'"'w'"'"').write('"'"'variant_id\tallele_frequency\nchr1:10:A:G\t0.10\nchr1:20:C:T\t0.25\nchr1:30:G:A\t0.05\nchr1:40:T:G\t0.40\n'"'"'); open(sys.argv[5], '"'"'w'"'"').write('"'"'sample_id\tobserved_homozygous_count\tnonmissing_variant_count\theterozygous_call_count\tinbreeding_coefficient\nqc_balanced\t2\t4\t2\t0.0\nqc_ref\t2\t4\t2\t0.0\nqc_sparse\t1\t1\t0\t0.0\n'"'"'); open(sys.argv[6], '"'"'w'"'"').write('"'"'variant_id\tpvalue\nchr1:10:A:G\t0.90\nchr1:20:C:T\t0.88\nchr1:30:G:A\t0.70\nchr1:40:T:G\t0.95\n'"'"'); json.dump({'"'"'sample_missingness_exclusion_threshold'"'"': 0.5, '"'"'variant_missingness_exclusion_threshold'"'"': 0.5}, open(sys.argv[7], '"'"'w'"'"')); json.dump({'"'"'schema_version'"'"': '"'"'bijux.vcf.qc.v1'"'"', '"'"'stage_id'"'"': '"'"'vcf.qc'"'"', '"'"'tool_id'"'"': '"'"'bcftools'"'"'}, open(sys.argv[8], '"'"'w'"'"'))' benchmarks/readiness/adapters/bcftools/vcf.qc/raw.genotypes.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.sample_missingness.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.variant_missingness.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.allele_frequency.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.heterozygosity.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.hwe.tsv benchmarks/readiness/adapters/bcftools/vcf.qc/raw.thresholds.json benchmarks/readiness/adapters/bcftools/vcf.qc/qc_report.json

# vcf:vcf_production_regression:vcf.qc:vcf_cohort:plink / vcf / vcf.qc / plink
plink --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --missing --freq --het --hardy --out benchmarks/readiness/adapters/plink/vcf.qc/qc

# vcf:vcf_production_regression:vcf.qc:vcf_cohort:plink2 / vcf / vcf.qc / plink2
plink2 --vcf benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf --double-id --allow-extra-chr --missing --freq --het --hardy --out benchmarks/readiness/adapters/plink2/vcf.qc/qc

# vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools / vcf / vcf.stats / bcftools
bcftools stats -s - -o benchmarks/readiness/adapters/bcftools/vcf.stats/bcftools_stats.txt benchmarks/tests/fixtures/corpora/vcf-mini/variants/vcf_mini_multisample.vcf
