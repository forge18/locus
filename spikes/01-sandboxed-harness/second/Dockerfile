# locus/base-hermes — the second harness, on a different capture source.
#
# `.specs` named cursor (ACP) or antigravity (stream-json). Neither is
# obtainable: cursor-agent needs a Cursor account token, and antigravity ships
# no public CLI package. Spike 1's task 12 found a better substitute than either
# — hermes installs from PyPI, passes detect, and ships an `acp` subcommand, so
# it is a DIFFERENT HARNESS on the ACP capture path the spec asked for.
FROM python:3.12-slim-bookworm

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      git ca-certificates curl jq procps socat \
 && rm -rf /var/lib/apt/lists/*

COPY bin/locus bin/locus-hook bin/locus-detect bin/locus-entrypoint bin/locus-sockd /usr/local/bin/
RUN chmod +x /usr/local/bin/locus /usr/local/bin/locus-hook \
             /usr/local/bin/locus-detect /usr/local/bin/locus-entrypoint \
             /usr/local/bin/locus-sockd \
 && mkdir -p /locus

# The harness binary lives in the image. The [acp] extra is what `hermes acp
# --check` asks for; without it the subcommand exists and refuses to run.
ARG HERMES_SPEC="hermes-agent[acp]"
RUN pip install --no-cache-dir "${HERMES_SPEC}"

ARG HARNESS_BINARY=hermes
ARG HARNESS_DETECT=--version
RUN locus-detect "${HARNESS_BINARY}" "${HARNESS_DETECT}"

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

ENV HERMES_HOME=/locus/config \
    HERMES_ACCEPT_HOOKS=1 \
    LOCUS_EVENT_BUFFER=/locus/events.ndjson \
    LOCUS_SOCK=/run/locus.sock

WORKDIR /workspace
ENTRYPOINT ["/usr/local/bin/locus-entrypoint"]
CMD ["hermes", "--version"]
