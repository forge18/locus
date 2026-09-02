import { For, Match, Show, Switch, createEffect, createSignal } from "solid-js";
import {
  fetchProjectCapabilityPolicy,
  saveProjectCapabilityPolicy,
  type CapabilityPolicies,
  type CapabilityPolicy,
} from "../../data/core";
import type { Envelope } from "../../data/envelope";
import { PageProjectFilter } from "../PageProjectFilter";
import { Button } from "../../ui/Button";
import { InlineError } from "../../ui/InlineError";

const POLICY_KEYS = ["cliTools", "commands", "skills"] as const;
type PolicyKey = (typeof POLICY_KEYS)[number];

const LABELS: Record<PolicyKey, string> = {
  cliTools: "CLI tools",
  commands: "Commands",
  skills: "Skills",
};

const deferred = (): CapabilityPolicy => "defer_to_project";
const emptyPolicies = (): CapabilityPolicies => ({
  cliTools: deferred(),
  commands: deferred(),
  skills: deferred(),
});

function allowedValues(policy: CapabilityPolicy): string {
  return typeof policy === "string" ? "" : policy.allow_only.join("\n");
}

function isDeferred(policy: CapabilityPolicy): boolean {
  return policy === "defer_to_project";
}

export function CapabilityPolicyPanel() {
  const [projectId, setProjectId] = createSignal<string>();
  const [policyEnvelope, setPolicyEnvelope] = createSignal<
    Envelope<{ revision: number; policies: CapabilityPolicies }>
  >({ status: "empty" });
  const [draft, setDraft] = createSignal<CapabilityPolicies>(emptyPolicies());
  const [saving, setSaving] = createSignal(false);
  const [message, setMessage] = createSignal<string | null>(null);

  createEffect(() => {
    const id = projectId();
    if (!id) {
      setPolicyEnvelope({ status: "empty" });
      setDraft(emptyPolicies());
      return;
    }
    setPolicyEnvelope({ status: "loading" });
    void fetchProjectCapabilityPolicy(id).then((envelope) => {
      setPolicyEnvelope(envelope);
      if (envelope.status === "ready") setDraft(envelope.data.policies);
    });
  });

  const policyError = () => {
    const envelope = policyEnvelope();
    return envelope.status === "failed" ? envelope.error.message : "";
  };

  function updatePolicy(key: PolicyKey, mode: string, values: string) {
    const next =
      mode === "inherit"
        ? deferred()
        : {
            allow_only: values
              .split("\n")
              .map((value) => value.trim())
              .filter(Boolean),
          };
    setDraft((current) => ({ ...current, [key]: next }));
  }

  async function save() {
    const id = projectId();
    if (!id) return;
    setSaving(true);
    setMessage(null);
    const envelope = await saveProjectCapabilityPolicy(id, draft());
    if (envelope.status === "ready") {
      setPolicyEnvelope(envelope);
      setMessage(`Policy revision ${envelope.data.revision} saved.`);
    } else if (envelope.status === "failed") {
      setMessage(envelope.error.message);
    }
    setSaving(false);
  }

  return (
    <section
      class="capability-policy-panel"
      data-testid="capability-policy-panel"
    >
      <header>
        <div>
          <h2>Capability policy</h2>
          <p>
            Project policy is the ceiling. Agent and workflow restrictions can
            only narrow it.
          </p>
        </div>
        <PageProjectFilter
          value={projectId()}
          required
          onChange={setProjectId}
        />
      </header>
      <Switch>
        <Match when={policyEnvelope().status === "loading"}>
          <p>Loading policy…</p>
        </Match>
        <Match when={policyEnvelope().status === "empty"}>
          <p>Choose a project to edit its policy.</p>
        </Match>
        <Match when={policyEnvelope().status === "failed"}>
          <InlineError
            cause={policyError()}
            next="The project capability policy could not be loaded."
          />
        </Match>
        <Match when={policyEnvelope().status === "ready"}>
          <div class="capability-policy-fields">
            <For each={POLICY_KEYS}>
              {(key) => (
                <label>
                  <span>{LABELS[key]}</span>
                  <select
                    value={isDeferred(draft()[key]) ? "inherit" : "allow"}
                    onChange={(event) =>
                      updatePolicy(
                        key,
                        event.currentTarget.value,
                        allowedValues(draft()[key]),
                      )
                    }
                  >
                    <option value="inherit">Defer to project catalog</option>
                    <option value="allow">Allow only</option>
                  </select>
                  <Show when={!isDeferred(draft()[key])}>
                    <textarea
                      value={allowedValues(draft()[key])}
                      placeholder="one capability per line"
                      onInput={(event) =>
                        updatePolicy(key, "allow", event.currentTarget.value)
                      }
                    />
                  </Show>
                </label>
              )}
            </For>
          </div>
          <Button
            variant="primary"
            disabled={saving()}
            onClick={() => void save()}
          >
            {saving() ? "Saving…" : "Save policy"}
          </Button>
        </Match>
      </Switch>
      <Show when={message()}>
        <p role="status">{message()}</p>
      </Show>
    </section>
  );
}
