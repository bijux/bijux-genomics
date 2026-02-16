# BIJUX_TEMPLATE: v1
# Required blocks for bijux-owned Apptainer definitions:
# 1) four-line repository license header
# 2) standard header markers:
#    - Tool ID
#    - Version policy
#    - Upstream source
#    - Build date policy
# 3) %labels with OCI source/revision/created/licenses/version/tool/title
# 4) %environment exports PATH + LC_ALL + TZ
# 5) %post begins with `set -eux` and includes cleanup
# 6) %post writes provenance file to /opt/bijux/VERSION.json when feasible
# 7) %runscript uses exec <tool> "$@" and predictable help behavior
