#!/usr/bin/env bash
# Q4 — /workspace as a container-local clone from a HOST bare remote, no mount.
#
# The one-liner in tasks.md task 4 proves the clone works against the fixture
# baked into the image. This proves the real shape from PLAN.md §The git model:
#
#   host: /var/lib/locus/repos/<project>.git      the bare local remote
#      |  clone                        push branch
#      v                                    ^
#   container: /workspace  (container-local, NOT a mount)
#
# and it proves the round trip, because a clone you cannot push back from is
# not the model — it is half of it.
set -euo pipefail
cd "$(dirname "$0")"

PORT="${LOCUS_GIT_PORT:-43900}"
RUN_ID="clone-$$"
WORK="out/clone"
REMOTE_ROOT="$PWD/$WORK/remotes"
rm -rf "$WORK"; mkdir -p "$REMOTE_ROOT"

# --- host side: a bare local remote with one commit on main -----------------
seed="$WORK/seed"
git init --quiet -b main "$seed"
printf 'fn main() { println!("host"); }\n' > "$seed/main.rs"
git -C "$seed" add -A
git -C "$seed" -c user.email=host@locus.invalid -c user.name=host commit --quiet -m 'host: initial'
HOST_SHA="$(git -C "$seed" rev-parse HEAD)"
git init --quiet --bare -b main "$REMOTE_ROOT/project.git"
git -C "$seed" push --quiet "$REMOTE_ROOT/project.git" main

# --- host side: expose it on a git transport the container can reach --------
git daemon --reuseaddr --listen=0.0.0.0 --port="$PORT" \
           --base-path="$REMOTE_ROOT" --export-all \
           --enable=receive-pack "$REMOTE_ROOT" >/dev/null 2>&1 &
DAEMON=$!
trap 'kill "$DAEMON" 2>/dev/null || true' EXIT
sleep 1

# --- container side: clone, work, push back. No -v. No --mount. -------------
docker run --rm \
  --add-host=host.docker.internal:host-gateway \
  -e LOCUS_REMOTE="git://host.docker.internal:$PORT/project.git" \
  -e LOCUS_RUN_ID="$RUN_ID" \
  locus/base-claude sh -eu -c '
    echo "workspace-is-worktree: $(git -C /workspace rev-parse --is-inside-work-tree)"
    echo "workspace-head-sha:    $(git -C /workspace rev-parse HEAD)"
    echo "workspace-branch:      $(git -C /workspace rev-parse --abbrev-ref HEAD)"
    if findmnt -no TARGET /workspace >/dev/null 2>&1; then
      echo "MOUNT-DETECTED"; exit 1
    fi
    echo "workspace-is-mount:    no"
    printf "// touched by the agent\n" >> /workspace/main.rs
    git -C /workspace add -A
    git -C /workspace commit --quiet -m "agent: touch"
    git -C /workspace push --quiet origin HEAD
    echo "pushed:                $(git -C /workspace rev-parse --abbrev-ref HEAD)"
  ' | tee "$WORK/container.log"

# --- host side: the branch came back, and main is untouched -----------------
branches="$(git -C "$REMOTE_ROOT/project.git" for-each-ref --format='%(refname:short)' refs/heads)"
echo "remote-branches: $(echo "$branches" | tr '\n' ' ')"

fail=0
grep -q "workspace-head-sha:    $HOST_SHA" "$WORK/container.log" \
  || { echo "FAIL: container did not clone the host commit"; fail=1; }
grep -q "workspace-is-mount:    no" "$WORK/container.log" \
  || { echo "FAIL: /workspace was a mount"; fail=1; }
echo "$branches" | grep -qx "agent/$RUN_ID" \
  || { echo "FAIL: agent branch did not reach the host remote"; fail=1; }
[ "$(git -C "$REMOTE_ROOT/project.git" rev-parse main)" = "$HOST_SHA" ] \
  || { echo "FAIL: main moved — the never-work-in-main invariant broke"; fail=1; }

if [ "$fail" -eq 0 ]; then
  echo "PASS: clone from host bare remote, no mount, branch pushed back, main untouched"
fi
exit "$fail"
