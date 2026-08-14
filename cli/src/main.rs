use clap::Parser;
use removemetadata_engine as engine;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "RemoveMetaData",
    version,
    about = "Remove AI metadata from PNG, JPG, WebP, MP4, PDF, DOCX, MP3, and more.",
    long_about = "A fast, cross-platform tool to strip AI-generated metadata (ChatGPT, DALL-E, Midjourney, etc.) and inject custom author/source tags."
)]
struct Cli {
    /// Input file or directory
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output directory (default: out/)
    #[arg(short, long, default_value = "out")]
    output: PathBuf,

    /// Custom author tag
    #[arg(short = 'A', long, default_value = engine::DEFAULT_AUTHOR)]
    author: String,

    /// Custom source tag
    #[arg(short = 'S', long, default_value = engine::DEFAULT_SOURCE)]
    source: String,

    /// Dry run — show what would be done without writing
    #[arg(long)]
    dry_run: bool,

    /// Don't rename files (remove ChatGPT/DALL-E prefix)
    #[arg(long)]
    no_rename: bool,

    /// Recursive directory processing
    #[arg(short, long)]
    recursive: bool,
}

fn process_one(
    path: &Path,
    output_dir: &Path,
    author: &str,
    source: &str,
    dry_run: bool,
    rename: bool,
) {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let display_name = name.clone();

    match fs::read(path) {
        Ok(data) => {
            let file_type = engine::detect_file_type(&name, &data);
            if file_type.is_empty() {
                println!("  [SKIP] Unsupported: {display_name}");
                return;
            }
            println!("  [TYPE] {file_type} — {display_name}");

            let meta = engine::Metadata::new(author, source);
            match engine::process_file(&name, &data, &meta) {
                Some(result) => {
                    let out_name = if rename {
                        engine::clean_filename(&name)
                    } else {
                        name.clone()
                    };
                    let out_path = output_dir.join(&out_name);

                    if !dry_run {
                        let _ = fs::create_dir_all(output_dir);
                        if let Err(e) = fs::write(&out_path, &result.output) {
                            println!("  [ERR]  {e}");
                            return;
                        }
                        println!("  [OUT]  {}", out_path.display());
                    }
                    println!("  [OK]   Removed {} AI metadata section(s)", result.removed);
                }
                None => {
                    println!("  [SKIP] No processor for type: {file_type}");
                }
            }
        }
        Err(e) => println!("  [ERR]  Cannot read {display_name}: {e}"),
    }
}

fn main() {
    let cli = Cli::parse();
    let output_dir = &cli.output;

    println!(
        "RemoveMetaData v{} — Rust Edition\n",
        env!("CARGO_PKG_VERSION")
    );

    for input_path in &cli.input {
        if input_path.is_file() {
            println!("[FILE] {}", input_path.display());
            process_one(
                input_path,
                output_dir,
                &cli.author,
                &cli.source,
                cli.dry_run,
                !cli.no_rename,
            );
        } else if input_path.is_dir() {
            let walker = if cli.recursive {
                WalkDir::new(input_path).max_depth(5)
            } else {
                WalkDir::new(input_path).max_depth(1)
            };
            let mut count = 0;
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if engine::is_supported(&name) {
                        println!("[FILE] {}", entry.path().display());
                        process_one(
                            entry.path(),
                            output_dir,
                            &cli.author,
                            &cli.source,
                            cli.dry_run,
                            !cli.no_rename,
                        );
                        count += 1;
                    }
                }
            }
            println!(
                "[DONE] Processed {count} file(s) from {}",
                input_path.display()
            );
        } else {
            println!("[SKIP] Not found: {}", input_path.display());
        }
    }

    if !cli.dry_run {
        println!("\n[DONE] Output in: {}", output_dir.display());
    } else {
        println!("\n[DRY RUN] No files were written.");
    }
}
