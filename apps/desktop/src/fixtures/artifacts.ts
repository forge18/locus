// schema: agents.artifacts + agents.artifact_comments
// replaced by: invoke("artifacts_list") + invoke("artifact_comments")

import type { Artifact, ArtifactComment, ArtifactKind } from "../types/agents";

/**
 * Two groups, and the split is load-bearing.
 *
 * Review kinds are what a person looks at. Reference kinds are storage with a
 * handle — an agent's own scratch. Without the split the inbox fills with the
 * second, and the surface built to protect attention becomes the one that spends it.
 */
export const REVIEW_KINDS: Array<{
  kind: ArtifactKind;
  label: string;
  icon: string;
  note: string;
}> = [
  { kind: "diff", label: "diff", icon: "git-diff", note: "2 files · +79 −7" },
  {
    kind: "plan",
    label: "walkthrough",
    icon: "list-checks",
    note: "6 steps, each with evidence",
  },
  { kind: "image", label: "image", icon: "image", note: "OCR text derived" },
  {
    kind: "recording",
    label: "recording",
    icon: "video",
    note: "9 keyframes derived",
  },
  {
    kind: "report",
    label: "diagram",
    icon: "graph",
    note: "nodes and edges, described",
  },
];

export const REFERENCE_KINDS: Array<{
  kind: string;
  label: string;
  icon: string;
  note: string;
}> = [
  {
    kind: "finding",
    label: "finding",
    icon: "magnifying-glass",
    note: "an agent reading its own work",
  },
  {
    kind: "payload",
    label: "payload",
    icon: "database",
    note: "a blob with a handle",
  },
];

export const REFERENCE_GROUP_LABEL = "Reference · never in the inbox";

export const ONE_VIEWER_NOTE = "one viewer per kind · three entry points";

export const ARTIFACTS: Artifact[] = [
  {
    id: "a-1",
    runId: "r-0000-0",
    kind: "diff",
    title: "crates/locus-core/src/store/notify.rs",
    body: null,
    blobPath: null,
    mediaType: "text/x-diff",
    sha256: "9f2c7b1a4e6d80c35ab1f0e29d4c7a83b5e61f0d2c9a8b7e6f5d4c3b2a190807",
    derivedText:
      "2 files changed, 79 insertions, 7 deletions. The send path now returns Dropped instead of silently discarding.",
    createdAt: "2026-08-20T14:26:00Z",
  },
  {
    id: "a-2",
    runId: "r-0001-0",
    kind: "report",
    title: "Verify report — weaver parser, iteration 3",
    body: "cargo test -p weaver parser::\n\nfailures:\n    parser::tests::unterminated_block\n\ntest result: FAILED. 41 passed; 1 failed",
    blobPath: null,
    mediaType: "text/plain",
    sha256: "3a1d5f9c8b2e7046d1c3a5b7e9f0d2c4a6b8e0f1d3c5a7b9e1f3d5c7a9b1e3d5",
    derivedText:
      "Same failure for the third iteration: parser::tests::unterminated_block. 41 passed, 1 failed.",
    createdAt: "2026-08-20T14:28:00Z",
  },
  {
    id: "a-3",
    runId: "r-0002-0",
    kind: "image",
    title: "Board after the ingest run",
    body: null,
    blobPath: "~/.locus/blobs/3f/3f81c0.png",
    mediaType: "image/png",
    sha256: "c7e1a3b5d7f9012e4a6c8b0d2f4a6c8e0b2d4f6a8c0e2b4d6f8a0c2e4b6d8f0a",
    // Text is cheaper than pixels: an agent gets this, never the blob.
    derivedText:
      "Screenshot of the board: 3 Ready, 2 Building, 1 Testing, 2 Reviewing, 1 Waiting For Approval, 1 Done.",
    createdAt: "2026-08-20T13:58:00Z",
  },
];

