import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

export interface SearchResult {
  kind: "task" | "wiki" | "run" | "code";
  project: string;
  label: string;
  locator: string;
  score: number;
}

export function searchAll(query: string): Promise<Envelope<SearchResult[]>> {
  return dataProvider().query<SearchResult>("search_all", { query });
}
