#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

DOCS_PY="${DOCS_PY:-python3}"
DOCS_VENV="${DOCS_VENV:-artifacts/docs/.venv}"
DOCS_REQ="${DOCS_REQ:-scripts/docs/requirements.txt}"

"${DOCS_PY}" -m venv "${DOCS_VENV}"
"${DOCS_VENV}/bin/pip" install --upgrade pip
"${DOCS_VENV}/bin/pip" install -r "${DOCS_REQ}"
