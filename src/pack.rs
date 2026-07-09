use std::fs;
use std::path::{Path, PathBuf};

use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use serde::Serialize;
use walkdir::WalkDir;

use crate::{Error, Result};

/// One input sprite after load.
#[derive(Clone)]
pub struct Sprite {
    /// Stable name (relative path without `.png`).
    pub name: String,
    /// Pixel data.
    pub image: RgbaImage,
}

impl Sprite {
    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.image.width()
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.image.height()
    }
}

/// Where a sprite landed in the atlas.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Frame {
    /// Sprite name (matches [`Sprite::name`]).
    pub name: String,
    /// Left edge in the atlas (pixels).
    pub x: u32,
    /// Top edge in the atlas (pixels).
    pub y: u32,
    /// Width in pixels.
    pub w: u32,
    /// Height in pixels.
    pub h: u32,
}

/// JSON sidecar written next to the atlas PNG.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AtlasJson {
    /// Atlas-level metadata.
    pub atlas: AtlasMeta,
    /// Per-sprite rectangles, sorted by name.
    pub frames: Vec<Frame>,
}

/// Metadata about the packed atlas image.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AtlasMeta {
    /// File name of the atlas image (not a full path).
    pub image: String,
    /// Atlas width in pixels.
    pub width: u32,
    /// Atlas height in pixels.
    pub height: u32,
    /// Padding used between sprites.
    pub padding: u32,
}

/// Result of packing: image pixels plus JSON metadata.
pub struct PackedAtlas {
    /// Combined atlas image (RGBA).
    pub image: RgbaImage,
    /// Frame sheet for loaders.
    pub json: AtlasJson,
}

/// Packer options.
#[derive(Debug, Clone)]
pub struct PackConfig {
    /// Pixels of empty space between sprites.
    pub padding: u32,
    /// Max atlas width/height. Packer grows in power-of-two steps up to this.
    pub max_size: u32,
    /// Force power-of-two atlas dimensions.
    pub power_of_two: bool,
    /// File name stored in JSON `atlas.image` (not a path).
    pub image_name: String,
}

impl Default for PackConfig {
    fn default() -> Self {
        Self {
            padding: 2,
            max_size: 4096,
            power_of_two: true,
            image_name: "atlas.png".into(),
        }
    }
}

/// Load all `.png` files under `dir` (recursive).
///
/// Names are relative paths with `/` separators and without the `.png` suffix,
/// sorted lexicographically for deterministic packing.
pub fn load_sprites(dir: &Path) -> Result<Vec<Sprite>> {
    if !dir.is_dir() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("not a directory: {}", dir.display()),
        )));
    }

    let mut paths: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .map(|e| e.into_path())
        .filter(|p| {
            p.is_file()
                && p.extension()
                    .and_then(|x| x.to_str())
                    .is_some_and(|x| x.eq_ignore_ascii_case("png"))
        })
        .collect();

    paths.sort();

    let mut sprites = Vec::with_capacity(paths.len());
    for path in paths {
        let rel = path.strip_prefix(dir).unwrap_or(&path).with_extension("");
        let name = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        let dyn_img = image::open(&path)?;
        let image = dyn_img.into_rgba8();
        sprites.push(Sprite { name, image });
    }

    if sprites.is_empty() {
        return Err(Error::NoSprites(dir.to_path_buf()));
    }

    // Deterministic order: sort by name (paths already sorted, but be explicit).
    sprites.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(sprites)
}

/// Pack sprites already in memory into a single atlas.
///
/// Output is **deterministic**: the same sprites and config always produce the
/// same pixel layout and JSON (frames are sorted by name).
pub fn pack_images(sprites: &[Sprite], config: &PackConfig) -> Result<PackedAtlas> {
    if sprites.is_empty() {
        return Err(Error::NoSprites(PathBuf::from("<memory>")));
    }

    // Sort largest-first for better shelf packing; record original names.
    let mut order: Vec<usize> = (0..sprites.len()).collect();
    order.sort_by(|&a, &b| {
        let aa = sprites[a].width() * sprites[a].height();
        let bb = sprites[b].width() * sprites[b].height();
        bb.cmp(&aa)
            .then_with(|| sprites[a].name.cmp(&sprites[b].name))
    });

    let mut size = initial_size(sprites, config)?;
    loop {
        if size > config.max_size {
            return Err(Error::PackFailed {
                width: config.max_size,
                height: config.max_size,
            });
        }

        match try_pack(sprites, &order, size, size, config.padding) {
            Some(frames) => {
                let image = blit(sprites, &frames, size, size);
                let json = AtlasJson {
                    atlas: AtlasMeta {
                        image: config.image_name.clone(),
                        width: size,
                        height: size,
                        padding: config.padding,
                    },
                    frames,
                };
                return Ok(PackedAtlas { image, json });
            }
            None => {
                size = next_size(size, config.power_of_two, config.max_size);
            }
        }
    }
}

/// Load PNGs from `input_dir` and pack them.
pub fn pack_directory(input_dir: &Path, config: &PackConfig) -> Result<PackedAtlas> {
    let sprites = load_sprites(input_dir)?;
    pack_images(&sprites, config)
}

