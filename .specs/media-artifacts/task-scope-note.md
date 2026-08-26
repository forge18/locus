# Media-artifacts task scope note

The host-side derivation primitives and the desktop viewer fixture are present, but item 16 remains
partial and is skipped for now.

## Deferred blockers

- Task 13 is only CLI allowlisting/parsing. No daemon route currently derives `--for-context` from a
  stored media artifact.
- The Tauri artifact IPC returns metadata/derived text but has no media blob URL or keyframe source;
  the viewer fixture uses a placeholder WebP and an empty video element.
- No production caller connects `services::media` to artifact persistence. OCR and ffmpeg paths are
  therefore not exercised against stored artifacts.
- The actual WebP encoder is the image crate's lossless encoder; `quality = 80` is metadata until a
  Linux-capable q80 encoder is selected.

## Evidence

- `cargo test -p locus-core media::` — derivation unit tests pass.
- `cargo test -p locus-cli artifact::for_context` — CLI parser test passes.
- `bash scripts/check-no-sips.sh` — passes.
- `pnpm -C apps/desktop test -- artifacts/media-viewers` — structural viewer test passes.
