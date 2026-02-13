#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

docs_root="${1:-artifacts/docs}"
rm -rf "${docs_root}"
