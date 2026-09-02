#!/usr/bin/env bash
# Verify that Docker is reachable before any Docker-backed test starts.
set -euo pipefail

if ! command -v docker >/dev/null 2>&1; then
    echo 'Docker is required for the Rust test suite but is not installed.' >&2
    exit 1
fi

run_with_timeout() {
    local seconds=$1
    shift
    "$@" >/dev/null 2>&1 &
    local command_pid=$!
    local attempt=0
    while [ "$attempt" -lt $((seconds * 10)) ]; do
        if ! kill -0 "$command_pid" 2>/dev/null; then
            wait "$command_pid"
            return $?
        fi
        sleep 0.1
        attempt=$((attempt + 1))
    done
    kill "$command_pid" >/dev/null 2>&1 || true
    wait "$command_pid" >/dev/null 2>&1 || true
    return 124
}

if ! run_with_timeout 5 docker info; then
    context=$(docker context show 2>/dev/null || printf 'unknown')
    echo "Docker is not reachable (context: ${context})." >&2
    if command -v colima >/dev/null 2>&1; then
        colima_socket="${HOME}/.colima/default/docker.sock"
        if [ -S "$colima_socket" ] && [ "$context" != "colima" ]; then
            echo 'Colima is available, but Docker is using a different context.' >&2
            echo 'Select the Colima context with: docker context use colima' >&2
        elif [ -S "$colima_socket" ]; then
            echo 'The Colima socket exists, but its Docker daemon is unavailable.' >&2
            echo 'Restart Colima with: colima restart' >&2
        else
            echo 'Colima is not running; start it with: colima start' >&2
        fi
    else
        echo 'Start Docker Desktop or another Docker daemon, then retry.' >&2
    fi
    exit 1
fi

echo 'Docker preflight passed.'
