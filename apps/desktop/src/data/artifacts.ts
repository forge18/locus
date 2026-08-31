import { convertFileSrc, isTauri } from "@tauri-apps/api/core";
import { REFERENCE_KINDS, REVIEW_KINDS } from "./demo/fixtures/artifacts";
import type { UnifiedRow } from "./demo/fixtures/artifacts";
import type { Artifact, ArtifactComment } from "../types/agents";
import { dataProvider } from "./provider";
import type { Envelope } from "./envelope";

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
} from "./demo/fixtures/artifacts";
export type { UnifiedRow, UnifiedRowKind } from "./demo/fixtures/artifacts";

/** Becomes: invoke("artifacts_list") */
export function fetchArtifactsFromCore(
 projectId: string,
): Promise<Envelope<Artifact[]>> {
 return dataProvider().query<Artifact>("artifacts_list", { projectId });
}

/** Becomes: invoke("artifact_comments", { projectId, artifactId }) */
export function fetchArtifactCommentsFromCore(
 projectId: string,
 artifactId: string,
): Promise<Envelope<ArtifactComment[]>> {
 return dataProvider().query<ArtifactComment>("artifact_comments", {
  projectId,
  artifactId,
 });
}

/** Fixture fallback until the Tauri runtime connects. */
export function useArtifacts(): Artifact[] {
 return dataProvider().read?.<Artifact[]>("artifacts_list") ?? [];
}

/** Becomes: invoke("artifact", { id }) */
export function useArtifact(id: string): Artifact | null {
 return dataProvider().read?.<Artifact | null>("artifact", { id }) ?? null;
}

/** Becomes: pane state, once the pane manager owns it. */
export function useDefaultArtifactId(): string {
 return dataProvider().read?.<string>("artifact_default_id") ?? "";
}

/** Becomes: invoke("artifact_diff", { id }) */
export function useUnifiedDiff(): UnifiedRow[] {
 return dataProvider().read?.<UnifiedRow[]>("artifact_diff") ?? [];
}

/** Fixture fallback until the Tauri runtime connects. */
export function useArtifactComments(artifactId: string): ArtifactComment[] {
 return (
  dataProvider().read?.<ArtifactComment[]>("artifact_comments", {
   artifactId,
  }) ?? []
 );
}

/** Resolve a host-owned blob path for the human media viewer. */
export function artifactMediaUrl(artifact: Artifact, fallback: string): string {
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
 return (
  dataProvider().read?.<{
   review: typeof REVIEW_KINDS;
   reference: typeof REFERENCE_KINDS;
  }>("artifact_kinds") ?? { review: [], reference: [] }
 );
}
