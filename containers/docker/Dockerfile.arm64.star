# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 Bijan Mousavi
# VERSION_SOURCE: containers/contracts/catalog.toml
FROM ubuntu:24.04@sha256:cd1dba651b3080c3686ecf4e3c4220f026b521fb76978881737d24f200828b2b
COPY --chmod=0555 containers/contracts/configure-apt-snapshot /usr/local/sbin/bijux-configure-apt-snapshot

ARG OCI_CREATED=governed-by-build-evidence
ARG TOOL_VERSION=2.7.11b
LABEL org.opencontainers.image.source="https://github.com/alexdobin/STAR" \
      org.opencontainers.image.revision="b1edc1208d91a53bf40ebae8669f71d50b994851" \
      org.opencontainers.image.created="${OCI_CREATED}" \
      org.opencontainers.image.licenses="GPL-3.0" \
      org.opencontainers.image.version="${TOOL_VERSION}" \
      io.bijux.tool.version="${TOOL_VERSION}" \
      org.opencontainers.image.tool="star" \
      org.opencontainers.image.title="STAR" \
      org.opencontainers.image.base.name="ubuntu:24.04@sha256:cd1dba651b3080c3686ecf4e3c4220f026b521fb76978881737d24f200828b2b" \
      org.opencontainers.image.base.digest="sha256:cd1dba651b3080c3686ecf4e3c4220f026b521fb76978881737d24f200828b2b"

ENV LANG=C LC_ALL=C TZ=UTC DEBIAN_FRONTEND=noninteractive
RUN /usr/local/sbin/bijux-configure-apt-snapshot && \
    apt-get install -y --no-install-recommends libbz2-1.0 libgomp1 zlib1g && \
    apt-get install -y --no-install-recommends curl g++ make xxd libbz2-dev zlib1g-dev && \
    curl --fail --location --proto '=https' --tlsv1.2 \
      --output /opt/star.tar.gz \
      https://codeload.github.com/alexdobin/STAR/tar.gz/b1edc1208d91a53bf40ebae8669f71d50b994851 && \
    echo '65976afe9b8c5dbda05282c7c02ba27d0f908a7eb83ec12e3fca86ed5eb91cac  /opt/star.tar.gz' | sha256sum --check --strict && \
    tar --extract --gzip --file /opt/star.tar.gz --directory /opt && \
    cd /opt/STAR-b1edc1208d91a53bf40ebae8669f71d50b994851/source && \
    find . -name Makefile -exec sed -Ei 's/-mavx2|-msse[0-9.]*|-mfpmath=sse//g' {} \; && \
    printf '%s\n' 'CFLAGS += -march=armv8-a' 'CXXFLAGS += -march=armv8-a' >> Makefile && \
    make -j"$(nproc)" STAR && \
    install -m 0755 STAR /usr/local/bin/star && \
    star --version | grep -Fx '2.7.11b' && \
    star --help >/dev/null && \
    useradd --system --uid 65532 --home-dir /work --create-home bijux && \
    cd / && \
    rm -rf /opt/star.tar.gz /opt/STAR-b1edc1208d91a53bf40ebae8669f71d50b994851 && \
    apt-get purge -y --auto-remove curl g++ make xxd libbz2-dev zlib1g-dev && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

USER 65532:65532
WORKDIR /work
ENTRYPOINT ["/usr/local/bin/star"]
CMD ["--help"]
