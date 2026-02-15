SHELL := /bin/sh

# Guardrail reveal helpers (read-only diagnostics).
# Expected empty output when constraints are satisfied.

reveal-max_loc:
	@find crates -name "*.rs" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^crates\// && $$1 > 1000'

reveal-max_depth:
	@find crates -name "*.rs" -print0 \
	| xargs -0 -I{} sh -c 'p="{}"; d=$$(printf "%s\n" "$$p" | awk -F/ "{print NF}"); echo "$$d $$p"' \
	| sort -n \
	| awk '$$1 > 7'

reveal-file-max_rs_files_per_dir:
	@find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 10' \
	| sort -nr

reveal-file-max_modules_per_dir:
	@find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 16' \
	| sort -nr

reveal-all: reveal-max_loc reveal-max_depth reveal-file-max_rs_files_per_dir reveal-file-max_modules_per_dir
	@:

.PHONY: reveal-all reveal-max_loc reveal-max_depth reveal-file-max_rs_files_per_dir reveal-file-max_modules_per_dir
