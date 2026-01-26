# trim_galore Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_TRIM_GALORE=0.6.10
ENV TRIM_GALORE=${VERSION_TRIM_GALORE}

RUN apt-get update && apt-get install -y --no-install-recommends \
    python3 python3-pip python3-dev build-essential fastqc wget && \
    rm -rf /var/lib/apt/lists/*


RUN wget -q https://github.com/FelixKrueger/TrimGalore/archive/${VERSION_TRIM_GALORE}.tar.gz \
    -O /tmp/trimgalore.tar.gz && \
    tar -xzf /tmp/trimgalore.tar.gz -C /opt && \
    ln -sf /opt/TrimGalore-${VERSION_TRIM_GALORE}/trim_galore /usr/local/bin/trim_galore && \
    chmod +x /usr/local/bin/trim_galore && \
    rm /tmp/trimgalore.tar.gz

RUN pip3 install --no-cache-dir --break-system-packages cutadapt

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "trim_galore \"$@\""]
CMD []
