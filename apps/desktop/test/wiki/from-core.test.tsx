import { fireEvent, render } from "@solidjs/testing-library";
import { describe, expect, it } from "vitest";
import { WIKI_KIND_CHIPS } from "../../src/data/knowledge";
import { MemoryWikiFixture } from "../../src/screens/memory/MemoryFixtures";

describe("wiki/from-core", () => {
  it("renders the typed page, visible kinds, provenance, and ingest entry point", () => {
    const { getByTestId, getByText } = render(() => <MemoryWikiFixture />);
    const wiki = getByTestId("desktop-memory-wiki");
    expect(getByText("Ingest a document")).toBeTruthy();
    expect(getByText("Provenance")).toBeTruthy();
    expect(wiki.querySelectorAll("[data-kind]").length).toBe(
      WIKI_KIND_CHIPS.length,
    );
    expect(wiki.querySelector('[data-kind="overview"]')).toBeNull();
  });

  it("filters the page list by the selected kind", () => {
    const { getByTestId, getByText } = render(() => <MemoryWikiFixture />);
    const filter = getByTestId("wiki-kind-filter");
    fireEvent.click(filter.querySelector('[data-kind="entity"]')!);

    expect(getByText("Entities 42")).toBeTruthy();
    expect(getByTestId("wiki-pages").querySelectorAll("[data-page-kind]")).toHaveLength(2);
    expect(getByTestId("wiki-pages").textContent).toContain("credential broker");
    expect(getByTestId("wiki-pages").textContent).not.toContain("Locus architecture");
  });
});
