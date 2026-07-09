use std::path::PathBuf;

/// Errors produced while loading or packing sprites.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No `.png` files were found under the given path.
    #[error("no PNG sprites found in {0}")]
    NoSprites(PathBuf),

    /// Sprites could not fit within the configured maximum atlas size.
    #[error("failed to pack sprites into {width}x{height} (try a larger --max-size)")]
    PackFailed {
        /// Attempted atlas width.
        width: u32,
        /// Attempted atlas height.
        height: u32,
    },

    /// A single sprite is larger than the maximum atlas dimensions.
    #[error("sprite '{name}' is larger than the atlas ({sw}x{sh} > {aw}x{ah})")]
    SpriteTooLarge {
        /// Sprite name.
        name: String,
        /// Sprite width in pixels.
        sw: u32,
        /// Sprite height in pixels.
        sh: u32,
        /// Atlas max width.
        aw: u32,
        /// Atlas max height.
        ah: u32,
    },

    /// Filesystem error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Image decode/encode error.
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),

    /// JSON serialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
