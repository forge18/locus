#!/usr/bin/env bash
set -euo pipefail
matrix="$(cd "$(dirname "$0")/.." && pwd)/webview-matrix.json"
node - "$matrix" <<'NODE'
const fs = require("node:fs");
const matrix = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
for (const platform of ["wkwebview", "webview2", "webkitgtk"]) {
  const entry = matrix[platform];
  if (!entry || !["passed", "failed", "untested"].includes(entry.status)) {
    throw new Error(`${platform} must have an explicit passed, failed, or untested status`);
  }
  if (entry.status === "untested") console.log(`${platform}: untested`);
}
NODE
