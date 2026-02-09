# SPDX-License-Identifier: GPL-3.0-or-later
# Copyright (C) 2026 Bijan Mousavi
#
# This file is part of bijux-dna.
#
# bijux-dna is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# bijux-dna is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with bijux-dna. If not, see <https://www.gnu.org/licenses/>.
# star Dockerfile (ARM64)
# License: Apache-2.0
FROM ubuntu:24.04
ENV DEBIAN_FRONTEND=noninteractive

ARG VERSION_STAR=2.7.11b
ENV VERSION_STAR=${VERSION_STAR}

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        wget ca-certificates build-essential \
        zlib1g-dev libbz2-dev xxd && \
    wget -q https://github.com/alexdobin/STAR/archive/${VERSION_STAR}.tar.gz \
         -O /tmp/star.tar.gz && \
    tar -xzf /tmp/star.tar.gz -C /opt && \
    cd /opt/STAR-${VERSION_STAR}/source && \
    find . -name Makefile -exec \
        sed -Ei 's/-mavx2|-msse[0-9.]*|-mfpmath=sse//g' {} \; && \
    echo 'CFLAGS  += -march=armv8-a+simd' >> Makefile && \
    echo 'CXXFLAGS+= -march=armv8-a+simd' >> Makefile && \
    make -j"$(nproc)" STAR && \
    mv STAR /usr/local/bin/star-bin && \
    rm -rf /tmp/star.tar.gz /opt/STAR-${VERSION_STAR} /var/lib/apt/lists/* && \
    apt-get purge -y wget ca-certificates build-essential xxd && \
    apt-get autoremove -y

# Create a wrapper script for unified STAR execution
RUN echo '#!/bin/sh' > /usr/local/bin/star && \
    echo 'if [ "$1" = "--version" ]; then' >> /usr/local/bin/star && \
    echo '    echo "$VERSION_STAR"' >> /usr/local/bin/star && \
    echo '    exit 0' >> /usr/local/bin/star && \
    echo 'elif [ "$1" = "--help" ]; then' >> /usr/local/bin/star && \
    echo '    /usr/local/bin/star-bin --help 2>&1' >> /usr/local/bin/star && \
    echo '    exit 0' >> /usr/local/bin/star && \
    echo 'else' >> /usr/local/bin/star && \
    echo '    /usr/local/bin/star-bin "$@"' >> /usr/local/bin/star && \
    echo 'fi' >> /usr/local/bin/star && \
    chmod +x /usr/local/bin/star

WORKDIR /data
ENTRYPOINT ["star"]
CMD ["--help"]
