# UI mockups

Which file is the design, and which files are history.

| File | Status |
| --- | --- |
| `Locus v2.dc.html` | **The design.** 29 views — the whole shell and every screen. |
| `AgentPanel.dc.html` | **The design.** The agent panel, embedded by Interact and by Plan → Converse. |
| `Locus UI mockups.html` | A byte-identical bundle of `Locus v2.dc.html` with fonts inlined. Opens with no server. No extra screens. |
| `design_handoff_locus_v2/` | **Superseded.** An earlier iteration. Its README describes the old rail and a nine-stage plan pipeline; `.specs` M0.6 was written from it. |
| `design_handoff_locus_desktop_ui/`, `Locus.dc.html` | v1, 14 screens. Historical. |
| `screenshots/`, `uploads/` | Renders and reference material. |

The `.dc.html` files are design references authored in HTML. Their runtime (`support.js`, `<x-dc>`,
`<sc-if>`, `<sc-for>`) is authoring scaffolding with no analogue in the app — recreate the screens as
SolidJS components in `apps/desktop`, never port the markup.

**Read [`../UI_MOCKUP_REVIEW.md`](../UI_MOCKUP_REVIEW.md) before working from these files.** It
carries the screen-by-screen contract, the navigation model, and the gaps against `.specs/` and the
code. `design_handoff_locus_v2/README.md` is the only prose description of a mockup in this
directory, and it describes the superseded one.
