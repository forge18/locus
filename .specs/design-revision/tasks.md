# design-revision — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Write the mockup review and the mockup-directory README | — | `test -f docs/UI_MOCKUP_REVIEW.md && test -f "docs/UI mockups for PLAN.md/README.md"` |
| 2 | Repoint every spec citing a deleted handoff directory | 1 | `! grep -rqi "$(printf '\\x64esign_\\x68andoff')" .specs PLAN.md` |
| 3 | Stop citing historical handoff directories as governing anywhere | 1 | `! grep -rqi "$(printf '\\x64esign_\\x68andoff')" .specs` |
| 4 | Add a "Superseded by" pointer line to every spec an M0.7 feature replaces | 3 | `grep -q "Superseded by" .specs/design-desktop/spec.md .specs/desktop-application-shell/spec.md .specs/dashboard-metrics/spec.md` |
| 5 | Purge the retired category names from the M0.7 specs | 3 | `! grep -rqiE "\b($(printf '\\x44evelop | \\x41utomate | \\x44ashboard'))\b" --exclude=tasks.md .specs/*-revision .specs/interact-sessions .specs/review-qa` |
| 6 | Record the 29-view inventory with one locator and one category each | 3 | `pnpm -C apps/desktop test -- nav/desktop-route-kinds` |
| 7 | State the seven-stage decision in `PLAN.md` §The planning module | 3 | `! grep -q "nine stages\|eight-step" PLAN.md` |
| 8 | State the new rail model in `PLAN.md` §Navigation and the Decisions table | 7 | `grep -q "Cross-Project" PLAN.md` |
| 9 | Move Goal off the canvas in `PLAN.md` and `.specs/workflow-canvas` | 8 | `! grep -qi "goal.*node\|node.*goal" PLAN.md .specs/workflow-canvas/spec.md` |
| 10 | Reconcile `PLAN.md` §Credentials and the ADR with Providers | 9 | `grep -q "keychain_reference" PLAN.md` |
| 11 | Correct the downgrade count in `PLAN.md` to the registry value | 10 | `cargo test -p locus-core --lib harness::registry` |
| 12 | Replace the fixed harness roster with Pi plus trusted user plugins | 11 | `grep -q "first-party.*Pi\|Pi.*first-party" PLAN.md .specs/harness-registry/spec.md` |
| 13 | Add the M0.7 milestone and its feature rows to `TODO.md` | 12 | `grep -q "Workshop scope reduction" TODO.md` |
| 14 | Add the M0.7 row to the `TODO.md` progress table and update the totals | 13 | `grep -qE '^\| \*\*M0\.7\*\*' TODO.md` |
