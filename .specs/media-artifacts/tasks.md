# media-artifacts — tasks

| # | Task | Deps | verify |
| --- | --- | --- | --- |
| 1 | WebP q80 encode with a 2560 longest-edge cap, via the `image` crate | — | `cargo test -p locus-core media::webp_encode` |
| 2 | Assert the original is never overwritten by a derived form | 1 | `cargo test -p locus-core media::original_preserved` |
| 3 | Dimensions read from metadata without decoding | 1 | `cargo test -p locus-core media::dimensions_from_metadata` |
| 4 | `tesseract` OCR path | — | `cargo test -p locus-core media::ocr` |
| 5 | Text-detection heuristic deciding OCR versus downscale | 4 | `cargo test -p locus-core media::text_detection` |
| 6 | 1500px downscale for appearance shots | 1 | `cargo test -p locus-core media::downscale_1500` |
| 7 | OCR confidence threshold with fallback to the image | 4 | `cargo test -p locus-core media::ocr_confidence_fallback` |
| 8 | Assert a low-confidence transcription is never returned as fact | 7 | `cargo test -p locus-core media::no_bad_transcription` |
| 9 | `ffmpeg` keyframe extraction | — | `cargo test -p locus-core media::keyframes` |
| 10 | Assert a clip is never returned to a model | 9 | `cargo test -p locus-core media::clip_never_to_model` |
| 11 | Derived-representation cache beside the original | 2 | `cargo test -p locus-core media::derived_cache` |
| 12 | Deleting the cache loses nothing; forms regenerate | 11 | `cargo test -p locus-core media::cache_is_regenerable` |
| 13 | `locus artifact get --for-context` routing by kind and content | 5,9 | `cargo test -p locus-cli artifact::for_context` |
| 14 | Walkthrough over forty screenshots is text-dominated | 13 | `cargo test -p locus-core media::walkthrough_is_affordable` |
| 15 | Assert no `sips` dependency; encoding works on Linux | 1,9 | `bash scripts/check-no-sips.sh` |
| 16 | Wire the Artifacts screen's image and recording viewers | 13 | `pnpm -C apps/desktop test -- artifacts/media-viewers` |
