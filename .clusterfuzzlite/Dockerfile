# SPDX-License-Identifier: AGPL-3.0-or-later
# ClusterFuzzLite build environment for echidnabot
FROM gcr.io/oss-fuzz-base/base-builder-rust

COPY . $SRC/echidnabot
WORKDIR $SRC/echidnabot

COPY .clusterfuzzlite/build.sh $SRC/
