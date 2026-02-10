#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
TMP_DIR=${TEST_TMP_DIR:-"$ROOT_DIR/artifacts/tmp"}
mkdir -p "$TMP_DIR"

DOM_TOOLS="$TMP_DIR/domain_tools.txt"
REG_TOOLS="$TMP_DIR/registry_tools.txt"
CODE_TOOLS="$TMP_DIR/code_tools.txt"
MAKE_TOOLS="$TMP_DIR/make_tools.txt"

awk -F'"' '/^tool_id:/{print $2}' "$ROOT_DIR"/domain/fastq/tools/*.yaml "$ROOT_DIR"/domain/bam/tools/*.yaml \
  | sort -u > "$DOM_TOOLS"

# Registry tools that are wired into FASTQ/BAM stages.
awk '
BEGIN{instage=0}
/^\[\[stages\]\]/{instage=1; next}
/^\[\[/{if($0 !~ /^\[\[stages\]\]/) instage=0}
instage && ($0 ~ /^primary_tools = / || $0 ~ /^optional_alternatives = / || $0 ~ /^validation_tools = / || $0 ~ /^reporting_tools = /) {
  line=$0
  while (match(line, /"[a-zA-Z0-9_\-]+"/)) {
    token=substr(line, RSTART+1, RLENGTH-2)
    print token
    line=substr(line, RSTART+RLENGTH)
  }
}
' "$ROOT_DIR/configs/tools.toml" | sort -u > "$REG_TOOLS"

rg -No 'ToolId::from_static\("([a-z0-9_\-]+)"\)' "$ROOT_DIR/crates" \
  | sed -E 's/.*from_static\("([a-z0-9_\-]+)"\).*/\1/' \
  | grep -Ev '^(tool|planner|unknown)$' \
  | sort -u > "$CODE_TOOLS" || :

# Resolve tools indirectly referenced by makefiles via stage-tools calls.
STAGES_FILE="$TMP_DIR/make_stage_ids.txt"
rg -No 'stage-tools ([a-z0-9._-]+) all' "$ROOT_DIR/makefiles" \
  | sed -E 's/.*stage-tools ([a-z0-9._-]+) all.*/\1/' \
  | sort -u > "$STAGES_FILE" || :

> "$MAKE_TOOLS"
while IFS= read -r stage_id; do
  [ -z "$stage_id" ] && continue
  "$ROOT_DIR/scripts/registry-tools.sh" stage-tools "$stage_id" all | tr ',' '\n' >> "$MAKE_TOOLS" || :
done < "$STAGES_FILE"

sed -i.bak '/^$/d' "$MAKE_TOOLS" 2>/dev/null || true
rm -f "$MAKE_TOOLS.bak"
sort -u "$MAKE_TOOLS" -o "$MAKE_TOOLS"

report_diff() {
  left="$1"
  right="$2"
  title="$3"
  missing=$(comm -23 "$left" "$right" || true)
  if [ -n "$missing" ]; then
    echo "[DIFF] $title"
    echo "$missing" | sed 's/^/  - /'
    return 1
  fi
  return 0
}

ok=0
report_diff "$DOM_TOOLS" "$REG_TOOLS" "domain tools missing from registry" || ok=1
report_diff "$CODE_TOOLS" "$REG_TOOLS" "code-referenced tools missing from registry" || ok=1
report_diff "$MAKE_TOOLS" "$REG_TOOLS" "make-referenced tools missing from registry" || ok=1
report_diff "$REG_TOOLS" "$DOM_TOOLS" "registry tools missing from domain" || ok=1

echo "--- inventory counts ---"
echo "domain:   $(wc -l < "$DOM_TOOLS" | tr -d ' ')"
echo "registry: $(wc -l < "$REG_TOOLS" | tr -d ' ')"
echo "code:     $(wc -l < "$CODE_TOOLS" | tr -d ' ')"
echo "make:     $(wc -l < "$MAKE_TOOLS" | tr -d ' ')"

if [ "$ok" -ne 0 ]; then
  echo "domain-inventory-drift: mismatch detected" >&2
  exit 1
fi

echo "domain-inventory-drift: OK"
