# BIJUX_TEMPLATE: v1
# Required blocks for bijux-owned Apptainer definitions:
# 1) four-line repository license header
# 2) %labels with OCI source/revision/created/licenses/version/tool/title
# 3) %post writes provenance file to /opt/bijux/VERSION.json
# 4) %runscript uses exec <tool> "$@"
