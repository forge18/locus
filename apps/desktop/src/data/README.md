# data

One accessor per data set. **Screens import from here, never from `src/fixtures/`.**

Production accessors read the configured provider. `App.tsx` explicitly selects
`liveProvider`; the test bootstrap explicitly selects `demoProvider`. A provider failure
stays a typed failed envelope, and a live provider never reads demo rows.

Demo fixture adapters live under `src/data/demo/`. The static guard rejects direct fixture
imports and fixture screens from production source, so component tests must opt into the
demo bootstrap instead of relying on browser or Tauri detection.
