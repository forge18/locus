import { describe, expect, it } from "vitest";
import { allSource, declarations, read, rules } from "./css";

const type = read("styles/type.css");

const px = (v: string) => Number.parseFloat(v);

/** Resolve a `var(--t-x)` size back to the px the scale gives it. */
const scale = () =>
  Object.fromEntries(
    [...read("styles/tokens.css").matchAll(/(--t-[a-z-]+): *([\d.]+)px/g)].map(
      (m) => [m[1], px(m[2])],
    ),
  );

const sizeOf = (body: string) => {
  const token = body.match(/font-size:\s*var\((--t-[a-z-]+)\)/);
  return token ? scale()[token[1]] : Number.NaN;
};

describe("type-scale", () => {
  it("sets Inter as the body family and 400 as the body weight", () => {
    const base = rules(type).find((r) => r.selector.includes("body"))!;
    expect(base.body).toContain("font-family: var(--fs)");
    expect(base.body).toContain("font-weight: 400");
  });

  it("routes every mono use through --fm", () => {
    const mono = rules(type).find((r) => r.selector === ".mono")!;
    expect(mono.body).toContain("font-family: var(--fm)");
  });

  it("keeps semantic emphasis at the vendored 500 weight", () => {
    const emphasis = rules(type).find((r) => r.selector === "strong, b")!;
    expect(emphasis.body).toContain("font-weight: 500");
  });

  it("uses 500 as the only emphasis weight, bar the one named exception", () => {
    // .rail-badge is 700 by ruling in .specs/app-shell/spec.md: it is a 15px accent
    // pill with a numeral in it, not text. It is excised by name rather than by a
    // looser pattern, so a second one cannot appear beside it.
    for (const [file, contents] of allSource()) {
      const source =
        file === "shell/shell.css"
          ? contents.replace(/\.rail-badge \{[\s\S]*?\n\}/, "")
          : contents;
      for (const w of declarations(source, "font-weight").map((x) =>
        x.replace(/['"]/g, ""),
      )) {
        expect(
          ["400", "500", "normal", "inherit"],
          `${file}: unexpected weight ${w}`,
        ).toContain(w);
      }
    }
    const badge = read("shell/shell.css").match(
      /\.rail-badge \{[\s\S]*?\n\}/,
    )![0];
    expect(declarations(badge, "font-weight")).toEqual(["700"]);
  });

  it("keeps section labels at the smallest two steps, with .11-.12em tracking", () => {
    for (const sel of [".t-section", ".t-section-lg"]) {
      const r = rules(type).find((x) => x.selector === sel)!;
      const size = sizeOf(r.body);
      const track = px(r.body.match(/letter-spacing:\s*([\d.]+)em/)![1]);
      // The handoff's 9-10px band, lifted onto the scale's floor.
      expect(size).toBeGreaterThanOrEqual(11);
      expect(size).toBeLessThanOrEqual(13);
      expect(track).toBeGreaterThanOrEqual(0.1);
      expect(track).toBeLessThanOrEqual(0.12);
      expect(r.body).toContain("text-transform: uppercase");
    }
  });

  it("keeps each band inside its range on the lifted scale", () => {
    // The handoff's bands were 10.5-11, 11.5-12.5, 13-15 and 17-27. Every step is
    // lifted by about a quarter with an 11px floor, so the hierarchy is the one
    // that was drawn and the smallest of it is legible.
    const bands: Array<[string[], number, number]> = [
      [[".t-meta", ".t-meta-lg"], 13, 14],
      [[".t-body", ".t-row", ".t-row-lg"], 14, 15],
      [[".t-title", ".t-title-lg"], 16, 19],
      [[".t-metric", ".t-metric-lg"], 22, 34],
    ];
    for (const [selectors, lo, hi] of bands) {
      for (const sel of selectors) {
        const r = rules(type).find((x) => x.selector === sel);
        expect(r, `missing ${sel}`).toBeDefined();
        const size = sizeOf(r!.body);
        expect(size, `${sel} is ${size}px`).toBeGreaterThanOrEqual(lo);
        expect(size, `${sel} is ${size}px`).toBeLessThanOrEqual(hi);
      }
    }
  });

  it("has a floor: nothing on the scale is smaller than 11px", () => {
    for (const [token, size] of Object.entries(scale())) {
      expect(size, `${token} is ${size}px`).toBeGreaterThanOrEqual(11);
    }
  });

  it("reads every size from the scale, so the whole thing moves in one edit", () => {
    for (const [file, contents] of allSource()) {
      if (file === "styles/tokens.css") continue;
      expect(contents, `${file} hardcodes a font size`).not.toMatch(
        /font-size: *[\d.]+px/,
      );
    }
  });

  it("names no font family outside the two tokens", () => {
    // tokens.css defines them; fonts.css declares the faces they point at.
    const defines = ["styles/tokens.css", "assets/fonts/fonts.css"];
    for (const [file, contents] of allSource()) {
      if (defines.includes(file)) continue;
      for (const fam of declarations(contents, "font-family")) {
        // `inherit` is how a form control picks up the body font rather than the UA's.
        expect(fam, `${file}: ${fam}`).toMatch(/var\(--f[ms]\)|^inherit$/);
      }
    }
  });
});
