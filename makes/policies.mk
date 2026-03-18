SHELL := /bin/sh

# Guardrail culprits helpers (read-only diagnostics).
# Expected empty output when constraints are satisfied.

culprits-max_loc:
	@err=$$(find crates -name "*.rs" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^crates\// && $$1 > 1000'); \
	warn=$$(find crates -name "*.rs" -print0 \
	| xargs -0 wc -l \
	| sort -n \
	| awk '$$2 ~ /^crates\// && $$1 > 800 && $$1 <= 1000'); \
	if [ -n "$$err" ]; then \
		printf '%s\n' "ERROR: max_loc policy violations (LOC > 1000):"; \
		printf '%s\n' "$$err"; \
		exit 1; \
	fi; \
	if [ -n "$$warn" ]; then \
		printf '%s\n' "WARN: max_loc advisory violations (800 < LOC <= 1000):"; \
		printf '%s\n' "$$warn"; \
	else \
		printf '%s\n' "INFO: max_loc policy compliant across all crates."; \
	fi

culprits-max_depth:
	@out=$$(find crates -name "*.rs" -print0 \
	| xargs -0 -I{} sh -c 'p="{}"; d=$$(printf "%s\n" "$$p" | awk -F/ "{print NF}"); echo "$$d $$p"' \
	| sort -n \
	| awk '$$1 > 7'); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: max_depth policy violations (depth > 7):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: max_depth policy compliant across all crates."; \
	fi

culprits-file-max_rs_files_per_dir:
	@out=$$(find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 10' \
	| sort -nr); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: max_rs_files_per_dir policy violations (files > 10):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: max_rs_files_per_dir policy compliant across all crates."; \
	fi

culprits-file-max_modules_per_dir:
	@out=$$(find crates -name "*.rs" -print0 \
	| xargs -0 -n1 dirname \
	| sort \
	| uniq -c \
	| awk '$$1 > 16' \
	| sort -nr); \
	if [ -n "$$out" ]; then \
		printf '%s\n' "ERROR: max_modules_per_dir policy violations (modules > 16):"; \
		printf '%s\n' "$$out"; \
		exit 1; \
	else \
		printf '%s\n' "INFO: max_modules_per_dir policy compliant across all crates."; \
	fi

culprits-all: culprits-max_loc culprits-max_depth culprits-file-max_rs_files_per_dir culprits-file-max_modules_per_dir
	@printf '%s\n' "INFO: culprits-all completed."

.PHONY: culprits-all culprits-max_loc culprits-max_depth culprits-file-max_rs_files_per_dir culprits-file-max_modules_per_dir