// Compaction handles are first-class artifacts: short-term can drop a result
// without making it unreachable from a later entry point.
ARTIFACTS.push(
  {
    id: "a-4",
    runId: "r-0003-0",
    kind: "recording",
    title: "Browser capture — 42 seconds",
    body: null,
    blobPath: "~/.locus/blobs/4a/4a42c0.webm",
    mediaType: "video/webm",
    sha256: "4a42c0d9e8f7012e3a5b6c7d8e9f00112233445566778899aabbccddeeff0011",
    derivedText:
      "9 keyframes extracted for model context; the original clip remains human-only.",
    createdAt: "2026-08-20T14:02:00Z",
  },
  {
    id: "a-7802",
    runId: "r-9f21",
    kind: "payload",
    title: "agentclientprotocol.com/protocol",
    body: null,
    blobPath: null,
    mediaType: "text/plain",
    sha256: "7802",
    derivedText:
      "Compacted web_fetch result for agentclientprotocol.com/protocol.",
    createdAt: "2026-08-20T14:20:00Z",
  },
  {
    id: "a-7811",
    runId: "r-9f21",
    kind: "log",
    title: "cargo build — full output",
    body: null,
    blobPath: null,
    mediaType: "text/plain",
    sha256: "7811",
    derivedText: "Compacted cargo build output.",
    createdAt: "2026-08-20T14:21:00Z",
  },
  {
    id: "a-7815",
    runId: "r-9f21",
    kind: "payload",
    title: "store/mod.rs — whole file",
    body: null,
    blobPath: null,
    mediaType: "text/plain",
    sha256: "7815",
    derivedText: "Compacted read_file result for store/mod.rs.",
    createdAt: "2026-08-20T14:22:00Z",
  },
);

export const SELECTED_ARTIFACT_ID = "a-1";

export const ARTIFACT_LOCATOR = "locus://tapestry/artifact/a-1";

export type UnifiedRowKind = "hunk" | "context" | "added" | "removed";

export interface UnifiedRow {
  kind: UnifiedRowKind;
  /** Null on a hunk header. */
  no: number | null;
  text: string;
  /** True where a comment thread hangs off this line. */
  commented?: boolean;
}

export const UNIFIED_DIFF: UnifiedRow[] = [
  { kind: "hunk", no: null, text: "@@ -18,7 +18,9 @@ impl Notifier" },
  {
    kind: "context",
    no: 19,
    text: "    pub async fn notify(&self, ch: &str, id: Uuid) -> Result<()> {",
  },
  {
    kind: "removed",
    no: 20,
    text: '        sqlx::query("SELECT pg_notify($1, $2)")',
  },
  {
    kind: "added",
    no: 20,
    text: "        // NOTIFY carries an id only — payload cap is 8000 bytes",
  },
  {
    kind: "added",
    no: 21,
    text: '        sqlx::query("SELECT pg_notify($1, $2)")',
    commented: true,
  },
  {
    kind: "removed",
    no: 21,
    text: "            .bind(ch).bind(serde_json::to_string(&row)?)",
  },
  { kind: "added", no: 22, text: "            .bind(ch).bind(id.to_string())" },
  { kind: "context", no: 23, text: "            .execute(&self.pool).await?;" },
  { kind: "hunk", no: null, text: "@@ -71,4 +73,6 @@ impl Notifier" },
  {
    kind: "context",
    no: 74,
    text: "    /// Listener reconnects with backoff.",
  },
  {
    kind: "added",
    no: 75,
    text: "        let mut backoff = Duration::from_millis(100);",
  },
];

export const COMMENTS_TITLE = "Comments steer the agent";

export const LIVE_COMMENT_NOTE =
  "run is still live · comment routed into the session";

export const ARTIFACT_COMMENTS: ArtifactComment[] = [
  {
    id: "c-1",
    artifactId: "a-1",
    parentId: null,
    author: "you",
    body: "Dropped needs to be in the public error enum, or callers cannot match on it.",
    createdAt: "2026-08-20T14:29:00Z",
  },
  {
    id: "c-2",
    artifactId: "a-1",
    parentId: "c-1",
    author: "builder@4",
    body: "Exported it and added the From impl. Re-pushed — the line you marked is the one that changed.",
    createdAt: "2026-08-20T14:31:00Z",
  },
];

export const SEND_TO_SESSION = "Send to session";
export const RESOLVE = "Resolve";
