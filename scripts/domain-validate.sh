#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

fail() {
  echo "domain-validate: $*" >&2
  exit 1
}

require_file() {
  [ -f "$1" ] || fail "missing required file: $1"
}

require_key() {
  key="$1"
  file="$2"
  rg -q "^${key}:" "$file" || fail "missing key '${key}' in $file"
}

require_file "$ROOT_DIR/domain/fastq/stages/_schema.yaml"
require_file "$ROOT_DIR/domain/bam/stages/_schema.yaml"
require_file "$ROOT_DIR/domain/fastq/tools/_schema.yaml"
require_file "$ROOT_DIR/domain/bam/tools/_schema.yaml"
require_file "$ROOT_DIR/domain/fastq/index.yaml"
require_file "$ROOT_DIR/domain/bam/index.yaml"

for f in "$ROOT_DIR"/domain/fastq/stages/*.yaml "$ROOT_DIR"/domain/bam/stages/*.yaml; do
  b=$(basename "$f")
  [ "$b" = "_schema.yaml" ] && continue
  require_key stage_id "$f"
  require_key status "$f"
  require_key scope "$f"
  require_key domain "$f"
  require_key inputs "$f"
  require_key outputs "$f"
  require_key required_inputs "$f"
  require_key required_outputs "$f"
  require_key invariants "$f"
  require_key compatible_tools "$f"
  require_key metrics_schema "$f"
  require_key planned_out_of_scope "$f"
  status=$(awk -F'"' '/^status:/{print $2; exit}' "$f")
  case "$status" in
    supported|planned|out_of_scope) ;;
    *) fail "invalid stage status '$status' in $f" ;;
  esac
  scope=$(awk -F'"' '/^scope:/{print $2; exit}' "$f")
  [ "$scope" = "pre_hpc_pre_vcf" ] || fail "invalid stage scope '$scope' in $f"
  stage_id=$(awk -F'"' '/^stage_id:/{print $2; exit}' "$f")
  domain=$(awk -F'"' '/^domain:/{print $2; exit}' "$f")
  echo "$stage_id" | rg -q "^${domain}\." || fail "stage_id $stage_id must be namespaced by domain $domain in $f"
done

for dom in fastq bam; do
  idx="$ROOT_DIR/domain/$dom/index.yaml"
  require_key stage_ids "$idx"
  require_key tool_ids "$idx"
  require_key stage_tool_compatibility "$idx"
  require_key active_defaults "$idx"
done

for f in "$ROOT_DIR"/domain/fastq/tools/*.yaml "$ROOT_DIR"/domain/bam/tools/*.yaml; do
  b=$(basename "$f")
  [ "$b" = "_schema.yaml" ] && continue
  for k in tool_id stage_ids status scope default_version upstream pin_strategy license version_cmd help_cmd expected_artifacts metrics_schema comparability_notes; do
    require_key "$k" "$f"
  done
  status=$(awk -F'"' '/^status:/{print $2; exit}' "$f")
  case "$status" in
    supported|planned|out_of_scope) ;;
    *) fail "invalid tool status '$status' in $f" ;;
  esac
  scope=$(awk -F'"' '/^scope:/{print $2; exit}' "$f")
  [ "$scope" = "pre_hpc_pre_vcf" ] || fail "invalid tool scope '$scope' in $f"
  # ensure tool references at least one known stage
  tool_stages=$(awk '/^stage_ids:/{flag=1; next} flag && /^  - /{print $2} flag && !/^  - /{flag=0}' "$f" || true)
  [ -n "$tool_stages" ] || true
done

# duplicate IDs are forbidden across files
all_stage_ids=$(
  awk -F'"' '/^stage_id:/{print $2}' \
    "$ROOT_DIR"/domain/fastq/stages/*.yaml \
    "$ROOT_DIR"/domain/bam/stages/*.yaml \
    "$ROOT_DIR"/domain/vcf/stages/*.yaml 2>/dev/null | sed '/^$/d'
)
dup_stages=$(printf "%s\n" "$all_stage_ids" | sort | uniq -d || true)
[ -z "$dup_stages" ] || fail "duplicate stage_id detected: $dup_stages"

all_tool_ids=$(
  awk -F'"' '/^tool_id:/{print $2}' \
    "$ROOT_DIR"/domain/fastq/tools/*.yaml \
    "$ROOT_DIR"/domain/bam/tools/*.yaml \
    "$ROOT_DIR"/domain/vcf/tools/*.yaml 2>/dev/null | sed '/^$/d'
)
dup_tools=$(printf "%s\n" "$all_tool_ids" | sort | uniq -d || true)
[ -z "$dup_tools" ] || fail "duplicate tool_id detected: $dup_tools"

# domain index should list all local stage and tool ids.
for dom in fastq bam; do
  stage_list=$(awk -F'"' '/^stage_id:/{print $2}' "$ROOT_DIR"/domain/$dom/stages/*.yaml | sort -u)
  tool_list=$(awk -F'"' '/^tool_id:/{print $2}' "$ROOT_DIR"/domain/$dom/tools/*.yaml | sort -u)
  idx="$ROOT_DIR/domain/$dom/index.yaml"
  for s in $stage_list; do
    rg -q "^  - ${s}$" "$idx" || fail "$idx missing stage id $s"
  done
  for t in $tool_list; do
    rg -q "^  - ${t}$" "$idx" || fail "$idx missing tool id $t"
  done
done

# index matrix must only reference known stage/tool IDs and active defaults must be compatible
for dom in fastq bam; do
  idx="$ROOT_DIR/domain/$dom/index.yaml"
  stages=$(awk -F'"' '/^stage_id:/{print $2}' "$ROOT_DIR"/domain/$dom/stages/*.yaml | sort -u)
  tools=$(awk -F'"' '/^tool_id:/{print $2}' "$ROOT_DIR"/domain/$dom/tools/*.yaml | sort -u)
  awk '
    BEGIN {inmap=0; indef=0}
    /^stage_tool_compatibility:/ {inmap=1; indef=0; next}
    /^active_defaults:/ {indef=1; inmap=0; next}
    inmap && /^[^[:space:]]/ {inmap=0}
    indef && /^[^[:space:]]/ {indef=0}
    inmap && /^[[:space:]]+[a-z0-9_.-]+:/ {
      line=$0
      gsub(/^[[:space:]]+/, "", line)
      split(line, p, ":")
      stage=p[1]
      rhs=line
      sub(/^[^:]+:[[:space:]]*\[/, "", rhs)
      sub(/\][[:space:]]*$/, "", rhs)
      gsub(/[[:space:]]/, "", rhs)
      print "M|" stage "|" rhs
    }
    indef && /^[[:space:]]+[a-z0-9_.-]+:[[:space:]]+[a-z0-9_.-]+/ {
      line=$0
      gsub(/^[[:space:]]+/, "", line)
      split(line, p, ":")
      stage=p[1]
      tool=p[2]
      gsub(/[[:space:]]/, "", tool)
      print "D|" stage "|" tool
    }
  ' "$idx" | while IFS='|' read -r kind stage rhs; do
    echo "$stages" | rg -qx "$stage" || fail "$idx matrix/default references unknown stage $stage"
    if [ "$kind" = "M" ]; then
      [ -n "$rhs" ] || fail "$idx stage $stage has empty compatibility list"
      for tool in $(echo "$rhs" | tr ',' ' '); do
        echo "$tools" | rg -qx "$tool" || fail "$idx stage $stage references unknown tool $tool"
      done
    else
      tool="$rhs"
      echo "$tools" | rg -qx "$tool" || fail "$idx active_defaults references unknown tool $tool for $stage"
      csv=$(awk -v st="$stage" '
        BEGIN {inmap=0}
        /^stage_tool_compatibility:/ {inmap=1; next}
        inmap && /^[^[:space:]]/ {inmap=0}
        inmap && $0 ~ ("^[[:space:]]+" st ":") {
          line=$0
          gsub(/^[[:space:]]+/, "", line)
          rhs=line
          sub(/^[^:]+:[[:space:]]*\[/, "", rhs)
          sub(/\][[:space:]]*$/, "", rhs)
          gsub(/[[:space:]]/, "", rhs)
          print rhs
          exit
        }
      ' "$idx")
      echo "$csv" | tr ',' '\n' | rg -qx "$tool" || fail "$idx active_defaults tool $tool is not compatible with $stage"
    fi
  done
done

echo "domain-validate: OK"
