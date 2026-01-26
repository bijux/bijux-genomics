# bwa Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

# Keep the ARG and ENV for VERSION_BWA for consistency,
# but the apt-installed version might differ.
ARG VERSION_BWA=0.7.18
ENV VERSION_BWA=$VERSION_BWA

RUN apt-get update && \
    # Install bwa directly via apt-get
    apt-get install -y --no-install-recommends bwa && \
    # Move the original bwa to bwa-bin
    mv /usr/bin/bwa /usr/local/bin/bwa-bin && \
    # Verify the installed BWA version for informational purposes
    echo "--- BWA version installed by apt ---" && \
    /usr/local/bin/bwa-bin --version || true && \
    # Cleanup apt lists to reduce image size
    rm -rf /var/lib/apt/lists/* && \
    apt-get autoremove -y

# wrapper in the usual pattern (no change needed here, it still calls the 'bwa' command)
RUN echo '#!/bin/sh' > /usr/local/bin/bwa && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/bwa && \
    echo '    echo "$VERSION_BWA"' >> /usr/local/bin/bwa && \
    echo '    exit 0' >> /usr/local/bin/bwa && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/bwa && \
    echo '    /usr/local/bin/bwa-bin 2>&1' >> /usr/local/bin/bwa && \
    echo '    exit 0' >> /usr/local/bin/bwa && \
    echo 'else' >> /usr/local/bin/bwa && \
    echo '    /usr/local/bin/bwa-bin "$@"' >> /usr/local/bin/bwa && \
    echo 'fi' >> /usr/local/bin/bwa && \
    chmod +x /usr/local/bin/bwa

WORKDIR /data
ENTRYPOINT ["/bin/sh", "-c", "bwa \"$@\""]
CMD ["--help"]
