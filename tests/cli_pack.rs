use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    env!("CARGO_BIN_EXE_atlaspack").into()
}

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sprites")
}

#[test]
fn cli_packs_fixture_sprites() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/out");
    let _ = fs::remove_dir_all(&out_dir);
    fs::create_dir_all(&out_dir).unwrap();

    let atlas = out_dir.join("atlas.png");
    let json = out_dir.join("atlas.json");

    let status = Command::new(bin())
        .args([
            fixtures().to_str().unwrap(),
            "-o",
            atlas.to_str().unwrap(),
            "-j",
            json.to_str().unwrap(),
            "-p",
            "2",
        ])
        .status()
        .expect("run atlaspack");

    assert!(status.success(), "atlaspack failed");
    assert!(atlas.is_file());
    assert!(json.is_file());

    let body = fs::read_to_string(&json).unwrap();
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["atlas"]["image"], "atlas.png");
    assert_eq!(v["atlas"]["padding"], 2);
    let frames = v["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 5);

    // Deterministic names, sorted.
    let names: Vec<&str> = frames.iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(names, vec!["coin", "heart", "hero", "slime", "tree"]);
}

#[test]
fn cli_is_deterministic() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/out_det");
    let _ = fs::remove_dir_all(&out_dir);
    fs::create_dir_all(&out_dir).unwrap();

    let run = |subdir: &str| {
        let dir = out_dir.join(subdir);
        fs::create_dir_all(&dir).unwrap();
        // Same output file name so JSON `atlas.image` matches.
        let atlas = dir.join("atlas.png");
        let json = dir.join("atlas.json");
        let status = Command::new(bin())
            .args([
                fixtures().to_str().unwrap(),
                "-o",
                atlas.to_str().unwrap(),
                "-j",
                json.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(status.success());
        (
            fs::read(&atlas).unwrap(),
            fs::read_to_string(&json).unwrap(),
        )
    };

    let (a_img, a_json) = run("run_a");
    let (b_img, b_json) = run("run_b");
    assert_eq!(a_img, b_img);
    assert_eq!(a_json, b_json);
}
