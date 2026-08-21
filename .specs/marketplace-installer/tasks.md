# marketplace-installer — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | Install-method dispatch: brew, cargo, and the rest a manifest declares | — | `cargo test -p locus-core install::methods` |
| 2 | Bake an agent's tool set into `locus/agent-<hash>` | 1 | `cargo test -p locus-core install::bakes` |
| 3 | Run each manifest's `verify` during the build | 2 | `cargo test -p locus-core install::verifies` |
| 4 | A failing `verify` fails the build | 3 | `cargo test -p locus-core install::verify_failure_fails_build` |
| 5 | Assert a non-allowlisted tool is absent from the image | 2 | `cargo test -p locus-core install::allowlist_enforced_at_build` |
| 6 | Identical tool lists still share one image after baking | 2 | `cargo test -p locus-core install::shared_image` |
| 7 | A changed pin rebuilds; a changed prose body does not | 2 | `cargo test -p locus-core install::rebuild_triggers` |
| 8 | Inject the catalog line for each installed tool | 2 | `cargo test -p locus-core install::catalog_injected` |
| 9 | Bodies remain on demand via `locus tools docs` | 8 | `cargo test -p locus-cli tools::docs_on_demand` |
| 10 | Decide and document hosting, pinning and the trust model | — | `test -s .specs/marketplace-installer/TRUST-MODEL.md` |
| 11 | Decide curation versus selection, with the reasoning recorded | 10 | `grep -q 'curation\|selection' .specs/marketplace-installer/TRUST-MODEL.md` |
