# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-07-09

### Added

- CLI: pack a folder of PNGs into `atlas.png` + JSON frame sheet
- Library API: `pack_directory`, `pack_images`, `pack_and_write`, `load_sprites`
- Deterministic shelf packing with padding and power-of-two atlas sizes
- Recursive PNG discovery; stable sprite names from relative paths

[1.0.0]: https://github.com/vinybrun/atlaspack/releases/tag/v1.0.0
