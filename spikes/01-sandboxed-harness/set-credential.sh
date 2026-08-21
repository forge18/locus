#!/usr/bin/env bash
# The operator-facing way in. Two paths, one destination.
#
#   set-credential.sh api-key    paste a key; read from a prompt with echo off,
#                                or from stdin so CI can pipe it
#   set-credential.sh sign-in    run the provider's browser flow on the HOST and
#                                keep the resulting token pair here
#   set-credential.sh status     what is stored, by kind — never the value
#   set-credential.sh clear      remove it
#
# The store lives OUTSIDE the repo, mode 0600. Nothing here ever writes a
# credential to the spike directory, echoes it, or puts it in an argv where the
# process table would show it.
set -euo pipefail

STORE="${LOCUS_CRED_STORE:-$HOME/.local/state/locus-spike/credentials.json}"
mkdir -p "$(dirname "$STORE")"

write_store() {   # write_store <json>
  umask 077
  printf '%s\n' "$1" > "$STORE"
  chmod 600 "$STORE"
  echo "stored at $STORE (mode $(stat -f '%Lp' "$STORE" 2>/dev/null || stat -c '%a' "$STORE"))"
}

case "${1:-status}" in
  api-key)
    if [ -t 0 ]; then
      printf 'Anthropic API key (input hidden): ' >&2
      stty -echo; IFS= read -r KEY; stty echo; printf '\n' >&2
    else
      IFS= read -r KEY
    fi
    [ -n "${KEY:-}" ] || { echo "no key entered" >&2; exit 2; }
    case "$KEY" in
      sk-ant-*) : ;;
      *) echo "warning: that does not look like an Anthropic API key (expected sk-ant-...)" >&2 ;;
    esac
    write_store "$(KEY="$KEY" jq -n '{kind:"api_key", secret:env.KEY}')"
    unset KEY
    ;;

  sign-in)
    # The sign-in flow runs on the HOST, in the operator's own browser, exactly
    # once. What it produces is a token pair that stays here; the container is
    # never part of the flow and never holds either token.
    #
    # For this spike the flow is not re-implemented — Claude Code already has
    # one, and it writes its result to the OS keychain. The supported path is to
    # hand that token over once:
    cat >&2 <<'MSG'
sign-in stores an OAuth token pair on this host. The container is never part of
the flow and never holds either token — the proxy refreshes and injects.

Paste the access token from an existing Claude Code login (input hidden).
On macOS it is in the Keychain under "Claude Code-credentials"; `claude setup-token`
also mints one.
MSG
    if [ -t 0 ]; then
      printf 'access token: ' >&2; stty -echo; IFS= read -r TOK; stty echo; printf '\n' >&2
      printf 'refresh token (optional, Enter to skip): ' >&2; stty -echo; IFS= read -r RTOK; stty echo; printf '\n' >&2
    else
      IFS= read -r TOK; RTOK=""
    fi
    [ -n "${TOK:-}" ] || { echo "no token entered" >&2; exit 2; }
    write_store "$(TOK="$TOK" RTOK="${RTOK:-}" jq -n '
      {kind:"oauth", access_token:env.TOK, expires_at:0}
      + (if env.RTOK == "" then {} else {refresh_token:env.RTOK} end)')"
    unset TOK RTOK
    ;;

  status)
    if [ -s "$STORE" ]; then
      jq '{kind, has_refresh_token: (has("refresh_token")), expires_at,
           secret: "<redacted>", access_token: "<redacted>", refresh_token: "<redacted>"}
          | with_entries(select(.value != null))' "$STORE"
      echo "path: $STORE  mode: $(stat -f '%Lp' "$STORE" 2>/dev/null || stat -c '%a' "$STORE")"
    else
      echo '{"kind":null,"hint":"set-credential.sh api-key | sign-in"}'
    fi
    ;;

  clear)
    rm -f "$STORE" && echo "cleared $STORE"
    ;;

  *)
    echo "usage: set-credential.sh api-key|sign-in|status|clear" >&2; exit 2
    ;;
esac
