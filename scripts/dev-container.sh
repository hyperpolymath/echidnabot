#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Rootless Podman development shell; no host Rust installation required.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v podman >/dev/null || { echo 'Install Podman first; see docs/DEVELOPMENT.adoc.' >&2; exit 1; }
image=${ECHIDNABOT_DEV_IMAGE:-localhost/echidnabot-dev}
if [[ ${1:-} == --build ]]; then
    shift
    podman build --target development --tag "$image" --file Containerfile .
fi
if ! podman image exists "$image"; then
    echo 'Development image missing. Run scripts/dev-container.sh --build.' >&2
    exit 1
fi
args=(--rm --userns=keep-id:uid=1000,gid=1000 --workdir /workspace
    --volume "$PWD:/workspace:Z"
    --volume echidnabot-dev-target:/workspace/target:U
    --volume echidnabot-dev-cargo:/home/dev/.cargo:U)
if [[ -t 0 && -t 1 ]]; then args+=(-it); fi
if [[ $# == 0 ]]; then set -- bash; fi
exec podman run "${args[@]}" "$image" "$@"
