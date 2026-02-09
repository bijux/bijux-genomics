#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
REGISTRY_FILE="${REGISTRY_FILE:-$ROOT_DIR/configs/tools.toml}"

if [ ! -f "$REGISTRY_FILE" ]; then
  echo "missing registry: $REGISTRY_FILE" >&2
  exit 2
fi

cmd="${1:-}"
case "$cmd" in
  stage-tools)
    stage_id="${2:-}"
    field="${3:-all}"
    if [ -z "$stage_id" ]; then
      echo "usage: $0 stage-tools <stage-id> [all|primary|optional|validation|reporting]" >&2
      exit 2
    fi
    awk -v sid="$stage_id" -v mode="$field" '
      function trim(s){ gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
      function add_item(x){
        x=trim(x); gsub(/^"/, "", x); gsub(/"$/, "", x)
        if (x != "" && !(x in seen)) { seen[x]=1; ord[++ord_n]=x }
      }
      function parse_and_add(line, arr, n, i){
        sub(/^[^=]*=/, "", line)
        line=trim(line)
        gsub(/^\[/, "", line); gsub(/\]$/, "", line)
        n=split(line, arr, ",")
        for (i=1; i<=n; i++) add_item(arr[i])
      }
      /^\[\[stages\]\]/ { in_stage=1; id=""; next }
      in_stage && /^[[:space:]]*id[[:space:]]*=/ {
        split($0,a,"="); id=trim(a[2]); gsub(/^"/, "", id); gsub(/"$/, "", id); next
      }
      in_stage && id==sid && /^[[:space:]]*primary_tools[[:space:]]*=/ {
        if (mode=="all" || mode=="primary") parse_and_add($0); next
      }
      in_stage && id==sid && /^[[:space:]]*optional_alternatives[[:space:]]*=/ {
        if (mode=="all" || mode=="optional") parse_and_add($0); next
      }
      in_stage && id==sid && /^[[:space:]]*validation_tools[[:space:]]*=/ {
        if (mode=="all" || mode=="validation") parse_and_add($0); next
      }
      in_stage && id==sid && /^[[:space:]]*reporting_tools[[:space:]]*=/ {
        if (mode=="all" || mode=="reporting") parse_and_add($0); next
      }
      END {
        for (i=1; i<=ord_n; i++) { printf "%s", ord[i]; if (i<ord_n) printf "," }
        printf "\n"
      }
    ' "$REGISTRY_FILE"
    ;;
  tools-by-runtime)
    runtime="${2:-}"
    if [ -z "$runtime" ]; then
      echo "usage: $0 tools-by-runtime <docker|apptainer>" >&2
      exit 2
    fi
    awk -v rt="$runtime" '
      function trim(s){ gsub(/^[[:space:]]+|[[:space:]]+$/, "", s); return s }
      /^\[\[tools\]\]/{ id=""; runtime_hit=0; container=1; next }
      /^[[:space:]]*id[[:space:]]*=/ { split($0,a,"="); id=trim(a[2]); gsub(/^"|"$/, "", id); next }
      /^[[:space:]]*container[[:space:]]*=/ { split($0,a,"="); container=trim(a[2]); next }
      /^[[:space:]]*runtimes[[:space:]]*=/ {
        line=$0
        if(line ~ rt){ runtime_hit=1 }
        next
      }
      /^$/ {
        if(id!="" && runtime_hit==1 && container!="false") print id
      }
      END { if(id!="" && runtime_hit==1 && container!="false") print id }
    ' "$REGISTRY_FILE" | awk '!seen[$0]++' | paste -sd, -
    ;;
  *)
    echo "unknown command: $cmd" >&2
    exit 2
    ;;
esac
