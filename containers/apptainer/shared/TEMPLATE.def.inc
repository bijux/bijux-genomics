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
# 6) OCI labels are the canonical provenance surface; do not duplicate metadata in /opt/bijux
# 7) %runscript uses exec <tool> "$@" and predictable help behavior
