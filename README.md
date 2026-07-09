# atlaspack

[![CI](https://github.com/vinybrun/atlaspack/actions/workflows/ci.yml/badge.svg)](https://github.com/vinybrun/atlaspack/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/atlaspack.svg)](https://crates.io/crates/atlaspack)
[![docs.rs](https://docs.rs/atlaspack/badge.svg)](https://docs.rs/atlaspack)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

**Tiny deterministic sprite atlas packer for game dev.**

Point it at a folder of PNGs. Get one atlas image and a JSON sheet. No engine dependency, no editor, no account — a finished CLI you can install and forget about.

```bash
atlaspack ./sprites -o atlas.png
# writes atlas.png + atlas.json
```

## Install

```bash
# From crates.io (after publish)
cargo install atlaspack

# From this repository
cargo install --git https://github.com/vinybrun/atlaspack

# From a local checkout
cargo install --path .
```

Requires a Rust toolchain (MSRV **1.88**).

## Usage

```text
atlaspack <INPUT> [OPTIONS]

Arguments:
  <INPUT>  Directory of .png sprites (recursive)

Options:
  -o, --output <OUTPUT>      Atlas PNG path [default: atlas.png]
  -j, --json <JSON>          JSON path [default: <output>.json]
  -p, --padding <PADDING>    Gap between sprites in px [default: 2]
      --max-size <MAX_SIZE>  Max atlas edge [default: 4096]
      --no-pot               Allow non power-of-two sizes
  -h, --help
  -V, --version
```

### Example

```bash
atlaspack tests/fixtures/sprites -o /tmp/atlas.png -p 2
cat /tmp/atlas.json
```

### JSON shape

```json
{
  "atlas": {
    "image": "atlas.png",
    "width": 64,
    "height": 64,
    "padding": 2
  },
  "frames": [
    { "name": "coin", "x": 2, "y": 2, "w": 8, "h": 8 },
    { "name": "hero", "x": 12, "y": 2, "w": 16, "h": 16 }
  ]
}
```

- **name** — path relative to the input dir, without `.png` (stable, sorted).
- **x, y, w, h** — pixel rect in the atlas (top-left origin).

Drop the JSON into Bevy, macroquad, ggez, Godot, or your own loader. This tool does not care.

## Guarantees

| Promise | Detail |
|--------|--------|
| Deterministic | Same files + flags → same pixels and JSON |
| Finished | v1.0 scope is locked; no roadmap treadmill |
| Engine-agnostic | Only depends on PNG + JSON |
| Low maintenance | Shelf packer, not a research project |

## Library

```rust,no_run
use atlaspack::{pack_directory, PackConfig};

let packed = pack_directory(
    std::path::Path::new("./sprites"),
    &PackConfig::default(),
)?;
// packed.image : RgbaImage
// packed.json  : AtlasJson
# Ok::<(), atlaspack::Error>(())
```

API docs: <https://docs.rs/atlaspack>

## Non-goals

- GUI / live preview  
- Bevy (or any engine) plugin  
- Texture compression (KTX2/Basis)  
- Rotated packing, multipack, trimming  

Those are fine projects for someone else. This crate stays small.

## Status

**Maintenance: best-effort.** Scope is intentionally locked at v1. Bug fixes welcome; large feature expansions are out of scope (fork freely).

## Contributing

Issues and small PRs are welcome. Please run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo test` before opening a PR.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
