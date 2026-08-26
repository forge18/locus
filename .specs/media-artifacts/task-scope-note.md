# Media-artifacts task scope note

The host-side derivation primitives and the desktop Artifacts viewers are wired. `MemoryArtifactsFixture`
loads the core artifact list when running under Tauri, resolves host-owned blob paths with the Tauri
asset protocol, and keeps deterministic image/WebM fixtures for browser previews and tests. The
original recording remains the human-facing video source; derived keyframes stay agent-facing.

## Remaining integration boundary

- Task 13 is still only CLI allowlisting/parsing. No daemon route currently derives `--for-context` from
  a stored media artifact.
- No production caller currently runs `services::media` against stored artifacts, so OCR and ffmpeg
  integration remain outside this viewer task.
- The actual WebP encoder is the image crate's lossless encoder; `quality = 80` is metadata until a
  Linux-capable q80 encoder is selected.

## Evidence

- `cargo test -p locus-core media::` — derivation unit tests pass.
- `cargo test -p locus-cli artifact::for_context` — CLI parser test passes.
- `bash scripts/check-no-sips.sh` — passes.
- `pnpm -C apps/desktop test -- artifacts/media-viewers` — viewer source and element assertions pass.
- `pnpm -C apps/desktop build` — desktop production build passes.
- `cargo test -p locus-tauri --lib -- --nocapture` — Tauri command/config tests pass.
