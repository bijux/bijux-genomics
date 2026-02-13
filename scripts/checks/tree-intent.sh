#!/usr/bin/env bash
set -euo pipefail
LC_ALL=C
export LC_ALL

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

dirs=(assets bin configs containers crates docs domain examples makefiles scripts)

echo "root directories and intent"
for d in "${dirs[@]}"; do
  if [[ ! -d "${d}" ]]; then
    continue
  fi
  desc=""
  for f in "${d}/index.md" "${d}/README.md"; do
    if [[ -f "${f}" ]]; then
      desc="$(awk 'NF{print; exit}' "${f}" | sed 's/^#\+ *//')"
      break
    fi
  done
  if [[ -z "${desc}" ]]; then
    desc="(no index.md/README.md summary)"
  fi
  printf '%-12s %s\n' "${d}" "${desc}"
done
