use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use atlaspack::{pack_and_write, PackConfig};

/// Pack a folder of PNG sprites into one atlas image + JSON sheet.
///
/// Deterministic: same inputs and flags always produce the same atlas.
#[derive(Debug, Parser)]
#[command(name = "atlaspack", version, about)]
struct Cli {
    /// Directory containing `.png` sprites (searched recursively).
    input: PathBuf,

    /// Output atlas image path (PNG).
    #[arg(short, long, default_value = "atlas.png")]
    output: PathBuf,

    /// Output JSON path. Defaults to `<output>` with a `.json` extension.
    #[arg(short = 'j', long)]
    json: Option<PathBuf>,

    /// Empty pixels between sprites.
    #[arg(short, long, default_value_t = 2)]
    padding: u32,

    /// Maximum atlas width/height in pixels.
    #[arg(long, default_value_t = 4096)]
    max_size: u32,

    /// Do not force power-of-two atlas dimensions.
    #[arg(long)]
    no_pot: bool,
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> atlaspack::Result<()> {
    let cli = Cli::parse();

    let json_path = cli.json.unwrap_or_else(|| {
        let mut p = cli.output.clone();
        p.set_extension("json");
        p
    });

    let config = PackConfig {
        padding: cli.padding,
        max_size: cli.max_size,
        power_of_two: !cli.no_pot,
        image_name: cli
            .output
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("atlas.png")
            .to_string(),
    };

    let packed = pack_and_write(&cli.input, &cli.output, &json_path, config)?;

    eprintln!(
        "packed {} sprites → {} ({}x{}), {}",
        packed.json.frames.len(),
        cli.output.display(),
        packed.json.atlas.width,
        packed.json.atlas.height,
        json_path.display()
    );

    Ok(())
}
