# Ship checklist (applied / remaining)

Practices drawn from the [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html),
[Rust API Guidelines](https://rust-lang.github.io/api-guidelines/),
[CLI book packaging chapter](https://rust-cli.github.io/book/tutorial/packaging.html),
and common first-release checklists.

## Done in-repo

- [x] Dual license MIT OR Apache-2.0 + license files + README boilerplate
- [x] Full `Cargo.toml` metadata (description, authors, repository, homepage, documentation, keywords, categories, readme, rust-version)
- [x] README with install, usage, example JSON, status, contributing
- [x] CHANGELOG (Keep a Changelog)
- [x] `#![forbid(unsafe_code)]` + `#![warn(missing_docs)]` + rustdoc example
- [x] CI: fmt, clippy, test, package, MSRV 1.88
- [x] Tests (unit + CLI integration)
- [x] `exclude` for build artifacts / local notes

## Per release

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo package --list          # only intended files
cargo publish --dry-run
# then: git tag vX.Y.Z && cargo publish && gh release create
```

## crates.io (optional, permanent)

```bash
# https://crates.io → log in with GitHub → verify email → create token
cargo login
cargo publish
```

Publish is permanent (yank ≠ delete). Dry-run first.

## Announce once (optional)

> `atlaspack` — folder of PNGs → atlas.png + JSON. Deterministic, engine-agnostic, v1 done.  
> https://github.com/vinybrun/atlaspack · `cargo install atlaspack`
