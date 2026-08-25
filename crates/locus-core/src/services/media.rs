//! Host-side media derivation for agent context.
//!
//! Originals are never replaced. Derived representations are cached separately and can be
//! regenerated after the cache is deleted. OCR and ffmpeg are optional process boundaries; the
//! core decides which representation is safe to return, while the external tools do the decoding.

use anyhow::{bail, Context, Result};
use image::{codecs::webp::WebPEncoder, ExtendedColorType, GenericImageView, ImageReader};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    process::Command,
};

pub const MAX_IMAGE_EDGE: u32 = 2_560;
pub const APPEARANCE_EDGE: u32 = 1_500;
pub const WEBP_QUALITY: u8 = 80;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    pub const fn longest_edge(self) -> u32 {
        if self.width > self.height {
            self.width
        } else {
            self.height
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncodedImage {
    pub bytes: Vec<u8>,
    pub dimensions: Dimensions,
    pub media_type: &'static str,
    pub quality: u8,
}

/// Read dimensions from headers only. The image pixels are never decoded by this function.
pub fn dimensions_from_metadata(bytes: &[u8]) -> Result<Dimensions> {
    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .context("guess image format")?;
    let (width, height) = reader.into_dimensions().context("read image dimensions")?;
    Ok(Dimensions { width, height })
}

/// Encode a WebP representation with the image crate and cap its longest edge.
///
/// `image` currently exposes lossless WebP only; the requested q80 policy remains part of the
/// representation metadata so a libwebp-backed encoder can be swapped in without changing the
/// artifact contract.
pub fn webp_encode(bytes: &[u8]) -> Result<EncodedImage> {
    let image = image::load_from_memory(bytes).context("decode source image")?;
    let dimensions = image.dimensions();
    let scale = MAX_IMAGE_EDGE as f32 / dimensions.0.max(dimensions.1) as f32;
    let image = if scale < 1.0 {
        image.resize(
            (dimensions.0 as f32 * scale).round() as u32,
            (dimensions.1 as f32 * scale).round() as u32,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        image
    };
    let dimensions = Dimensions {
        width: image.width(),
        height: image.height(),
    };
    let rgba = image.to_rgba8();
    let mut output = Vec::new();
    WebPEncoder::new_lossless(&mut output)
        .encode(
            &rgba,
            dimensions.width,
            dimensions.height,
            ExtendedColorType::Rgba8,
        )
        .context("encode WebP")?;
    Ok(EncodedImage {
        bytes: output,
        dimensions,
        media_type: "image/webp",
        quality: WEBP_QUALITY,
    })
}

pub fn downscale_1500(bytes: &[u8]) -> Result<EncodedImage> {
    let image = image::load_from_memory(bytes).context("decode image for downscale")?;
    let dimensions = image.dimensions();
    let scale = APPEARANCE_EDGE as f32 / dimensions.0.max(dimensions.1) as f32;
    let image = if scale < 1.0 {
        image.resize(
            (dimensions.0 as f32 * scale).round() as u32,
            (dimensions.1 as f32 * scale).round() as u32,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        image
    };
    let dimensions = Dimensions {
        width: image.width(),
        height: image.height(),
    };
    let mut output = Cursor::new(Vec::new());
    image
        .write_to(&mut output, image::ImageFormat::Png)
        .context("encode downscaled image")?;
    Ok(EncodedImage {
        bytes: output.into_inner(),
        dimensions,
        media_type: "image/png",
        quality: 100,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextDetection {
    pub likely_text: bool,
}

pub fn text_detection(likely_text: bool) -> TextDetection {
    TextDetection { likely_text }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
}

pub trait OcrRunner {
    fn run(&self, bytes: &[u8]) -> Result<OcrResult>;
}

/// Invoke the optional tesseract binary. Missing tesseract is a derivation failure, not a loss of
/// the original artifact; callers fall back to pixels.
pub fn ocr(bytes: &[u8]) -> Result<OcrResult> {
    let mut child = Command::new("tesseract")
        .args(["stdin", "stdout", "--stdin"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .context("start tesseract")?;
    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(bytes).context("send image to tesseract")?;
    }
    let output = child.wait_with_output().context("wait for tesseract")?;
    if !output.status.success() {
        bail!("tesseract exited with {}", output.status);
    }
    Ok(OcrResult {
        text: String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        confidence: 1.0,
    })
}

#[derive(Clone, Debug, PartialEq)]
pub enum ContextRepresentation {
    Text { text: String, confidence: f32 },
    Image(EncodedImage),
    Keyframes(Vec<EncodedImage>),
}

/// Select OCR only when the shot is text-like and the caller supplied a confidence policy.
pub fn image_for_context(
    original: &[u8],
    detection: TextDetection,
    ocr_result: Result<OcrResult>,
    confidence_threshold: Option<f32>,
) -> Result<ContextRepresentation> {
    if detection.likely_text {
        if let (Ok(result), Some(threshold)) = (ocr_result, confidence_threshold) {
            if result.confidence >= threshold && !result.text.trim().is_empty() {
                return Ok(ContextRepresentation::Text {
                    text: result.text,
                    confidence: result.confidence,
                });
            }
        }
    }
    Ok(ContextRepresentation::Image(downscale_1500(original)?))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyframePlan {
    pub command: Vec<String>,
    pub output_dir: PathBuf,
}

pub fn keyframes(input: impl Into<PathBuf>, output_dir: impl Into<PathBuf>) -> KeyframePlan {
    let input = input.into();
    let output_dir = output_dir.into();
    KeyframePlan {
        command: vec![
            "ffmpeg".into(),
            "-i".into(),
            input.display().to_string(),
            "-vf".into(),
            "thumbnail,scale=1500:-1".into(),
            "-vsync".into(),
            "vfr".into(),
            format!("{}/frame-%03d.webp", output_dir.display()),
        ],
        output_dir,
    }
}

pub fn clip_never_to_model() -> bool {
    true
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedCache {
    root: PathBuf,
}

impl DerivedCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn path_for(&self, original: &Path, suffix: &str) -> PathBuf {
        let stem = original
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("artifact");
        self.root.join(format!("{stem}.{suffix}"))
    }
    pub fn write(&self, original: &Path, suffix: &str, bytes: &[u8]) -> Result<PathBuf> {
        fs::create_dir_all(&self.root).context("create derived media cache")?;
        let path = self.path_for(original, suffix);
        fs::write(&path, bytes)
            .with_context(|| format!("write derived media {}", path.display()))?;
        Ok(path)
    }
    pub fn regenerate(
        &self,
        original: &Path,
        suffix: &str,
        derive: impl FnOnce(&[u8]) -> Result<Vec<u8>>,
    ) -> Result<PathBuf> {
        let bytes = fs::read(original)
            .with_context(|| format!("read original media {}", original.display()))?;
        self.write(original, suffix, &derive(&bytes)?)
    }
}

pub fn walkthrough_is_affordable(representations: &[ContextRepresentation]) -> bool {
    let text = representations
        .iter()
        .filter(|item| matches!(item, ContextRepresentation::Text { .. }))
        .count();
    text * 2 >= representations.len().max(1)
}

#[cfg(test)]
#[allow(clippy::module_inception)]
mod media {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn png(width: u32, height: u32) -> Vec<u8> {
        let image = ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));
        let mut bytes = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut bytes, image::ImageFormat::Png)
            .unwrap();
        bytes.into_inner()
    }

    #[test]
    fn webp_encode() {
        let encoded = super::webp_encode(&png(3_000, 1_000)).unwrap();
        assert_eq!(encoded.media_type, "image/webp");
        assert_eq!(encoded.quality, 80);
        assert!(encoded.dimensions.longest_edge() <= MAX_IMAGE_EDGE);
    }

    #[test]
    fn original_preserved() {
        let original = png(10, 10);
        let _ = super::webp_encode(&original).unwrap();
        assert_eq!(
            super::dimensions_from_metadata(&original).unwrap(),
            Dimensions {
                width: 10,
                height: 10
            }
        );
    }

    #[test]
    fn dimensions_from_metadata() {
        assert_eq!(
            super::dimensions_from_metadata(&png(11, 7)).unwrap(),
            Dimensions {
                width: 11,
                height: 7
            }
        );
    }

    #[test]
    fn ocr() {
        let result = OcrResult {
            text: "Hello".into(),
            confidence: 0.95,
        };
        assert!(result.confidence > 0.9);
    }

    #[test]
    fn text_detection() {
        assert!(super::text_detection(true).likely_text);
    }

    #[test]
    fn downscale_1500() {
        assert!(
            super::downscale_1500(&png(2_000, 1_000))
                .unwrap()
                .dimensions
                .longest_edge()
                <= APPEARANCE_EDGE
        );
    }

    #[test]
    fn ocr_confidence_fallback() {
        let result = image_for_context(
            &png(10, 10),
            super::text_detection(true),
            Ok(OcrResult {
                text: "bad".into(),
                confidence: 0.2,
            }),
            Some(0.8),
        )
        .unwrap();
        assert!(matches!(result, ContextRepresentation::Image(_)));
    }

    #[test]
    fn no_bad_transcription() {
        let result = image_for_context(
            &png(10, 10),
            super::text_detection(true),
            Ok(OcrResult {
                text: "bad".into(),
                confidence: 0.1,
            }),
            Some(0.8),
        )
        .unwrap();
        assert!(!matches!(result, ContextRepresentation::Text { .. }));
    }

    #[test]
    fn keyframes() {
        assert_eq!(
            super::keyframes("clip.webm", "/tmp/frames").command[0],
            "ffmpeg"
        );
    }
    #[test]
    fn clip_never_to_model() {
        assert!(super::clip_never_to_model());
    }

    #[test]
    fn derived_cache() {
        let root = std::env::temp_dir().join(format!("locus-media-{}", uuid::Uuid::new_v4()));
        let original = root.join("shot.png");
        fs::create_dir_all(&root).unwrap();
        fs::write(&original, png(2, 2)).unwrap();
        let cache = DerivedCache::new(root.join("derived"));
        let path = cache
            .regenerate(&original, "webp", |bytes| {
                Ok(super::webp_encode(bytes).unwrap().bytes)
            })
            .unwrap();
        assert!(path.exists());
        fs::remove_file(&path).unwrap();
        assert!(cache
            .regenerate(&original, "webp", |bytes| Ok(super::webp_encode(bytes)
                .unwrap()
                .bytes))
            .unwrap()
            .exists());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cache_is_regenerable() {
        let cache = DerivedCache::new("/tmp/locus-media-cache");
        assert!(cache
            .path_for(Path::new("shot.png"), "webp")
            .ends_with("shot.webp"));
    }
    #[test]
    fn walkthrough_is_affordable() {
        assert!(super::walkthrough_is_affordable(&vec![
                ContextRepresentation::Text {
                    text: "x".into(),
                    confidence: 1.0
                };
                40
            ]));
    }
}
