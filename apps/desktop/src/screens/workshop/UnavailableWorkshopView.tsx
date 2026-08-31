import { InlineError } from "../../ui/InlineError";

export function UnavailableWorkshopView(props: {
  route: string;
  label: string;
  command: string;
}) {
  return (
    <div
      data-testid={`workshop-${props.route}`}
      class="workshop-unavailable"
      data-live-state="unavailable"
    >
      <h1>{props.label}</h1>
      <InlineError
        cause={`${props.label} is unavailable`}
        next={`${props.command} has no persisted desktop contract yet.`}
      />
    </div>
  );
}
