#!/usr/bin/env python3
"""Small first-party JSON-RPC executables used until packaged binaries exist."""

import argparse
import json
import sys

CATALOG = {
    "gh": {
        "kind": "cli_tool",
        "capabilities": [
            "cli_tool.install",
            "cli_tool.verify",
            "cli_tool.docs",
            "cli_tool.digest",
        ],
        "permissions": ["network", "repository_read"],
    },
    "openai": {
        "kind": "provider",
        "capabilities": ["provider.models", "provider.verify", "provider.aliases"],
        "permissions": ["keychain_reference"],
        "models": [
            {"id": "gpt-4o", "alias": "GPT-4o"},
            {"id": "gpt-4.1", "alias": None},
        ],
    },
    "anthropic": {
        "kind": "provider",
        "capabilities": ["provider.models", "provider.verify", "provider.aliases"],
        "permissions": ["keychain_reference"],
        "models": [
            {"id": "claude-sonnet-4", "alias": "Sonnet"},
            {"id": "claude-opus-4", "alias": "Opus"},
        ],
    },
    "openrouter": {
        "kind": "provider",
        "capabilities": ["provider.models", "provider.verify", "provider.aliases"],
        "permissions": ["keychain_reference"],
        "models": [
            {"id": "openai/gpt-4o", "alias": "GPT-4o"},
            {"id": "anthropic/claude-sonnet-4", "alias": "Sonnet"},
        ],
    },
}


def emit(request, result=None, error=None):
    payload = {"jsonrpc": "2.0", "id": request.get("id")}
    payload["error" if error is not None else "result"] = (
        error if error is not None else result
    )
    print(json.dumps(payload, separators=(",", ":")), flush=True)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--id", required=True, choices=sorted(CATALOG))
    args = parser.parse_args()
    plugin = CATALOG[args.id]
    for raw in sys.stdin:
        try:
            request = json.loads(raw)
        except (json.JSONDecodeError, TypeError):
            emit({"id": None}, error={"code": -32700, "message": "invalid JSON"})
            continue
        if not isinstance(request, dict):
            emit(
                {"id": None},
                error={"code": -32600, "message": "request must be an object"},
            )
            continue
        method = request.get("method")
        if method == "plugin.initialize":
            emit(
                request,
                {"protocol": "locus.plugin.v1", "capabilities": plugin["capabilities"]},
            )
        elif method == "plugin.describe":
            emit(
                request,
                {
                    "protocol": "locus.plugin.v1",
                    "kind": plugin["kind"],
                    "id": args.id,
                    "version": "1.0.0",
                    "capabilities": plugin["capabilities"],
                    "schema_versions": {"plugin": "v1"},
                    "permissions": plugin["permissions"],
                },
            )
        elif method == "plugin.health":
            emit(request, {"ready": True})
        elif method == "plugin.shutdown":
            emit(request, {"stopped": True})
            break
        elif method == "provider.models":
            emit(request, {"models": plugin["models"]})
        elif method == "provider.verify":
            emit(request, {"status": "unverified"})
        elif method == "cli_tool.describe":
            emit(
                request,
                {
                    "install": "gh --version",
                    "verify": "gh --version",
                    "docs": "https://cli.github.com/manual/",
                    "digest": "registry-pinned",
                    "permissions": plugin["permissions"],
                },
            )
        else:
            emit(request, error={"code": -32601, "message": "method not found"})


if __name__ == "__main__":
    main()
