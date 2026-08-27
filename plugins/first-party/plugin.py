#!/usr/bin/env python3
"""Small first-party JSON-RPC executables used until packaged binaries exist."""

import argparse
import json
import subprocess
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
    "github": {
        "kind": "provider",
        "capabilities": [
            "work_item.snapshot",
            "work_item.comment",
            "work_item.resolve",
        ],
        "permissions": ["network", "repository_read", "repository_write"],
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


def _required_string(value, name):
    if not isinstance(value, str) or not value.strip() or "\x00" in value:
        raise ValueError(f"{name} must be a non-empty string")
    return value


def _lookup(params):
    if not isinstance(params, dict):
        raise ValueError("params must be an object")
    plugin_id = _required_string(params.get("plugin_id"), "plugin_id")
    if plugin_id != "github":
        raise ValueError("plugin_id must be github")
    lookup = {
        "plugin_id": plugin_id,
        "host": _required_string(params.get("host"), "host"),
        "project": _required_string(params.get("project"), "project"),
        "native_id": _required_string(params.get("native_id"), "native_id"),
    }
    if lookup["host"] != "github.com":
        raise ValueError("github host must be github.com")
    if any(lookup[name].startswith("-") for name in ("host", "project", "native_id")):
        raise ValueError("lookup values must not start with a dash")
    parts = lookup["project"].split("/")
    if len(parts) != 2 or any(not part or any(char.isspace() for char in part) for part in parts):
        raise ValueError("project must be an owner/repository pair")
    if not lookup["native_id"].isascii() or not lookup["native_id"].isdigit():
        raise ValueError("native_id must be a GitHub issue number")
    return lookup


def _repository(lookup):
    return f"{lookup['host']}/{lookup['project']}"


def _view_issue(lookup, fields):
    return _run_gh(
        [
            "issue",
            "view",
            lookup["native_id"],
            "--repo",
            _repository(lookup),
            "--json",
            fields,
        ]
    )


def _run_gh(arguments, decode_json=True):
    try:
        completed = subprocess.run(
            ["gh", *arguments],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except FileNotFoundError as error:
        raise RuntimeError("GitHub CLI `gh` is not installed") from error
    except subprocess.TimeoutExpired as error:
        raise RuntimeError("GitHub CLI request timed out") from error
    if completed.returncode != 0:
        message = completed.stderr.strip() or "request failed"
        raise RuntimeError(f"GitHub CLI request failed: {message}")
    if not decode_json:
        return None
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError("GitHub CLI returned invalid JSON") from error


def _snapshot(params):
    lookup = _lookup(params)
    result = _view_issue(lookup, "number,url,title,body,labels,state")
    if not isinstance(result, dict):
        raise RuntimeError("GitHub CLI returned a non-object issue")
    labels = result.get("labels", [])
    if not isinstance(labels, list) or any(
        not isinstance(label, dict) for label in labels
    ):
        raise RuntimeError("GitHub CLI returned invalid issue labels")
    body = result.get("body") or ""
    if not isinstance(body, str):
        raise RuntimeError("GitHub CLI returned an invalid issue body")
    return {
        "identity": {
            **lookup,
            "native_id": str(result.get("number", lookup["native_id"])),
        },
        "url": _required_string(result.get("url"), "url"),
        "title": _required_string(result.get("title"), "title"),
        "body": body,
        "labels": [
            _required_string(label.get("name"), "label.name") for label in labels
        ],
        "status": _required_string(result.get("state"), "state").lower(),
    }


def _completion_params(params):
    if not isinstance(params, dict):
        raise ValueError("params must be an object")
    lookup = _lookup(params.get("identity"))
    event = params.get("event")
    if not isinstance(event, dict):
        raise ValueError("event must be an object")
    comment = _required_string(event.get("comment"), "event.comment")
    event_id = _required_string(event.get("id"), "event.id")
    return lookup, comment, event_id


def _complete(method, params):
    lookup, comment, event_id = _completion_params(params)
    if method == "work_item.comment":
        existing = _view_issue(lookup, "comments")
        comments = existing.get("comments") if isinstance(existing, dict) else None
        if not isinstance(comments, list):
            raise RuntimeError("GitHub CLI returned invalid issue comments")
        marker = f"<!-- locus-completion:{event_id} -->"
        if any(
            isinstance(item, dict)
            and isinstance(item.get("body"), str)
            and marker in item["body"]
            for item in comments
        ):
            return {"commented": True, "already_present": True}
        _run_gh(
            [
                "issue",
                "comment",
                lookup["native_id"],
                "--repo",
                _repository(lookup),
                "--body",
                comment,
            ],
            decode_json=False,
        )
        return {"commented": True}
    state = _view_issue(lookup, "state")
    if not isinstance(state, dict) or not isinstance(state.get("state"), str):
        raise RuntimeError("GitHub CLI returned invalid issue state")
    if state["state"].upper() == "CLOSED":
        return {"resolved": True, "already_closed": True}
    _run_gh(
        [
            "issue",
            "close",
            lookup["native_id"],
            "--repo",
            _repository(lookup),
            "--reason",
            "completed",
        ],
        decode_json=False,
    )
    return {"resolved": True}


def work_item_call(method, params):
    if method == "work_item.snapshot":
        return _snapshot(params)
    if method in ("work_item.comment", "work_item.resolve"):
        return _complete(method, params)
    raise ValueError("method not found")


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
        request_id = request.get("id")
        valid_id = isinstance(request_id, int) and not isinstance(request_id, bool)
        if (
            request.get("jsonrpc") != "2.0"
            or not valid_id
            or not isinstance(method, str)
            or not method
        ):
            emit(
                {"id": request_id},
                error={"code": -32600, "message": "invalid JSON-RPC request"},
            )
            continue
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
        elif method == "provider.models" and "models" in plugin:
            emit(request, {"models": plugin["models"]})
        elif method == "provider.verify" and "models" in plugin:
            emit(request, {"status": "unverified"})
        elif method == "cli_tool.describe" and plugin["kind"] == "cli_tool":
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
        elif method in (
            "work_item.snapshot",
            "work_item.comment",
            "work_item.resolve",
        ):
            try:
                emit(request, work_item_call(method, request.get("params")))
            except (RuntimeError, ValueError) as error:
                emit(request, error={"code": -32000, "message": str(error)})
        else:
            emit(request, error={"code": -32601, "message": "method not found"})


if __name__ == "__main__":
    main()
