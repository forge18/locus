import type { Envelope } from "./envelope.ts";
import { dataProvider } from "./provider.ts";

export type StoreHealthStatus =
  | "not_configured"
  | "connecting"
  | "connected"
  | "unavailable";

export interface StoreHealth {
  status: StoreHealthStatus;
  message: string | null;
}

export function fetchStoreHealth(): Promise<Envelope<StoreHealth>> {
  return dataProvider().queryOne<StoreHealth>("store_health");
}
