import { createAvatar, type StyleMeta } from "@dicebear/core";
import * as dicebearCollection from "@dicebear/collection";
import { DEFAULT_BOT_AVATAR_STYLE } from "./setting";

/** The style definition shape accepted by DiceBear's core renderer. */
type CollectionStyle = Parameters<typeof createAvatar>[0];
type CollectionStyleId = keyof typeof dicebearCollection;

type CollectionStyleModule = CollectionStyle & {
  meta?: StyleMeta;
};

const COLLECTION_STYLES = dicebearCollection as unknown as Record<
  CollectionStyleId,
  CollectionStyleModule
>;

export type AvatarStyleId = CollectionStyleId;

function fallbackLabel(id: string): string {
  return id
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replace(/^./, (character) => character.toUpperCase());
}

function styleLabel(id: string, meta: StyleMeta): string {
  const title = meta.title?.trim() || fallbackLabel(id);
  if (id.endsWith("Neutral") && !/\bneutral\b/i.test(title)) {
    return `${title} Neutral`;
  }
  return title;
}

function styleMetadata(
  id: CollectionStyleId,
  style: CollectionStyleModule,
): AvatarStyleOption {
  const meta = style.meta;
  const creator = meta?.creator?.trim();
  const license = meta?.license?.name?.trim();
  const licenseUrl = meta?.license?.url?.trim();
  if (!creator || !license || !licenseUrl) {
    throw new Error(
      `DiceBear style ${id} is missing creator or license metadata`,
    );
  }
  return Object.freeze({
    id,
    label: styleLabel(id, meta ?? {}),
    creator,
    license,
    licenseUrl,
    style,
  });
}

/** Every style exported by the bundled DiceBear collection, with its attribution metadata. */
export interface AvatarStyleOption {
  id: AvatarStyleId;
  label: string;
  creator: string;
  license: string;
  licenseUrl: string;
  style: CollectionStyleModule;
}

export const AVATAR_STYLE_IDS: readonly AvatarStyleId[] = Object.freeze(
  (Object.keys(COLLECTION_STYLES) as AvatarStyleId[]).sort((left, right) =>
    left.localeCompare(right),
  ),
);

export const AVATAR_STYLES: readonly AvatarStyleOption[] = Object.freeze(
  AVATAR_STYLE_IDS.map((id) => styleMetadata(id, COLLECTION_STYLES[id])),
);

const STYLES_BY_ID = new Map(
  AVATAR_STYLES.map((style) => [style.id, style] as const),
);

/** Unknown or missing settings fail closed to the shipped Bottts default. */
export function normalizeAvatarStyle(
  value: string | null | undefined,
): AvatarStyleId {
  return STYLES_BY_ID.has(value as AvatarStyleId)
    ? (value as AvatarStyleId)
    : DEFAULT_BOT_AVATAR_STYLE;
}

export function avatarStyleOption(
  value: string | null | undefined,
): AvatarStyleOption {
  const option = STYLES_BY_ID.get(normalizeAvatarStyle(value));
  if (!option) throw new Error("DiceBear avatar style registry is empty");
  return option;
}

const avatarCache = new Map<AvatarStyleId, Map<string, string>>();

/**
 * Derive a transparent, deterministic avatar data URI. The cache is webview-memory only;
 * the style and seed remain the complete source of truth.
 */
export function deriveAvatar(
  style: string | null | undefined,
  seed: string,
): string {
  const styleId = normalizeAvatarStyle(style);
  let styleCache = avatarCache.get(styleId);
  if (!styleCache) {
    styleCache = new Map<string, string>();
    avatarCache.set(styleId, styleCache);
  }

  const cached = styleCache.get(seed);
  if (cached) return cached;

  const dataUri = createAvatar(COLLECTION_STYLES[styleId], {
    seed,
    backgroundColor: ["transparent"],
  }).toDataUri();
  styleCache.set(seed, dataUri);
  return dataUri;
}

/** Exposed for focused tests without exposing the cache itself. */
export function derivedAvatarCacheSize(): number {
  let size = 0;
  for (const entries of avatarCache.values()) size += entries.size;
  return size;
}
