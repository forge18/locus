import type { Envelope } from "./envelope";
import { dataProvider } from "./provider";

export interface CliTool {
  id: string;
  name: string;
  version: string;
  category: string;
  enabled: boolean;
  source: "builtin" | "uploaded";
  binarySha256: string | null;
  installCommand: string;
  verifyCommand: string;
  documentationUrl: string | null;
  lastRebuiltAt: string | null;
}

export interface CliToolEnableRequest {
  id: string;
  enabled: boolean;
}

export interface CliToolUploadRequest {
  manifest: number[];
  manifestSignature: string;
  binary: number[];
  binarySignature: string;
}

export function fetchCliTools(): Promise<Envelope<CliTool[]>> {
  return dataProvider().query<CliTool>("cli_tools_list");
}

export function setCliToolEnabled(
  request: CliToolEnableRequest,
): Promise<Envelope<CliTool>> {
  return dataProvider().queryOne<CliTool>("cli_tool_enabled_set", { request });
}

export function uploadCliTool(
  request: CliToolUploadRequest,
): Promise<Envelope<CliTool>> {
  return dataProvider().queryOne<CliTool>("cli_tool_upload", { request });
}
