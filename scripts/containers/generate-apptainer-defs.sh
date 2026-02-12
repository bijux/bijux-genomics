#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
APPTAINER_ROOT="$ROOT_DIR/containers/apptainer"

normalize_def() {
  file="$1"
  tmp=$(mktemp "${TMPDIR:-/tmp}/apptainer-def.XXXXXX")
  trap 'rm -f "$tmp"' EXIT INT TERM

  awk '
    BEGIN { has_help = 0 }
    { print $0 }
    /^%help[[:space:]]*$/ { has_help = 1 }
    END {
      if (has_help == 0) {
        print ""
        print "%help"
        print "    Apptainer image generated under bijux container contracts."
        print "    Runscript executes tool entrypoint with argument passthrough."
      }
    }
  ' "$file" > "$tmp"

  # Keep deterministic trailing newline and atomic replacement.
  mv "$tmp" "$file"
  trap - EXIT INT TERM
}

find "$APPTAINER_ROOT" -type f -name '*.def' | sort | while IFS= read -r file; do
  normalize_def "$file"
done

echo "generated apptainer defs: ok"
