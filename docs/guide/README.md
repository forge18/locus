# Guide

`Locus-Operator-Guide.pdf` — the operator and analyst guide: internals, screens, plugin authoring,
workflows, operations. Compiled from `PLAN.md` and the feature contracts under `.specs/`.

`locus-operator-guide.html` is the source. All diagrams are inline SVG; no external assets.

Regenerate:

```sh
"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
  --headless --disable-gpu --print-to-pdf-no-header \
  --print-to-pdf=docs/guide/Locus-Operator-Guide.pdf \
  docs/guide/locus-operator-guide.html
```
