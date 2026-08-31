/**
 * The one seam every live data accessor reads through. A provider answers a
 * registered Tauri command with a typed `Envelope` — never a fixture, never a
 * silent empty. See .specs/desktop-data-integration/contract.md.
 *
 * This module imports neither fixtures nor the demo provider: a Tauri runtime
 * can never silently serve demo data through it. The demo provider lives in
 * `src/data/demo/` and is reachable only where a host explicitly selects it.
 */
import { invoke } from "@tauri-apps/api/core";
import { failed, ready, readyOne, type Envelope } from "./envelope";

export interface DataProvider {
 /** "live" reads the Rust store through registered commands; "demo" reads fixtures. */
 readonly kind: "live" | "demo";
 /** Query a list-returning command. An empty result is `empty`, not an empty `ready`. */
 query<T>(
  command: string,
  args?: Record<string, unknown>,
 ): Promise<Envelope<T[]>>;
 /** Query a single-value command. An absent row is `empty`. */
 queryOne<T>(
  command: string,
  args?: Record<string, unknown>,
 ): Promise<Envelope<T>>;
}

/** The provider a Tauri runtime bootstraps: every call crosses the real IPC boundary. */
export const liveProvider: DataProvider = {
 kind: "live",
 async query<T>(command: string, args?: Record<string, unknown>) {
  try {
   return ready((await invoke<T[]>(command, args)) ?? []);
  } catch (cause) {
   return failed(command, cause);
  }
 },
 async queryOne<T>(command: string, args?: Record<string, unknown>) {
  try {
   return readyOne(await invoke<T | null>(command, args));
  } catch (cause) {
   return failed(command, cause);
  }
 },
};

let active: DataProvider | undefined;

/**
 * Called once at bootstrap: `liveProvider` under Tauri, the demo provider only in
 * an explicit demo/test host. Reconfiguring is allowed for tests.
 */
export function configureDataProvider(provider: DataProvider): void {
 active = provider;
}

/**
 * Every accessor resolves its provider here. Unset is a bootstrap bug: fail loudly
 * rather than silently serving fixtures.
 */
export function dataProvider(): DataProvider {
 if (!active) {
  throw new Error(
   "data provider not configured — call configureDataProvider(liveProvider) at bootstrap (demo/test hosts select the demo provider explicitly)",
  );
 }
 return active;
}
