import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { GITHUB_WORK_ITEM_PROVIDER_FIXTURE } from "../../src/data/work-items";
import ManageView from "../../src/screens/manage/ManageView";

describe("automate/import-confirmation", () => {
  it("requires workflow selection and states the provider sync boundary", async () => {
    const { getByText, getByTestId } = render(() => (
      <ManageView
        workItemProviders={GITHUB_WORK_ITEM_PROVIDER_FIXTURE}
        projectId="00000000-0000-0000-0000-000000000002"
        workflowDefinitions={[
          {
            id: "00000000-0000-0000-0000-000000000001",
            name: "build-and-verify",
            version: 1,
          },
        ]}
      />
    ));
    await fireEvent.click(getByText("Import task"));
    expect(getByTestId("automate-import-workflow")).toBeTruthy();
    expect(getByText("build-and-verify · v1")).toBeTruthy();
    expect(getByTestId("automate-import-one-way").textContent).toContain(
      "Status and note synchronization is enabled",
    );
    expect(getByTestId("automate-import-kanban").textContent).toContain(
      "preview before local task creation",
    );
    await fireEvent.click(getByText("GitHub"));
    expect(
      getByTestId("automate-import-preview").getAttribute("data-provider"),
    ).toBe("github");
    expect(getByText("GitHub").getAttribute("aria-pressed")).toBe("true");
  });

  it("does not invent a provider when the registry is empty", async () => {
    const { getByText, getByTestId } = render(() => (
      <ManageView workItemProviders={[]} />
    ));
    await fireEvent.click(getByText("Import task"));
    expect(getByTestId("automate-import-no-provider").textContent).toContain(
      "No work-item plugin is configured",
    );
  });
});
