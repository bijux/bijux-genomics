# multiqc Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_MULTIQC=1.24
ENV VERSION_MULTIQC=${VERSION_MULTIQC}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        python3-pip \
        python3-dev && \
    pip3 install --break-system-packages multiqc==${VERSION_MULTIQC} && \
    apt-get purge -y python3-dev && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /data
ENTRYPOINT ["multiqc"]
CMD []
