SHELL := /bin/sh

##@ Policies

policy-fast: ## Run fast policy checks (no snapshots)
	cargo test -p bijux-policies --test dependency_graph --test purity_scans --test core_layering --test domain_dependency_policy --test ci_tools_policy --test dev_deps_policy --test heavy_deps_policy

policy-full: ## Run full policy suite
	cargo test -p bijux-policies
	$(MAKE) docs-lint

.PHONY: policy-fast policy-full
