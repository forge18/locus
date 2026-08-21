# media-artifacts

**Milestone** M3.5 · **Depends on** `artifacts`, `locus-browse`

## Purpose

Two representations, because a human and a model want opposite things. Media is **stored once for you
and derived on demand for a model** — and the derivation is not a compression convenience, it is the
difference between an artifact that costs tokens to say less and one that says what it means.

## Governed by

- PLAN.md §Two representations: one you look at, one the agent reads
- PLAN.md §Artifacts — media has a retention policy; text does not

## Contract

| | Stored | Agent-facing |
| --- | --- | --- |
| `image` | WebP q80, longest edge capped at 2560 | **OCR text if the shot carries text**; otherwise downscaled to 1500px |
| `recording` | WebM as the browser produced it, capped by duration | extracted keyframes, **never the clip** |
| `diff` · `plan` · `walkthrough` | text in Postgres | the same text |

**Text is cheaper, searchable and quotable; pixels are none of those.** An error dialog, a terminal
capture, or a failing assertion is **text wearing a screenshot's clothes**, so `--for-context` OCRs it
and returns characters. Only appearance — a layout, a rendering bug, a diagram — justifies pixels, and
1500px on the longest edge carries all of it. Past that you pay tokens for detail no model uses.

**Four rules that keep this from going wrong:**
- **The stored copy is the original.** Derived representations are cached beside it and regenerable;
  nothing overwrites the artifact a human will open.
- **OCR is lossy on tables and low-resolution text.** When it looks wrong, the agent gets the image
  instead — **a bad transcription asserted as fact is worse than the pixels it saved.**
- **Dimensions are metadata.** Deciding how to handle a shot never requires loading it.
- **Encoding is host-side and in Rust** — the `image` crate for resize and WebP, `ffmpeg` for keyframes,
  `tesseract` for OCR. `local-dx`'s `sips` is macOS-only and Locus ships on Linux too.

**This is also why the walkthrough is affordable.** A session that produced forty screenshots inlines
forty OCR blocks and a handful of images, not forty megabytes.

## Acceptance

1. A stored image is WebP q80 with its longest edge at most 2560, and is never overwritten by a derived
   form.
2. `locus artifact get --for-context` on a text-bearing screenshot returns **characters, not bytes**.
3. The same call on a layout screenshot returns a 1500px image, not OCR.
4. A recording's agent-facing form is keyframes; the clip is never returned to a model.
5. Low-confidence OCR falls back to the image rather than asserting a bad transcription.
6. Dimensions are readable from metadata without decoding the file.
7. Derived forms are cached and regenerable — deleting the cache loses nothing.
8. A walkthrough over forty screenshots is dominated by text, and its size is asserted.
9. Encoding works on Linux as well as macOS — no `sips` dependency.

## Open

- The OCR-confidence threshold for falling back to the image. It is the one number that decides between
  a wrong fact and a token cost, and PLAN.md gives the rule but not the value.
