import { For, Show, createSignal, onMount } from "solid-js";
import { fetchProjects } from "../data/core";
import type { ProjectSummary } from "../types/core";

export interface PageProjectFilterProps {
  value: string | undefined;
  onChange: (projectId: string | undefined) => void;
  required?: boolean;
  allLabel?: string;
}

/** A page-owned project scope. It never writes to shell navigation or settings. */
export function PageProjectFilter(props: PageProjectFilterProps) {
  const [projects, setProjects] = createSignal<ProjectSummary[]>([]);
  const [error, setError] = createSignal<string | null>(null);

  onMount(() => {
    void fetchProjects().then((envelope) => {
      if (envelope.status === "ready") setProjects(envelope.data);
      else if (envelope.status === "failed") setError(envelope.error.message);
    });
  });

  return (
    <label class="page-project-filter" data-testid="page-project-filter">
      <span>Project</span>
      <select
        value={props.value ?? ""}
        aria-required={props.required ? "true" : undefined}
        onChange={(event) =>
          props.onChange(event.currentTarget.value || undefined)
        }
      >
        <Show when={props.required}>
          <option value="">Choose a project</option>
        </Show>
        <Show when={!props.required}>
          <option value="">{props.allLabel ?? "All projects"}</option>
        </Show>
        <For each={projects()}>
          {(project) => <option value={project.id}>{project.name}</option>}
        </For>
      </select>
      <Show when={error()}>
        <small role="alert">{error()}</small>
      </Show>
    </label>
  );
}
