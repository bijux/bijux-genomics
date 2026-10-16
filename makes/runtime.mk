# Prefer repository-provisioned test tools without changing the caller's home.
# Runtime preflight remains responsible for checking the selected tool versions.
BIJUX_TEST_RUNTIME_ROOT ?= $(abspath $(ARTIFACT_ROOT)/runtime)

_bijux_bcftools_bin := $(abspath $(BIJUX_TEST_RUNTIME_ROOT)/bcftools/install/bin)
_bijux_bash_bin := $(abspath $(BIJUX_TEST_RUNTIME_ROOT)/bash/install/bin)
_bijux_checks_python := $(abspath $(ARTIFACT_ROOT)/python/checks-environment/bin/python3)
_bijux_shell_quote = '$(subst ','"'"',$(1))'

_bijux_bcftools_available := $(shell test -x $(call _bijux_shell_quote,$(_bijux_bcftools_bin)/bcftools) && printf yes)
_bijux_bash_available := $(shell test -x $(call _bijux_shell_quote,$(_bijux_bash_bin)/bash) && printf yes)

# Remove only the selected directories before prepending them. This keeps nested
# Make invocations idempotent and preserves spaces and empty entries in PATH.
define _bijux_runtime_path_script
 remaining=$(call _bijux_shell_quote,$(PATH)); result=; separator=; \
 while :; do \
   entry=$${remaining%%:*}; selected=; \
   if [ "$(_bijux_bcftools_available)" = yes ] && [ "$$entry" = $(call _bijux_shell_quote,$(_bijux_bcftools_bin)) ]; then selected=yes; fi; \
   if [ "$(_bijux_bash_available)" = yes ] && [ "$$entry" = $(call _bijux_shell_quote,$(_bijux_bash_bin)) ]; then selected=yes; fi; \
   if [ "$$selected" != yes ]; then result="$$result$$separator$$entry"; separator=:; fi; \
   case "$$remaining" in *:*) remaining=$${remaining#*:} ;; *) break ;; esac; \
 done; \
 prefix=; \
 if [ "$(_bijux_bcftools_available)" = yes ]; then prefix=$(call _bijux_shell_quote,$(_bijux_bcftools_bin)); fi; \
 if [ "$(_bijux_bash_available)" = yes ]; then prefix="$${prefix:+$$prefix:}"$(call _bijux_shell_quote,$(_bijux_bash_bin)); fi; \
 if [ "$$separator" = : ] && [ -n "$$prefix" ]; then prefix="$$prefix:"; fi; \
 printf '%s' "$$prefix$$result"
endef
_bijux_runtime_path := $(shell $(_bijux_runtime_path_script))
override PATH := $(_bijux_runtime_path)
export PATH

ifeq ($(origin PYTHON),undefined)
PYTHON := $(if $(shell test -x $(call _bijux_shell_quote,$(_bijux_checks_python)) && printf yes),$(_bijux_checks_python),python3)
endif
export PYTHON

ifeq ($(origin BIJUX_GATE_TEST_BASH),undefined)
ifeq ($(_bijux_bash_available),yes)
BIJUX_GATE_TEST_BASH := $(_bijux_bash_bin)/bash
export BIJUX_GATE_TEST_BASH
endif
endif
