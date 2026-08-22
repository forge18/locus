import type { JSX } from "solid-js";

export type ShellStateKind = "loading" | "empty" | "error";

export function ShellState(props: {
  kind: ShellStateKind;
  children: JSX.Element;
}) {
  return (
    <section data-testid={`shell-state-${props.kind}`} data-state={props.kind}>
      {props.children}
    </section>
  );
}
