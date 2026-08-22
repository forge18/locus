import {
  formatV2Locator,
  resolveV2Locator,
  type V2NavTarget,
  type V2RouteId,
} from "./v2-locator";

/** The sole v2 navigation boundary used by rails, palette results, and direct locators. */
export function navigateV2(locator: string): V2NavTarget {
  return resolveV2Locator(locator);
}

export function destinationV2(route: V2RouteId, project?: string): string {
  return formatV2Locator(route, project);
}