/// Pack sprites from `input_dir` and write the atlas image and JSON sheet.
///
/// Parent directories for the outputs are created as needed. The JSON `atlas.image`
/// field is set from `output_image`'s file name.
pub fn pack_and_write(
    input_dir: &Path,
    output_image: &Path,
    output_json: &Path,
    mut config: PackConfig,
) -> Result<PackedAtlas> {
    if let Some(name) = output_image.file_name().and_then(|s| s.to_str()) {
        config.image_name = name.to_string();
    }

    let packed = pack_directory(input_dir, &config)?;

    if let Some(parent) = output_image.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Some(parent) = output_json.parent() {
        fs::create_dir_all(parent)?;
    }

    DynamicImage::ImageRgba8(packed.image.clone()).save(output_image)?;
    let json = serde_json::to_string_pretty(&packed.json)?;
    fs::write(output_json, json)?;
    Ok(packed)
}

fn initial_size(sprites: &[Sprite], config: &PackConfig) -> Result<u32> {
    let pad = config.padding;
    let mut min_side = 1u32;
    for s in sprites {
        let need_w = s.width().saturating_add(pad);
        let need_h = s.height().saturating_add(pad);
        if need_w > config.max_size || need_h > config.max_size {
            return Err(Error::SpriteTooLarge {
                name: s.name.clone(),
                sw: s.width(),
                sh: s.height(),
                aw: config.max_size,
                ah: config.max_size,
            });
        }
        min_side = min_side.max(need_w).max(need_h);
    }

    // Area lower bound.
    let area: u64 = sprites
        .iter()
        .map(|s| {
            let w = u64::from(s.width().saturating_add(pad));
            let h = u64::from(s.height().saturating_add(pad));
            w * h
        })
        .sum();
    let area_side = (area as f64).sqrt().ceil() as u32;
    let mut size = min_side.max(area_side).max(16);

    if config.power_of_two {
        size = size.next_power_of_two();
    }
    Ok(size.min(config.max_size))
}

fn next_size(current: u32, power_of_two: bool, max: u32) -> u32 {
    if power_of_two {
        current.saturating_mul(2).min(max).max(current + 1)
    } else {
        // Grow by ~25% when not constrained to PoT.
        let grown = current.saturating_add((current / 4).max(16));
        grown.min(max).max(current + 1)
    }
}

/// Shelf packer: left-to-right, new shelf when the row is full.
fn try_pack(
    sprites: &[Sprite],
    order: &[usize],
    width: u32,
    height: u32,
    padding: u32,
) -> Option<Vec<Frame>> {
    let mut frames = Vec::with_capacity(order.len());
    let mut x = padding;
    let mut y = padding;
    let mut shelf_h = 0u32;

    for &idx in order {
        let s = &sprites[idx];
        let w = s.width();
        let h = s.height();
        let step_x = w.saturating_add(padding);

        if w > width.saturating_sub(padding) || h > height.saturating_sub(padding) {
            return None;
        }

        if x > padding && x.saturating_add(w) > width.saturating_sub(padding) {
            // new shelf
            x = padding;
            y = y.saturating_add(shelf_h).saturating_add(padding);
            shelf_h = 0;
        }

        if y.saturating_add(h) > height.saturating_sub(padding) {
            return None;
        }
        if x.saturating_add(w) > width.saturating_sub(padding) {
            return None;
        }

        frames.push(Frame {
            name: s.name.clone(),
            x,
            y,
            w,
            h,
        });

        x = x.saturating_add(step_x);
        shelf_h = shelf_h.max(h);
    }

    // Stable JSON: sort frames by name.
    frames.sort_by(|a, b| a.name.cmp(&b.name));
    Some(frames)
}

fn blit(sprites: &[Sprite], frames: &[Frame], width: u32, height: u32) -> RgbaImage {
    let mut atlas: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba([0, 0, 0, 0]));
    let by_name: std::collections::BTreeMap<&str, &Sprite> =
        sprites.iter().map(|s| (s.name.as_str(), s)).collect();

    for frame in frames {
        let sprite = by_name[frame.name.as_str()];
        for row in 0..frame.h {
            for col in 0..frame.w {
                let px = *sprite.image.get_pixel(col, row);
                atlas.put_pixel(frame.x + col, frame.y + row, px);
            }
        }
    }
    atlas
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn solid(name: &str, w: u32, h: u32, c: [u8; 4]) -> Sprite {
        Sprite {
            name: name.into(),
            image: ImageBuffer::from_pixel(w, h, Rgba(c)),
        }
    }

    #[test]
    fn packs_deterministically() {
        let sprites = vec![
            solid("b", 8, 8, [1, 0, 0, 255]),
            solid("a", 4, 4, [0, 1, 0, 255]),
            solid("c", 6, 10, [0, 0, 1, 255]),
        ];
        let cfg = PackConfig {
            padding: 1,
            max_size: 256,
            power_of_two: true,
            image_name: "atlas.png".into(),
        };
        let a = pack_images(&sprites, &cfg).unwrap();
        let b = pack_images(&sprites, &cfg).unwrap();
        assert_eq!(a.json, b.json);
        assert_eq!(a.image.into_raw(), b.image.into_raw());
    }

    #[test]
    fn frames_cover_all_sprites() {
        let sprites = vec![
            solid("hero", 16, 16, [255, 0, 0, 255]),
            solid("coin", 8, 8, [255, 215, 0, 255]),
        ];
        let packed = pack_images(&sprites, &PackConfig::default()).unwrap();
        assert_eq!(packed.json.frames.len(), 2);
        let names: Vec<_> = packed.json.frames.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["coin", "hero"]);
    }
}
