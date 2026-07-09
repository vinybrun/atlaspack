//! Deterministic sprite atlas packer.
//!
//! Point at a folder of PNGs, get one atlas image and a JSON sheet.
//!
//! # Example
//!
//! ```no_run
//! use atlaspack::{pack_directory, PackConfig};
//!
//! let packed = pack_directory(
//!     std::path::Path::new("./sprites"),
//!     &PackConfig::default(),
//! )?;
//! println!("{} frames", packed.json.frames.len());
//! # Ok::<(), atlaspack::Error>(())
//! ```

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod error;
mod pack;

pub use error::Error;
pub use pack::{
    load_sprites, pack_and_write, pack_directory, pack_images, AtlasJson, AtlasMeta, Frame,
    PackConfig, PackedAtlas, Sprite,
};

/// Crate-level result type.
pub type Result<T> = std::result::Result<T, Error>;
