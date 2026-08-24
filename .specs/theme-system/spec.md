# theme-system

**Milestone** M0.6 · **Depends on** `design-desktop` · **Blocks** every desktop desktop surface.

## Purpose

Make Locus themeable without making each component invent its own palette. M0.6 ships both desktop Dark
and a cool-neutral Light theme. Later themes add a value set and fixtures; they do not change component
CSS, fixture data, or behavior.

## Governed by

- `DESIGN.md` §Visual rules
- `.specs/design-desktop/spec.md` §Shell and screen inventory
- `docs/UI_MOCKUP_REVIEW.md` §Navigation and §Screens

## Contract

### Token layers

1. **Theme values** exist only under `[data-theme]`.
2. **Semantic tokens** express role, not hue: `--surface-ground`, `--surface-chrome`,
   `--surface-raised`, `--surface-selected`, `--text-primary`, `--text-secondary`,
   `--text-muted`, `--border-subtle`, `--action-attention`, `--status-working`,
   `--status-success`, `--status-danger`, and `--data-1…3` / `--data-hi`.
3. **Component tokens** may map a component role to semantic tokens; components may not read a raw
   palette value or a different theme's selector.

The desktop names (`--bg`, `--sf`, `--ac`, `--ac2`, `--data-*`, `--ok`, `--bad`) remain compatibility
aliases during migration. New component styles use semantic roles.

### Shipped themes

| Role | Dark desktop | Light cool-neutral |
| --- | --- | --- |
| ground / chrome | `#1d2731` / `#151d25` | `#f3f6f8` / `#e8eef3` |
| raised / selected | `#22303c` / `#293947` | `#ffffff` / `#e3edf5` |
| primary / secondary text | `#eef2f6` / `rgba(238,242,246,.78)` | `#16212b` / `#405262` |
| attention | `#ffbb39` | `#9a5b00` |
| working | `#9184d9` | `#675bb0` |
| success / danger | `#68ad91` / `#df8a7d` | `#237250` / `#a7372d` |

The complete light data ramp and border/text-muted values are derived and checked in the contrast
matrix. Attention and working remain distinct in both themes; data magnitude never consumes either.

### Theme selection

The desktop root owns `data-theme`. Appearance offers **Dark** and **Light**, persists their stable
identifiers, and defaults safely to Dark for a missing or unknown value. Persistence stores only the
identifier, never computed colors. System preference is not an automatic third mode in M0.6.

### Accessibility and verification

Every installed theme must pass the same semantic contrast matrix and preserve focus, success, danger,
and working-state distinctions. Token lint rejects raw colors outside the theme-value source and assets.
Visual fixtures run once for each installed theme. Adding a later theme requires a value set and fixture
declaration only.

## Acceptance

1. Switching `data-theme` changes all shell, component, chart, diff, and focus colors through tokens.
2. No component CSS or TSX consumes a raw color value or a theme-specific selector.
3. Dark resolves the desktop values and Light resolves the cool-neutral values in the table.
4. Appearance persists Dark/Light across restart and an unknown identifier safely resolves to Dark.
5. The contrast and visual-fixture commands enumerate every installed theme.
6. Adding a test theme requires only token values and a fixture declaration; no component edit.
