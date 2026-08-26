import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import {
 ARTIFACTS,
 ARTIFACT_COMMENTS,
 REFERENCE_KINDS,
 REVIEW_KINDS,
 SELECTED_ARTIFACT_ID,
 UNIFIED_DIFF,
} from "../fixtures/artifacts";
import type { UnifiedRow } from "../fixtures/artifacts";
import type { Artifact, ArtifactComment } from "../types/agents";

export {
 ARTIFACT_LOCATOR,
 COMMENTS_TITLE,
 LIVE_COMMENT_NOTE,
 ONE_VIEWER_NOTE,
 REFERENCE_GROUP_LABEL,
 REFERENCE_KINDS,
 RESOLVE,
 REVIEW_KINDS,
 SEND_TO_SESSION,
} from "../fixtures/artifacts";
export type { UnifiedRow, UnifiedRowKind } from "../fixtures/artifacts";

/** Becomes: invoke("artifacts_list") */
export async function fetchArtifactsFromCore(): Promise<Artifact[]> {
 // No host (browser preview, tests) → fixture, not an error.
 if (!isTauri()) return ARTIFACTS;
 return invoke<Artifact[]>("artifacts_list");
}

/** Becomes: invoke("artifact_comments", { artifactId }) */
export async function fetchArtifactCommentsFromCore(
 artifactId: string,
): Promise<ArtifactComment[]> {
 // No host (browser preview, tests) → fixture, not an error.
 if (!isTauri())
  return ARTIFACT_COMMENTS.filter((c) => c.artifactId === artifactId);
 return invoke<ArtifactComment[]>("artifact_comments", { artifactId });
}

/** Fixture fallback until the Tauri runtime connects. */
export function useArtifacts(): Artifact[] {
 return ARTIFACTS;
}

/** Becomes: invoke("artifact", { id }) */
export function useArtifact(id: string): Artifact | null {
 return ARTIFACTS.find((a) => a.id === id) ?? null;
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultArtifactId(): string {
 return SELECTED_ARTIFACT_ID;
}

/** Becomes: invoke("artifact_diff", { id }) */
export function useUnifiedDiff(): UnifiedRow[] {
 return UNIFIED_DIFF;
}

/** Fixture fallback until the Tauri runtime connects. */
export function useArtifactComments(artifactId: string): ArtifactComment[] {
 return ARTIFACT_COMMENTS.filter((c) => c.artifactId === artifactId);
}

/** Resolve a host-owned blob path for the human media viewer. */
export function artifactMediaUrl(
 artifact: Artifact,
 fallback: string,
): string {
 if (!isTauri() || !artifact.blobPath) return fallback;
 return convertFileSrc(artifact.blobPath);
}

/**
 * Becomes: invoke("artifact_kinds")
 *
 * The review kinds a person looks at, and the reference kinds that stay out of
 * the inbox. The split is what keeps an agent's own scratch off the one surface
 * built to protect attention.
 */
export function useArtifactKinds(): {
 review: typeof REVIEW_KINDS;
 reference: typeof REFERENCE_KINDS;
} {
 return { review: REVIEW_KINDS, reference: REFERENCE_KINDS };
}
