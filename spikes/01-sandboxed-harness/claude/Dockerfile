# locus/base-<harness> — layer 1 of the two-layer image model (PLAN.md §Images).
#
#   locus/base-claude:<version>   OS, git, the harness CLI, locus and locus-hook
#   locus/agent-<hash>            the agent's tools allowlist, built on top
#
# Config is NOT a layer. It is materialized per run into /locus/config as a
# read-only mount, so editing a skill never invalidates an image.
FROM node:24-bookworm-slim

# ---- OS layer -------------------------------------------------------------
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      git ca-certificates curl jq procps socat \
 && rm -rf /var/lib/apt/lists/*

# ---- locus's own binaries -------------------------------------------------
# These are in the base image, not on the host: an agent's only surface is the
# CLI over /run/locus.sock (PLAN.md §The one surface).
COPY bin/locus bin/locus-hook bin/locus-detect bin/locus-entrypoint bin/locus-sockd /usr/local/bin/
RUN chmod +x /usr/local/bin/locus /usr/local/bin/locus-hook \
             /usr/local/bin/locus-detect /usr/local/bin/locus-entrypoint \
             /usr/local/bin/locus-sockd \
 && mkdir -p /locus

# ---- the harness CLI ------------------------------------------------------
# "The harness binary lives in the image, never on the host."
ARG HARNESS_PKG=@anthropic-ai/claude-code
ARG HARNESS_VERSION=latest
RUN npm install -g --no-fund --no-audit "${HARNESS_PKG}@${HARNESS_VERSION}"

# ---- detect ---------------------------------------------------------------
# From harnesses/claude.toml:  binary = "claude"   detect = ["--version"]
# This step exists to FAIL. See Dockerfile.nodetect for the proof.
ARG HARNESS_BINARY=claude
ARG HARNESS_DETECT=--version
RUN locus-detect "${HARNESS_BINARY}" "${HARNESS_DETECT}"

# ---- a bare local remote, so /workspace can be a clone with no mount ------
# The real system points LOCUS_REMOTE at the project's bare remote on the host
# (PLAN.md §The git model). This baked fixture is what makes the no-mount claim
# runnable standalone: `docker run --rm locus/base-claude ...` with no volumes.
RUN git config --global user.email agent@locus.invalid \
 && git config --global user.name  "locus agent" \
 && git config --global init.defaultBranch main \
 && git config --global --add safe.directory '*' \
 && git init --quiet --bare -b main /var/lib/locus/fixture.git \
 && tmp="$(mktemp -d)" && git init --quiet -b main "$tmp" \
 && printf '# fixture\n' > "$tmp/README.md" \
 && git -C "$tmp" add -A && git -C "$tmp" commit --quiet -m 'fixture: initial' \
 && git -C "$tmp" push --quiet /var/lib/locus/fixture.git main \
 && rm -rf "$tmp"

# Claude reads its home from this env var (harnesses/claude.toml [config]).
ENV CLAUDE_CONFIG_DIR=/locus/config \
    LOCUS_EVENT_BUFFER=/locus/events.ndjson \
    LOCUS_SOCK=/run/locus.sock

WORKDIR /workspace
ENTRYPOINT ["/usr/local/bin/locus-entrypoint"]
CMD ["claude", "--version"]
