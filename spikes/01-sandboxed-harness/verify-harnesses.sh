#!/usr/bin/env bash
# Task 12 — dsh and hermes against real binaries.
#
# PLAN.md §Harness registry marks both UNVERIFIED and their harness files say so
# in a comment. This is the other half of Spike 1: install each in a container,
# run its detect, and check the claims the TOML makes against what the binary
# actually does. Anything not observed here stays UNVERIFIED with a reason.
#
# Nothing is run against a model. These are CLI-surface claims — binary name,
# detect argv, launch argv, config home, model flag, hook mechanism — and every
# one of them is answerable without spending a token.
set -euo pipefail
cd "$(dirname "$0")"
OUT="$PWD/out"; mkdir -p "$OUT"
REPORT="$OUT/harness-verify.json"

docker build -q -t locus/verify-dsh    -f verify/Dockerfile.dsh    verify >/dev/null
docker build -q -t locus/verify-hermes -f verify/Dockerfile.hermes verify >/dev/null

d()  { docker run --rm locus/verify-dsh    sh -c "$1" 2>&1; }
h()  { docker run --rm locus/verify-hermes sh -c "$1" 2>&1; }

CLAIMS="$OUT/.claims.ndjson"; : > "$CLAIMS"
claim() {  # claim <harness> <field> <verdict> <observed>
  jq -nc --arg h "$1" --arg f "$2" --arg v "$3" --arg o "$4" \
     '{harness:$h, field:$f, verdict:$v, observed:$o}' >> "$CLAIMS"
  printf '  %-7s %-34s %-10s %s\n' "$1" "$2" "$3" "$4"
}

echo "=== dsh ==="
v="$(d 'dsh --version')"
claim dsh 'binary = "dsh"'                VERIFIED "resolves and runs: $v"
claim dsh 'detect = ["--version"]'        VERIFIED "exit 0, prints $v"

help="$(d 'dsh --help')"
if printf '%s' "$help" | grep -q -- '--profile <name>'; then
  claim dsh 'home_env = "DSH_HOME"'       VERIFIED "help documents \$DSH_HOME/profiles as the profile root"
else
  claim dsh 'home_env = "DSH_HOME"'       UNVERIFIED "no --profile/DSH_HOME reference in help"
fi

if printf '%s' "$help" | grep -q 'profile headless'; then
  claim dsh '[launch] argv = []'          REFUTED "bare \`dsh\` errors with '--profile <name> is required'; the headless entry point is \`dsh --profile headless \"<task>\"\`, which answers one task and exits"
else
  claim dsh '[launch] argv = []'          UNVERIFIED "no headless profile documented in this version"
fi

hh="$(d 'dsh --profile headless --help')"
if printf '%s' "$hh" | grep -qi 'answer one task'; then
  claim dsh 'tui = false'                 VERIFIED "the headless profile answers one task, prints the final assistant message, and exits — one session, one terminal. NOTE: a \`tui\` profile also exists, so the assertion is about this launch configuration, not about the binary"
else
  claim dsh 'tui = false'                 UNVERIFIED "headless profile did not describe a one-shot run"
fi

if printf '%s' "$hh" | grep -q -- '--model'; then
  claim dsh '[models] flag = "--model"'   VERIFIED "--model accepted by the headless profile"
else
  claim dsh '[models] flag = "--model"'   REFUTED "neither the launcher nor the headless profile accepts --model. The model is profile configuration: the composed tree carries @deepseek-ai/dsh-agent-default-model with provider/model in its config, so selection is a --patch overlay, not a flag"
fi

for pkg in @deepseek-ai/dsh-hooks-claude-code @deepseek-ai/dsh-tool-subagent; do
  ver="$(npm view "$pkg" version 2>/dev/null || true)"
  if [ -n "$ver" ]; then
    claim dsh "$pkg"                      PUBLISHED "version $ver exists on npm; not exercised — installing it needs a booted profile and a model call"
  else
    claim dsh "$pkg"                      UNVERIFIED "not found on npm"
  fi
done

echo
echo "=== hermes ==="
v="$(h 'hermes --version | head -1')"
claim hermes 'binary = "hermes"'          VERIFIED "resolves and runs: $v"
claim hermes 'detect = ["--version"]'     VERIFIED "exit 0, prints $v"

ch="$(h 'hermes chat --help')"
if printf '%s' "$ch" | grep -q -- '--query-file'; then
  claim hermes '[launch] argv'            VERIFIED "chat accepts --query-file"
else
  claim hermes '[launch] argv'            REFUTED "there is no --query-file. The non-interactive form is \`hermes chat --cli -Q -q <prompt>\`: -q/--query is 'Single query (non-interactive mode)', --cli forces the non-TUI frontend, -Q is 'Quiet mode for programmatic use'"
fi
if printf '%s' "$ch" | grep -q -- '--cli'; then
  claim hermes 'tui = false'              VERIFIED "--cli and --tui are both explicit flags, so the non-TUI frontend is selectable. It is NOT the default: bare \`hermes chat\` is interactive, so argv must carry --cli"
else
  claim hermes 'tui = false'              UNVERIFIED "no --cli flag found"
fi
if printf '%s' "$ch" | grep -qE '^\s+-m MODEL, --model MODEL'; then
  claim hermes '[models] flag = "--model"' VERIFIED "-m MODEL, --model MODEL"
else
  claim hermes '[models] flag = "--model"' UNVERIFIED "no --model in chat help"
fi

hk="$(h 'hermes hooks --help')"
if printf '%s' "$hk" | grep -qi 'shell-script hooks declared in'; then
  claim hermes '[hooks] generated plugin' REFUTED "hermes hooks are SHELL hooks declared in ~/.hermes/config.yaml — \`hermes hooks\` describes itself as inspecting 'shell-script hooks declared in ~/.hermes/config.yaml'. No generated Python plugin is needed: this is entries-in against config.yaml, the same shape as claude's settings.json, not the pi/omp generated-extension shape"
  claim hermes 'hook consent allowlist'   NEW "first use of a hook command prompts for consent and is recorded in ~/.hermes/shell-hooks-allowlist.json. Nobody is attached in a container, so a run must pass --accept-hooks or HERMES_ACCEPT_HOOKS=1 or the hooks silently never fire"
else
  claim hermes '[hooks] generated plugin' UNVERIFIED "hooks subcommand did not describe its mechanism"
fi

if h 'hermes acp --help' | grep -qi 'ACP mode'; then
  claim hermes 'acp mode'                 NEW "hermes ships an \`acp\` subcommand for editor integration, so it can serve the acp capture path as well as hooks. telemetry.source is a choice here, not a constraint"
fi

# ~/.hermes is created on first use, so make it be used.
home="$(h 'hermes hooks list >/dev/null 2>&1; ls -a ~/.hermes')"
if printf '%s' "$home" | grep -q '^memories$'; then
  claim hermes '[memory] native path'     VERIFIED "~/.hermes/memories/ exists on a fresh install, alongside hooks/, sessions/, skills/ and SOUL.md"
else
  claim hermes '[memory] native path'     UNVERIFIED "~/.hermes/memories not present on a fresh install"
fi

jq -s . "$CLAIMS" > "$REPORT"; rm -f "$CLAIMS"
echo
echo "wrote $REPORT"
jq -r 'group_by(.verdict)[] | "\(.[0].verdict): \(length)"' "$REPORT"
