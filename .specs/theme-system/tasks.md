# theme-system — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Define v2 Dark values under `[data-theme="dark"]` and preserve compatibility aliases during migration | — | `pnpm -C apps/desktop test -- theme/dark-token-contract` |
| 2 | Define the approved cool-neutral Light values and contrast matrix under `[data-theme="light"]` | 1 | `pnpm -C apps/desktop test -- theme/light-token-contract && bash apps/desktop/scripts/check-contrast.sh` |
| 3 | Map component and screen styles from palette tokens to semantic roles | 1 | `bash apps/desktop/scripts/check-theme-token-boundary.sh` |
| 4 | Add `--ac2` working-state and `--data-*` magnitude roles without reusing `--ac` | 1 | `pnpm -C apps/desktop test -- theme/accent-role-separation` |
| 5 | Add the Appearance Dark/Light selector, persist its identifier, and fall back safely for an unknown value | 1,2 | `pnpm -C apps/desktop test -- theme/preference-fallback` |
| 6 | Render the v2 fixture inventory in both shipped themes | 2,3,4,5 | `pnpm -C apps/desktop test -- theme/fixtures` |
| 7 | Run contrast and visual fixtures for every installed theme in CI | 2,6 | `pnpm -C apps/desktop test -- theme/all-installed && bash apps/desktop/scripts/check-contrast.sh` |
| 8 | Prove a test theme can be registered with values and a fixture declaration only | 3,5,7 | `pnpm -C apps/desktop test -- theme/registers-without-component-changes` |
