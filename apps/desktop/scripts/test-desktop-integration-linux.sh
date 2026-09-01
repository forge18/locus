#!/usr/bin/env bash
set -euo pipefail

repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../../.." && pwd)
image=${LOCUS_DESKTOP_INTEGRATION_IMAGE:-locus-desktop-integration:local}
volume_prefix=${image//[^a-zA-Z0-9_.-]/-}
build_context=$(mktemp -d "${TMPDIR:-/tmp}/locus-desktop-integration.XXXXXX")
permissions_dir="$repo_root/apps/desktop/src-tauri/permissions"
created_permissions_dir=0
if [ ! -e "$permissions_dir" ]; then
  mkdir -p "$permissions_dir"
  created_permissions_dir=1
fi
cleanup() {
  rm -rf "$build_context"
  if [ "$created_permissions_dir" -eq 1 ]; then
    rmdir "$permissions_dir" 2>/dev/null || true
  fi
}
trap cleanup EXIT

cp "$repo_root/apps/desktop/Dockerfile.integration" "$build_context/Dockerfile"
docker build --file "$build_context/Dockerfile" --tag "$image" "$build_context"

docker run --rm --init --network host \
  --env CI=true \
  --volume "$repo_root:/workspace:ro" \
  --volume /var/run/docker.sock:/var/run/docker.sock \
  --volume "$volume_prefix-node-modules:/workspace/apps/desktop/node_modules" \
  --volume "$volume_prefix-dist:/workspace/apps/desktop/dist" \
  --tmpfs /workspace/apps/desktop/src-tauri/permissions \
  --volume "$volume_prefix-target:/workspace/target" \
  --volume "$volume_prefix-cargo-registry:/root/.cargo/registry" \
  --volume "$volume_prefix-cargo-git:/root/.cargo/git" \
  --volume "$volume_prefix-pnpm-store:/tmp/pnpm-store" \
  --workdir /workspace \
  "$image" \
  bash -lc 'pnpm --store-dir /tmp/pnpm-store --dir apps/desktop install --frozen-lockfile && xvfb-run -a --server-args="-screen 0 1440x900x24" bash apps/desktop/scripts/test-desktop-integration.sh'
