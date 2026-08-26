# LSP runtime routing task

The runtime-routing slice is implemented. `locusd` authenticates each run from host-owned registration files, authorizes the LSP lease separately, and returns only a pinned descriptor; `locus` executes the leased descriptor inside the authenticated run's `/workspace`. Host editor panes use a supervised project server, share panes, replay documents after restart, publish project-filtered diagnostics, and mount through the shared CodeMirror editor surfaces.

The boundary is deliberate: `locusd` never answers an agent's LSP request from a host checkout. Absolute lease paths and runs without the LSP capability are rejected. Project descriptor pins can be persisted through the Tauri Store-backed commands and are validated before activation; imported descriptors remain disabled until explicitly pinned. Focused and full-suite evidence lives in the LSP, daemon, CLI, editor, project, and serial core tests.
