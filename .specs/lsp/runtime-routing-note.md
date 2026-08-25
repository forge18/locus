# LSP runtime routing blocker

The CLI validates `locus lsp symbols <file>`, but end-to-end LSP routing remains intentionally skipped for this batch. `locusd` still uses `UnroutedVerbs`, and the host LSP supervisor/client stack is not present. Semantic-token full/delta/decorations remain an open implementation slice.
