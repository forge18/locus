import { For, Show, createSignal, onMount } from "solid-js";
import {
  fetchCliTools,
  setCliToolEnabled,
  uploadCliTool,
  type CliTool,
} from "../../data/cli-tools";
import { dataProvider } from "../../data/provider";
import type { Envelope } from "../../data/envelope";
import { Button } from "../../ui/Button";
import { InlineError } from "../../ui/InlineError";
import { Input } from "../../ui/Input";

const CATEGORY_LABELS: Record<string, string> = {
  "source-control": "Source control",
  "search-files": "Search & files",
  rust: "Rust",
  database: "Database",
  "web-network": "Web & network",
};

export function CliToolsView() {
  const [catalog, setCatalog] = createSignal<Envelope<CliTool[]>>({
    status: "loading",
  });
  const [message, setMessage] = createSignal<string>();
  const [error, setError] = createSignal<string>();
  const [manifestFile, setManifestFile] = createSignal<File>();
  const [binaryFile, setBinaryFile] = createSignal<File>();
  const [manifestSignature, setManifestSignature] = createSignal("");
  const [binarySignature, setBinarySignature] = createSignal("");

  onMount(() => void fetchCliTools().then(setCatalog));

  const tools = () => {
    const state = catalog();
    return state.status === "ready" ? state.data : [];
  };
  const loadError = () => {
    const state = catalog();
    return state.status === "failed" ? state.error.message : "";
  };
  const replaceTool = (tool: CliTool) => {
    setCatalog({
      status: "ready",
      data: [
        ...tools().filter((candidate) => candidate.id !== tool.id),
        tool,
      ].sort(
        (left, right) =>
          left.category.localeCompare(right.category) ||
          left.name.localeCompare(right.name),
      ),
    });
  };
  const toggle = async (tool: CliTool) => {
    setError(undefined);
    const result = await setCliToolEnabled({
      id: tool.id,
      enabled: !tool.enabled,
    });
    if (result.status === "ready") {
      replaceTool(result.data);
      setMessage(
        `${result.data.name} ${result.data.enabled ? "enabled" : "disabled"}.`,
      );
    } else if (result.status === "failed") {
      setError(result.error.message);
    }
  };
  const upload = async () => {
    const manifest = manifestFile();
    const binary = binaryFile();
    if (!manifest || !binary || !manifestSignature() || !binarySignature()) {
      setError("Manifest, binary, and both Minisign signatures are required.");
      return;
    }
    setError(undefined);
    const [manifestBytes, binaryBytes] = await Promise.all([
      manifest.arrayBuffer(),
      binary.arrayBuffer(),
    ]);
    const result = await uploadCliTool({
      manifest: [...new Uint8Array(manifestBytes)],
      manifestSignature: manifestSignature(),
      binary: [...new Uint8Array(binaryBytes)],
      binarySignature: binarySignature(),
    });
    if (result.status === "ready") {
      replaceTool(result.data);
      setMessage(`${result.data.name} admitted to the catalog.`);
    } else if (result.status === "failed") {
      setError(result.error.message);
    }
  };

  return (
    <div
      class="workshop cli-tools"
      data-testid="cli-tools"
      data-live-state={dataProvider().kind}
    >
      <header class="ws-head">
        <div>
          <span class="ws-title">CLI tools</span>
          <p class="ws-note">
            Enabled tools are baked into the base image, not installed per run.
          </p>
        </div>
        <label class="cli-upload-label">
          <span>Upload manifest</span>
          <input
            type="file"
            data-testid="cli-manifest"
            onChange={(event) =>
              setManifestFile(event.currentTarget.files?.[0])
            }
          />
        </label>
        <label class="cli-upload-label">
          <span>Upload binary</span>
          <input
            type="file"
            data-testid="cli-binary"
            onChange={(event) => setBinaryFile(event.currentTarget.files?.[0])}
          />
        </label>
        <p class="cli-upload-note">
          Category, install, verify, and documentation metadata come from the
          signed manifest.
        </p>
        <Input
          value={manifestSignature()}
          aria-label="Manifest Minisign signature"
          placeholder="manifest signature"
          onInput={(event) => setManifestSignature(event.currentTarget.value)}
        />
        <Input
          value={binarySignature()}
          aria-label="Binary Minisign signature"
          placeholder="binary signature"
          onInput={(event) => setBinarySignature(event.currentTarget.value)}
        />
        <Button variant="primary" data-testid="cli-upload" onClick={upload}>
          Upload a CLI
        </Button>
      </header>
      <Show when={message()}>
        <p data-testid="cli-message">{message()}</p>
      </Show>
      <Show when={error() || loadError()}>
        <InlineError
          cause={error() ?? loadError()}
          next="Check the signed upload or Locus store connection."
        />
      </Show>
      <Show when={catalog().status === "loading"}>
        <p data-testid="cli-loading">Loading the CLI catalog…</p>
      </Show>
      <Show when={catalog().status === "empty"}>
        <p data-testid="cli-empty">No admitted CLI tools are available.</p>
      </Show>
      <For each={Object.keys(CATEGORY_LABELS)}>
        {(category) => (
          <section
            class="cli-category"
            data-testid={`cli-category-${category}`}
          >
            <h2>{CATEGORY_LABELS[category]}</h2>
            <For each={tools().filter((tool) => tool.category === category)}>
              {(tool) => (
                <label class="cli-tool-row">
                  <input
                    type="checkbox"
                    checked={tool.enabled}
                    onChange={() => void toggle(tool)}
                  />
                  <strong>{tool.name}</strong>
                  <span>v{tool.version}</span>
                  <small>
                    {tool.source} · {tool.verifyCommand}
                  </small>
                </label>
              )}
            </For>
          </section>
        )}
      </For>
    </div>
  );
}

export default CliToolsView;
