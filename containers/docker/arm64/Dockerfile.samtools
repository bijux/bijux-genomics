# samtools Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_SAMTOOLS=1.21
ENV VERSION_SAMTOOLS=${VERSION_SAMTOOLS}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        samtools && \
    mv /usr/bin/samtools /usr/local/bin/samtools-bin && \
    rm -rf /var/lib/apt/lists/* && \
    apt-get autoremove -y

# Create a wrapper script for unified Samtools execution
RUN echo '#!/bin/sh' > /usr/local/bin/samtools && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/samtools && \
    echo '    echo "$VERSION_SAMTOOLS"' >> /usr/local/bin/samtools && \
    echo '    exit 0' >> /usr/local/bin/samtools && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/samtools && \
    echo '    /usr/local/bin/samtools-bin --help' >> /usr/local/bin/samtools && \
    echo '    exit 0' >> /usr/local/bin/samtools && \
    echo 'else' >> /usr/local/bin/samtools && \
    echo '    /usr/local/bin/samtools-bin "$@"' >> /usr/local/bin/samtools && \
    echo 'fi' >> /usr/local/bin/samtools && \
    chmod +x /usr/local/bin/samtools

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "samtools \"$@\""]
CMD ["--help"]
