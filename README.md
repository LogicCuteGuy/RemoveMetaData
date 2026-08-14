# RemoveMetaData

A fast, cross-platform tool to strip AI-generated metadata from images, documents, video, and audio — and inject custom metadata fields.

Built in **Rust**. No Python required.

[![CI](https://github.com/FreedomTrails/RemoveMetaData/actions/workflows/ci.yml/badge.svg)](https://github.com/FreedomTrails/RemoveMetaData/actions/workflows/ci.yml)
[![Pages](https://github.com/FreedomTrails/RemoveMetaData/actions/workflows/pages.yml/badge.svg)](https://freedomtrails.github.io/RemoveMetaData/)

## Features

- **Remove AI Metadata** — Strips C2PA manifests, EXIF/XMP, ID3 tags, ISO BMFF atoms, XMP in PDFs, and OOXML properties from 160+ AI generators
- **Inject Custom Metadata** — Set Author, Source, Title, Description, Credit, Keywords, Category, and Comments per file
- **12 Format Support** — PNG, JPG, WebP, GIF, MP4, MOV, MP3, FLAC, PDF, DOCX, XLSX, PPTX, EXE
- **Batch Processing** — Drop entire folders, process recursively
- **Non-Destructive** — Output to a separate folder by default
- **Rename Cleanup** — Removes "ChatGPT Image" / "DALL-E" / "Midjourney" prefixes
- **Dry Run** — Preview changes without modifying files

## Project Structure

```
RemoveMetaData/
├── engine/          Core metadata processing library (all formats)
├── cli/             CLI binary (clap args, batch mode)
├── gui/             Native GUI (eframe/egui, dark theme, drag-and-drop)
├── web/             WASM bridge for browser use
├── index.html       Web frontend (list/grid view, thumbnails, per-file editor)
└── Cargo.toml       Workspace root
```

## Installation

### Download Prebuilt Binaries

Grab the latest release for your platform from [Releases](../../releases).

### Build from Source

Requires [Rust](https://rustup.rs/) (1.70+).

```bash
git clone https://github.com/FreedomTrails/RemoveMetaData.git
cd RemoveMetaData

# Build CLI + GUI
cargo build --release

# Binaries in target/release/
#   RemoveMetaData.exe       — CLI
#   RemoveMetaData-GUI.exe   — GUI
```

#### Build the Web (WASM)

Requires [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/):

```bash
cargo install wasm-pack
wasm-pack build web --target web --out-dir ..
```

This outputs `removemetadata_wasm.js` + `.wasm` to the project root for use with `index.html`.

## Usage

### GUI

```bash
./target/release/RemoveMetaData-GUI
```

- Drag and drop files or folders
- Toggle between **List** and **Grid** views (grid shows image thumbnails)
- Click any file to open the **property editor** — set all 8 metadata fields per file
- Global Author/Source defaults apply to all files unless overridden

### CLI

```bash
# Process a directory
RemoveMetaData.exe ./images -o ./clean

# Custom metadata
RemoveMetaData.exe ./images -o ./clean --author "John Doe" --source "My Team"

# Preview without modifying
RemoveMetaData.exe ./images --dry-run

# Recursive + keep original filenames
RemoveMetaData.exe ./images -o ./clean --recursive --no-rename
```

### Web (WASM)

Open `index.html` in any browser — no server needed.

- Drag files onto the page
- Toggle **☰ List** / **⊞ Grid** views (grid shows image thumbnails)
- Click any file to open the **popup editor** — override all 8 metadata fields per file
- Process and download individually or as a ZIP
- Deploys automatically to [GitHub Pages](https://freedomtrails.github.io/RemoveMetaData/) on push to `main`

## Supported Formats

| Format | What Gets Removed | How |
|--------|-------------------|-----|
| **PNG** | C2PA (`caBX`), AI tEXt/zTXt/iTXt chunks | Chunk walker |
| **JPG** | EXIF, XMP, ICC with AI keywords | Segment scanner |
| **WebP** | EXIF, XMP RIFF chunks | Chunk scanner |
| **GIF** | AI metadata comment blocks | Block scanner |
| **MP4/MOV** | `udta`/`meta`/`ilst` atoms with AI keywords | ISO BMFF atom walker |
| **MP3** | ID3v2 tags with AI keywords | Tag scanner |
| **FLAC** | Vorbis Comment blocks with AI keywords | Block walker |
| **PDF** | XMP `<x:xmpmeta>` blocks, /Author, /Title, /Subject, /Keywords | XML scanner |
| **DOCX/XLSX/PPTX** | `docProps/core.xml` + `app.xml` properties | ZIP replacement |
| **EXE** | Embedded AI keyword byte sequences | Byte scanner |

## Metadata Fields

| Field | CLI Flag | Web Editor | Description |
|-------|----------|------------|-------------|
| Author | `--author` | ✓ Per-file | Creator name |
| Source | `--source` | ✓ Per-file | Organization or source |
| Title | — | ✓ Per-file | Document/image title |
| Description | — | ✓ Per-file | Content description |
| Credit | — | ✓ Per-file | Attribution credit |
| Keywords | — | ✓ Per-file | Search keywords |
| Category | — | ✓ Per-file | Content category |
| Comments | — | ✓ Per-file | Freeform notes |

## Supported AI Generators

| Category | Platforms |
|----------|-----------|
| **OpenAI** | ChatGPT, DALL-E, GPT-Image |
| **Stable Diffusion** | SDXL, ComfyUI, A1111, Stability AI |
| **Midjourney** | MJ-v4/v5/v6 |
| **Adobe** | Firefly, Sensei |
| **Google** | Imagen, Gemini, Parti |
| **Meta** | Make-A-Scene, Emu |
| **Microsoft** | Bing Image Creator, Copilot, Designer |
| **Chinese AI** | 通义万相, 文心一格, LiblibAI, 海艺AI, 吐司AI, Vega AI |
| **Other** | Leonardo AI, Playground AI, Craiyon, NightCafe, DreamStudio, Ideogram, Flux, Kandinsky |
| **Generic** | Generative AI, AI Generated, AI Art, Text-to-Image, C2PA, Content Credentials |

## CI/CD

- **CI** (`ci.yml`) — Runs on every push/PR: `cargo fmt`, `clippy`, build, test across Windows/macOS/Linux, plus WASM build verification
- **Release** (`release.yml`) — Tag-triggered: builds CLI+GUI for 5 targets (Win x64, Linux x64/arm64, macOS x64/arm64), creates GitHub Release
- **Pages** (`pages.yml`) — Auto-deploys web version to GitHub Pages on push to `main`

## License

MIT License
