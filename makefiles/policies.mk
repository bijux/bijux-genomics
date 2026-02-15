SHELL := /bin/sh

# Guardrail culprits helpers (read-only diagnostics).
# Expected empty output when constraints are satisfied.

culprits-max_loc:
	@find crates -name "*.rs" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^crates\// && $$1 > 1000'

culprits-max_depth:
	@find crates -name "*.rs" -print0 \
	| xargs -0 -I{} sh -c 'p="{}"; d=$$(printf "%s\n" "$$p" | awk -F/ "{print NF}"); echo "$$d $$p"' \
	| sort -n \
	| awk '$$1 > 7'

culprits-file-max_rs_files_per_dir:
	@find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 10' \
	| sort -nr

culprits-file-max_modules_per_dir:
	@find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 16' \
	| sort -nr

culprits-all: culprits-max_loc culprits-max_depth culprits-file-max_rs_files_per_dir culprits-file-max_modules_per_dir
	@:

.PHONY: culprits-all culprits-max_loc culprits-max_depth culprits-file-max_rs_files_per_dir culprits-file-max_modules_per_dir
