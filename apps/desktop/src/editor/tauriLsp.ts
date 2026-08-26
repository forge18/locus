import { Channel, invoke } from "@tauri-apps/api/core";
import type { HostLspSupervisor, LspDiagnostics } from "./lsp";

interface AttachResponse {
  projectRoot: string;
  paneId: string;
  descriptorId: string;
}

export interface TauriLspOptions {
  projectRoot: string;
  paneId: string;
  filePath: string;
  /** Optional persisted project identity used to hydrate descriptor pins before attach. */
  projectId?: string;
  onDiagnostics?: (diagnostics: LspDiagnostics) => void;
}

function response(id: number, result: unknown): string {
  return JSON.stringify({ jsonrpc: "2.0", id, result });
}

function failure(id: number, error: unknown): string {
  return JSON.stringify({
    jsonrpc: "2.0",
    id,
    error: {
      code: -32603,
      message: error instanceof Error ? error.message : String(error),
    },
  });
}

/** Attach one editor pane to the Tauri host's project-shared LSP process. */
export async function attachTauriLsp(
  options: TauriLspOptions,
): Promise<
  HostLspSupervisor & { descriptorId: string; dispose: () => Promise<void> }
> {
  if (options.projectId) {
    await invoke("lsp_load_project_descriptors", {
      projectRoot: options.projectRoot,
      projectId: options.projectId,
    });
  }
  const attached = await invoke<AttachResponse>("lsp_attach", {
    request: {
      projectRoot: options.projectRoot,
      paneId: options.paneId,
      filePath: options.filePath,
    },
  });
  const handlers = new Set<(message: string) => void>();
  let disposed = false;
  let generation = 0;
  const diagnostics = new Channel<LspDiagnostics>();
  diagnostics.onmessage = (value) => {
    if (!disposed) options.onDiagnostics?.(value);
  };
  let subscriptionId: number;
  try {
    subscriptionId = await invoke<number>("lsp_diagnostics_subscribe", {
      projectRoot: attached.projectRoot,
      channel: diagnostics,
    });
  } catch (error) {
    diagnostics.onmessage = () => undefined;
    await invoke("lsp_detach", {
      projectRoot: attached.projectRoot,
      paneId: attached.paneId,
    }).catch(() => undefined);
    throw error;
  }

  const publish = (message: string) => {
    for (const handler of handlers) handler(message);
  };
  const send = (message: string) => {
    if (disposed) return;
    let parsed: {
      id?: number | string | null;
      method?: string;
      params?: unknown;
    };
    try {
      parsed = JSON.parse(message) as typeof parsed;
    } catch {
      return;
    }
    if (!parsed.method) return;
    let params = parsed.params ?? null;
    if (
      parsed.method === "initialize" &&
      typeof params === "object" &&
      params !== null
    ) {
      const rootUri = `file://${encodeURI(attached.projectRoot)}`;
      params = {
        ...(params as Record<string, unknown>),
        rootUri,
        workspaceFolders: [{ uri: rootUri, name: "workspace" }],
      };
    }
    const requestGeneration = generation;
    if ("id" in parsed && parsed.id !== undefined) {
      void invoke("lsp_request", {
        projectRoot: attached.projectRoot,
        method: parsed.method,
        params,
      })
        .then((result) => {
          if (!disposed && requestGeneration === generation) {
            publish(response(Number(parsed.id), result));
          }
        })
        .catch((error: unknown) => {
          if (!disposed && requestGeneration === generation) {
            publish(failure(Number(parsed.id), error));
          }
        });
    } else {
      void invoke("lsp_notify", {
        projectRoot: attached.projectRoot,
        method: parsed.method,
        params,
      }).catch(() => undefined);
    }
  };

  return {
    descriptorId: attached.descriptorId,
    send,
    subscribe: (handler) => void handlers.add(handler),
    unsubscribe: (handler) => void handlers.delete(handler),
    dispose: async () => {
      if (disposed) return;
      disposed = true;
      generation += 1;
      handlers.clear();
      diagnostics.onmessage = () => undefined;
      try {
        await invoke("lsp_diagnostics_unsubscribe", { subscriptionId });
      } finally {
        await invoke("lsp_detach", {
          projectRoot: attached.projectRoot,
          paneId: attached.paneId,
        });
      }
    },
  };
}
