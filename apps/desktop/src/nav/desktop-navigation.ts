import {
  formatDesktopLocator,
  resolveDesktopLocator,
  type DesktopNavTarget,
  type DesktopRouteId,
} from "./desktop-locator";

/** The sole desktop navigation boundary used by rails, palette results, and direct locators. */
export function navigateDesktop(locator: string): DesktopNavTarget {
  return resolveDesktopLocator(locator);
}

export function destinationDesktop(route: DesktopRouteId, project?: string): string {
  return formatDesktopLocator(route, project);
}
