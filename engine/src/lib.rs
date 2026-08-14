/*!
Core metadata processing engine for RemoveMetaData.
Supports PNG, JPG, WebP, GIF, MP4, PDF, DOCX/XLSX/PPTX, MP3, FLAC, EXE/DLL.
100% Rust — no Python, no Pyodide.
*/

use std::io::Read as _;

// ─── Constants ────────────────────────────────────────────────────────────────
pub const DEFAULT_AUTHOR: &str = "LogicCuteGuy";
pub const DEFAULT_SOURCE: &str = "LogicCuteGuy";

/// Metadata fields that can be injected into processed files.
#[derive(Clone, Debug, Default)]
pub struct Metadata {
    pub author: String,
    pub source: String,
    pub title: String,
    pub description: String,
    pub credit: String,
    pub keywords: String,
    pub category: String,
    pub comments: String,
}

impl Metadata {
    pub fn new(author: &str, source: &str) -> Self {
        Self {
            author: author.to_string(),
            source: source.to_string(),
            ..Default::default()
        }
    }
}

pub const SUPPORTED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tif", "mp3", "flac", "wav", "ogg", "mp4",
    "mov", "avi", "mkv", "pdf", "docx", "xlsx", "pptx", "exe", "dll",
];

/// 160+ AI keywords from the original Python engine
const AI_KEYWORDS: &[&str] = &[
    "ChatGPT",
    "DALL-E",
    "DALL_E",
    "DallE",
    "OpenAI",
    "openai",
    "chatgpt",
    "dall-e",
    "dall_e",
    "dalle",
    "gpt-image",
    "gpt_image",
    "gptimage",
    "DALL·E",
    "dall·e",
    "Stable Diffusion",
    "stable-diffusion",
    "stable_diffusion",
    "stablediffusion",
    "Stability AI",
    "stability-ai",
    "StableDiffusion",
    "SDXL",
    "SD 1.5",
    "SD 2.1",
    "ComfyUI",
    "comfyui",
    "A1111",
    "AUTOMATIC1111",
    "Midjourney",
    "midjourney",
    "MIDJOURNEY",
    "Adobe Firefly",
    "firefly",
    "Firefly",
    "FIREFLY",
    "Adobe Sensei",
    "Adobe GenAI",
    "Imagen",
    "imagen",
    "Imagen 2",
    "Imagen 3",
    "Google AI",
    "Google GenAI",
    "Gemini",
    "Make-A-Scene",
    "Make-A-Video",
    "Emu",
    "Meta AI",
    "Llama Gen",
    "Bing Image Creator",
    "DALL-E 3",
    "Designer",
    "Microsoft Designer",
    "Copilot",
    "Amazon Titan",
    "Bedrock",
    "Leonardo AI",
    "leonardo.ai",
    "Leonardo",
    "Playground AI",
    "Craiyon",
    "NightCafe",
    "DreamStudio",
    "Runway",
    "Pika",
    "Sora",
    "通义万相",
    "Tongyi Wanxiang",
    "文心一格",
    "ERNIE ViLG",
    "LiblibAI",
    "海艺AI",
    "SeaArt",
    "像素蛋糕",
    "吐司AI",
    "6pen",
    "MewXAI",
    "Vega AI",
    "Ideogram",
    "Flux",
    "Kandinsky",
    "Disco Diffusion",
    "Wombo",
    "Lensa",
    "Remini",
    "Topaz",
    "Generative AI",
    "GenAI",
    "genai",
    "GENERATIVE",
    "AI Generated",
    "ai-generated",
    "ai_generated",
    "AI Art",
    "ai-art",
    "ai_art",
    "Machine Learning",
    "Neural Network",
    "Deep Learning",
    "Text-to-Image",
    "text-to-image",
    "txt2img",
    "trainedAlgorithmicMedia",
    "synthetic media",
    "c2pa",
    "C2PA",
    "content credentials",
    "ContentCredentials",
    "content-credentials",
    "Juggernaut",
    "DreamShaper",
    "Realistic Vision",
    "Deliberate",
    "Epic Diffusion",
    "OpenJourney",
    "Anything Diffusion",
    "Waifu Diffusion",
];

// ─── File Type Detection ──────────────────────────────────────────────────────

/// Detect file type from magic bytes, falling back to extension.
pub fn detect_file_type(name: &str, data: &[u8]) -> &'static str {
    if data.len() >= 8 && &data[..8] == b"\x89PNG\r\n\x1a\n" {
        return "png";
    }
    if data.len() >= 3 && &data[..3] == b"\xff\xd8\xff" {
        return "jpg";
    }
    if data.len() >= 6 && (&data[..6] == b"GIF87a" || &data[..6] == b"GIF89a") {
        return "gif";
    }
    if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return "webp";
    }
    if data.len() >= 4 && (&data[..4] == b"II\x2a\x00" || &data[..4] == b"MM\x00\x2a") {
        return "tiff";
    }
    if data.len() >= 4 && &data[..4] == b"fLaC" {
        return "flac";
    }
    if data.len() >= 4 && &data[..4] == b"%PDF" {
        return "pdf";
    }
    if data.len() >= 4 && &data[..2] == b"MZ" {
        return "exe";
    }
    // ZIP / OOXML
    if data.len() >= 4 && &data[..4] == b"PK\x03\x04" {
        let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
        return match ext.as_str() {
            "docx" | "xlsx" | "pptx" => "ooxml",
            _ => "zip",
        };
    }

    // Fallback to extension
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "png" => "png",
        "jpg" | "jpeg" => "jpg",
        "gif" => "gif",
        "webp" => "webp",
        "tiff" | "tif" => "tiff",
        "mp3" => "mp3",
        "flac" => "flac",
        "pdf" => "pdf",
        "docx" | "xlsx" | "pptx" => "ooxml",
        "mp4" | "mov" => "mp4",
        "exe" | "dll" => "exe",
        _ => "",
    }
}

/// Check if a file extension is supported
pub fn is_supported(name: &str) -> bool {
    let ext = name.rsplit('.').next().unwrap_or("").to_lowercase();
    SUPPORTED_EXTENSIONS.contains(&ext.as_str())
}

// ─── AI Keyword Detection ────────────────────────────────────────────────────

/// Check if byte slice contains any AI keyword (case-insensitive)
fn has_ai_keywords(data: &[u8]) -> bool {
    // Convert to string lossy for keyword matching
    let text = String::from_utf8_lossy(data);
    let text_lower = text.to_lowercase();
    AI_KEYWORDS
        .iter()
        .any(|kw| text_lower.contains(&kw.to_lowercase()))
}

/// Remove "ChatGPT Image", "DALL-E", "Midjourney ..." prefixes from filename
pub fn clean_filename(name: &str) -> String {
    let mut s = name.to_string();
    // Remove "ChatGPT Image" prefix
    if let Some(rest) = s.strip_prefix("ChatGPT Image") {
        s = rest.trim_start().to_string();
    }
    // Remove "DALL-E " or "DALL-E N " prefix
    if let Some(rest) = s.strip_prefix("DALL-E") {
        let rest = rest.trim_start();
        // Skip optional digit
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            s = rest[1..].trim_start().to_string();
        } else {
            s = rest.to_string();
        }
    }
    // Remove "Midjourney ..." prefix (until first " - ")
    if let Some(rest) = s.strip_prefix("Midjourney") {
        if let Some(idx) = rest.find(" - ") {
            s = rest[idx + 3..].trim_start().to_string();
        } else {
            s = rest.trim_start().to_string();
        }
    }
    s
}

// ─── Process Result ──────────────────────────────────────────────────────────

pub struct ProcessResult {
    pub output: Vec<u8>,
    pub removed: usize,
    pub file_type: String,
}

// ─── PNG Processor ───────────────────────────────────────────────────────────

struct PngChunk {
    chunk_type: [u8; 4],
    data: Vec<u8>,
}

fn png_read_chunks(data: &[u8]) -> Vec<PngChunk> {
    let mut chunks = Vec::new();
    let mut offset = 8; // skip PNG signature
    while offset + 8 <= data.len() {
        let length = u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        let mut ctype = [0u8; 4];
        ctype.copy_from_slice(&data[offset + 4..offset + 8]);
        if offset + 12 + length > data.len() {
            break;
        }
        let chunk_data = data[offset + 8..offset + 8 + length].to_vec();
        chunks.push(PngChunk {
            chunk_type: ctype,
            data: chunk_data,
        });
        offset += 12 + length;
        if &ctype == b"IEND" {
            break;
        }
    }
    chunks
}

fn png_make_chunk(ctype: &[u8; 4], cdata: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(12 + cdata.len());
    out.extend_from_slice(&(cdata.len() as u32).to_be_bytes());
    out.extend_from_slice(ctype);
    out.extend_from_slice(cdata);
    let crc = crc32fast::hash(&[ctype.as_slice(), cdata].concat());
    out.extend_from_slice(&crc.to_be_bytes());
    out
}

fn png_should_remove(ctype: &[u8; 4], cdata: &[u8]) -> bool {
    let ct = String::from_utf8_lossy(ctype);
    if ct == "caBX" {
        return true;
    }
    if ct != "tEXt" && ct != "zTXt" && ct != "iTXt" {
        return false;
    }
    // For tEXt: keyword\0value
    if ct == "tEXt" {
        if let Some(ni) = cdata.iter().position(|&b| b == 0) {
            let content = format!(
                "{} {}",
                String::from_utf8_lossy(&cdata[..ni]),
                String::from_utf8_lossy(&cdata[ni + 1..])
            );
            return has_ai_keywords(content.as_bytes());
        }
        return false;
    }
    // zTXt: keyword\0\0 compressed_value (zlib)
    if ct == "zTXt" {
        if let Some(ni) = cdata.iter().position(|&b| b == 0) {
            if ni + 2 < cdata.len() && cdata[ni + 1] == 0 {
                let kw = String::from_utf8_lossy(&cdata[..ni]);
                if has_ai_keywords(kw.as_bytes()) {
                    return true;
                }
                let raw = &cdata[ni + 2..];
                let mut buf = Vec::new();
                let _ = flate2::read::ZlibDecoder::new(raw).read_to_end(&mut buf);
                if has_ai_keywords(&buf) {
                    return true;
                }
            }
        }
        return false;
    }
    false
}

fn png_custom_chunks(meta: &Metadata) -> Vec<Vec<u8>> {
    let mut chunks = Vec::new();
    let desc = if meta.description.is_empty() {
        format!("Created by {}", meta.author)
    } else {
        meta.description.clone()
    };
    let credit = if meta.credit.is_empty() {
        meta.author.clone()
    } else {
        meta.credit.clone()
    };
    let fields: Vec<(&str, &str)> = vec![
        ("Author", &meta.author),
        ("Description", &desc),
        ("Credit", &credit),
        ("Source", &meta.source),
    ];
    for (kw, val) in fields {
        if val.is_empty() {
            continue;
        }
        let mut cdata = Vec::new();
        cdata.extend_from_slice(kw.as_bytes());
        cdata.push(0);
        cdata.extend_from_slice(val.as_bytes());
        chunks.push(png_make_chunk(b"tEXt", &cdata));
    }
    chunks
}

pub fn process_png(data: &[u8], meta: &Metadata) -> ProcessResult {
    let chunks = png_read_chunks(data);
    let custom_kw: &[&[u8]] = &[b"Author", b"Description", b"Credit", b"Source"];
    let mut kept: Vec<&PngChunk> = Vec::new();
    let mut removed = 0;

    for chunk in &chunks {
        if png_should_remove(&chunk.chunk_type, &chunk.data) {
            removed += 1;
        } else if &chunk.chunk_type == b"tEXt" {
            if let Some(ni) = chunk.data.iter().position(|&b| b == 0) {
                let kw = &chunk.data[..ni];
                if custom_kw.contains(&kw) {
                    continue;
                }
            }
            kept.push(chunk);
        } else {
            kept.push(chunk);
        }
    }

    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut iend: Option<&PngChunk> = None;
    for chunk in &kept {
        if &chunk.chunk_type == b"IEND" {
            iend = Some(chunk);
        } else {
            out.extend_from_slice(&png_make_chunk(&chunk.chunk_type, &chunk.data));
        }
    }
    for ch in png_custom_chunks(meta) {
        out.extend_from_slice(&ch);
    }
    if let Some(iend) = iend {
        out.extend_from_slice(&png_make_chunk(&iend.chunk_type, &iend.data));
    }
    ProcessResult {
        output: out,
        removed,
        file_type: "png".to_string(),
    }
}

// ─── JPG Processor ───────────────────────────────────────────────────────────

pub fn process_jpg(data: &[u8], _meta: &Metadata) -> ProcessResult {
    let mut removed = 0;
    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[..2.min(data.len())]);
    let mut i = 2;

    while i + 1 < data.len() {
        if data[i] != 0xFF {
            out.push(data[i]);
            i += 1;
            continue;
        }
        let marker = data[i + 1];
        // Standalone markers
        if marker == 0x00 || marker == 0x01 || (0xD0..=0xD9).contains(&marker) {
            out.extend_from_slice(&data[i..i + 2]);
            i += 2;
            continue;
        }
        if marker == 0xD9 {
            out.extend_from_slice(&data[i..i + 2]);
            i += 2;
            continue;
        }
        if i + 3 >= data.len() {
            out.extend_from_slice(&data[i..]);
            break;
        }
        let length = u16::from_be_bytes([data[i + 2], data[i + 3]]) as usize;
        let seg_end = i + 2 + length;
        if seg_end > data.len() {
            out.extend_from_slice(&data[i..]);
            break;
        }
        // APP1 (0xE1) and APP2 (0xE2) — EXIF/XMP/ICC
        if marker == 0xE1 || marker == 0xE2 {
            let chunk = &data[i + 4..seg_end];
            if has_ai_keywords(chunk) {
                removed += 1;
                i = seg_end;
                continue;
            }
        }
        out.extend_from_slice(&data[i..seg_end]);
        i = seg_end;
    }
    ProcessResult {
        output: out,
        removed,
        file_type: "jpg".to_string(),
    }
}

// ─── WebP Processor ──────────────────────────────────────────────────────────

pub fn process_webp(data: &[u8], _meta: &Metadata) -> ProcessResult {
    if data.len() < 12 || &data[..4] != b"RIFF" || &data[8..12] != b"WEBP" {
        return ProcessResult {
            output: data.to_vec(),
            removed: 0,
            file_type: "webp".to_string(),
        };
    }
    let mut removed = 0;
    let mut out = Vec::with_capacity(data.len());
    out.extend_from_slice(&data[..12]); // RIFF header
    let mut i = 12;

    while i + 8 <= data.len() {
        let cid = &data[i..i + 4];
        let csize =
            u32::from_le_bytes([data[i + 4], data[i + 5], data[i + 6], data[i + 7]]) as usize;
        let cend = i + 8 + csize + (csize % 2);
        if cend > data.len() {
            out.extend_from_slice(&data[i..]);
            break;
        }
        if (cid == b"EXIF" || cid == b"XMP ") && has_ai_keywords(&data[i + 8..i + 8 + csize]) {
            removed += 1;
        } else {
            out.extend_from_slice(&data[i..cend]);
        }
        i = cend;
    }
    // Update RIFF size
    let riff_size = (out.len() - 8) as u32;
    out[4..8].copy_from_slice(&riff_size.to_le_bytes());
    ProcessResult {
        output: out,
        removed,
        file_type: "webp".to_string(),
    }
}

// ─── MP4/MOV Processor ──────────────────────────────────────────────────────

fn mp4_has_ai_in_range(data: &[u8], start: usize, end: usize) -> bool {
    if end > data.len() {
        return false;
    }
    has_ai_keywords(&data[start..end])
}

pub fn process_mp4(data: &[u8], _meta: &Metadata) -> ProcessResult {
    let mut removed = 0;
    let mut out = Vec::with_capacity(data.len());

    fn walk(data: &[u8], out: &mut Vec<u8>, removed: &mut usize, start: usize, end: usize) {
        let mut pos = start;
        while pos + 8 <= end {
            let sz = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            let tp = if pos + 8 <= end {
                &data[pos + 4..pos + 8]
            } else {
                b""
            };
            if sz < 8 || pos + sz > end {
                out.extend_from_slice(&data[pos..end]);
                return;
            }
            if (tp == b"udta" || tp == b"meta" || tp == b"ilst")
                && mp4_has_ai_in_range(data, pos, pos + sz)
            {
                *removed += 1;
            } else if tp == b"moov"
                || tp == b"trak"
                || tp == b"mdia"
                || tp == b"minf"
                || tp == b"stbl"
            {
                out.extend_from_slice(&data[pos..pos + 8]);
                walk(data, out, removed, pos + 8, pos + sz);
            } else {
                out.extend_from_slice(&data[pos..pos + sz]);
            }
            pos += sz;
        }
    }

    walk(data, &mut out, &mut removed, 0, data.len());
    ProcessResult {
        output: out,
        removed,
        file_type: "mp4".to_string(),
    }
}

// ─── PDF Processor ───────────────────────────────────────────────────────────

pub fn process_pdf(data: &[u8], meta: &Metadata) -> ProcessResult {
    let mut removed = 0;
    let mut out = data.to_vec();

    // Remove XMP metadata blocks that contain AI keywords
    let pattern = b"<x:xmpmeta";
    let close_pattern = b"</x:xmpmeta>";
    let mut i = 0;
    let mut ranges_to_remove: Vec<(usize, usize)> = Vec::new();

    while let Some(start) = out[i..].windows(pattern.len()).position(|w| w == pattern) {
        let abs_start = i + start;
        if let Some(end_rel) = out[abs_start..]
            .windows(close_pattern.len())
            .position(|w| w == close_pattern)
        {
            let abs_end = abs_start + end_rel + close_pattern.len();
            let chunk = &out[abs_start..abs_end];
            if has_ai_keywords(chunk) {
                ranges_to_remove.push((abs_start, abs_end));
                removed += 1;
            }
            i = abs_end;
        } else {
            break;
        }
    }

    // Remove in reverse order to preserve indices
    for &(start, end) in ranges_to_remove.iter().rev() {
        out.drain(start..end);
    }

    // Add author info before %%EOF
    if let Some(eof_pos) = out.windows(5).rposition(|w| w == b"%%EOF") {
        let mut info = String::new();
        if !meta.author.is_empty() {
            info.push_str(&format!("/Author ({})\n", meta.author));
        }
        if !meta.source.is_empty() {
            info.push_str(&format!("/Creator ({})\n", meta.source));
        }
        if !meta.title.is_empty() {
            info.push_str(&format!("/Title ({})\n", meta.title));
        }
        if !meta.description.is_empty() {
            info.push_str(&format!("/Subject ({})\n", meta.description));
        }
        if !meta.keywords.is_empty() {
            info.push_str(&format!("/Keywords ({})\n", meta.keywords));
        }
        out.splice(eof_pos..eof_pos, info.bytes());
    }

    ProcessResult {
        output: out,
        removed,
        file_type: "pdf".to_string(),
    }
}

// ─── OOXML (DOCX/XLSX/PPTX) Processor ──────────────────────────────────────

pub fn process_ooxml(data: &[u8], meta: &Metadata) -> ProcessResult {
    use std::io::{Cursor, Read, Write};
    use zip::read::ZipArchive;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let mut removed = 0;
    let cursor = Cursor::new(data);
    let mut zin = match ZipArchive::new(cursor) {
        Ok(z) => z,
        Err(_) => {
            return ProcessResult {
                output: data.to_vec(),
                removed: 0,
                file_type: "ooxml".to_string(),
            }
        }
    };

    let mut buf = Vec::new();
    {
        let mut zout = ZipWriter::new(Cursor::new(&mut buf));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for i in 0..zin.len() {
            let mut item = match zin.by_index(i) {
                Ok(item) => item,
                Err(_) => continue,
            };
            let fname = item.name().to_string();
            let mut content = Vec::new();
            item.read_to_end(&mut content).ok();

            if fname == "docProps/core.xml" && has_ai_keywords(&content) {
                let creator = if meta.author.is_empty() {
                    "".to_string()
                } else {
                    meta.author.clone()
                };
                let title = if meta.title.is_empty() {
                    "".to_string()
                } else {
                    meta.title.clone()
                };
                let desc = if meta.description.is_empty() {
                    "".to_string()
                } else {
                    meta.description.clone()
                };
                let kw = if meta.keywords.is_empty() {
                    "".to_string()
                } else {
                    meta.keywords.clone()
                };
                let comments = if meta.comments.is_empty() {
                    "".to_string()
                } else {
                    meta.comments.clone()
                };
                content = format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<cp:coreProperties xmlns:cp="http://schemas.openxmlformats.org/package/2006/metadata/core-properties"
  xmlns:dc="http://purl.org/dc/elements/1.1/">
  <dc:creator>{creator}</dc:creator>
  <dc:title>{title}</dc:title>
  <dc:description>{desc}</dc:description>
  <cp:keywords>{kw}</cp:keywords>
  <cp:comments>{comments}</cp:comments>
</cp:coreProperties>"#
                ).into_bytes();
                removed += 1;
            } else if fname == "docProps/app.xml" && has_ai_keywords(&content) {
                let app = if meta.source.is_empty() {
                    "".to_string()
                } else {
                    meta.source.clone()
                };
                let cat = if meta.category.is_empty() {
                    "".to_string()
                } else {
                    meta.category.clone()
                };
                content = format!(
                    r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Properties xmlns="http://schemas.openxmlformats.org/officeDocument/2006/extended-properties">
  <Application>{app}</Application>
  <Category>{cat}</Category>
</Properties>"#
                )
                .into_bytes();
                removed += 1;
            }

            zout.start_file(&fname, options).ok();
            zout.write_all(&content).ok();
        }
    }

    ProcessResult {
        output: buf,
        removed,
        file_type: "ooxml".to_string(),
    }
}

// ─── MP3 Processor ───────────────────────────────────────────────────────────

pub fn process_mp3(data: &[u8], _meta: &Metadata) -> ProcessResult {
    let mut removed = 0;
    let mut out = data.to_vec();

    if out.len() >= 10 && &out[..3] == b"ID3" {
        let tsize = ((out[6] as usize) << 21)
            | ((out[7] as usize) << 14)
            | ((out[8] as usize) << 7)
            | (out[9] as usize);
        let tend = 10 + tsize;
        if tend <= out.len() && has_ai_keywords(&out[10..tend]) {
            out.drain(10..tend);
            removed += 1;
        }
    }
    ProcessResult {
        output: out,
        removed,
        file_type: "mp3".to_string(),
    }
}

// ─── FLAC Processor ──────────────────────────────────────────────────────────

pub fn process_flac(data: &[u8], _meta: &Metadata) -> ProcessResult {
    let mut removed = 0;
    let mut out = Vec::with_capacity(data.len());

    // FLAC must start with "fLaC" magic
    if data.len() < 4 || &data[..4] != b"fLaC" {
        return ProcessResult {
            output: data.to_vec(),
            removed: 0,
            file_type: "flac".to_string(),
        };
    }
    out.extend_from_slice(&data[..4]); // copy magic
    let mut i = 4;

    while i + 4 <= data.len() {
        let bh = data[i];
        let btype = bh & 0x7F;
        let is_last = (bh & 0x80) != 0;
        let bsz =
            ((data[i + 1] as usize) << 16) | ((data[i + 2] as usize) << 8) | (data[i + 3] as usize);
        let block_end = i + 4 + bsz;

        if block_end > data.len() {
            out.extend_from_slice(&data[i..]);
            break;
        }

        // Vorbis comment block (type 4)
        if btype == 4 && has_ai_keywords(&data[i + 4..block_end]) {
            removed += 1;
            i = block_end;
            continue;
        }

        out.extend_from_slice(&data[i..block_end]);
        i = block_end;

        if is_last {
            out.extend_from_slice(&data[i..]);
            break;
        }
    }

    ProcessResult {
        output: out,
        removed,
        file_type: "flac".to_string(),
    }
}

// ─── EXE/DLL Processor ──────────────────────────────────────────────────────

pub fn process_exe(data: &[u8], _meta: &Metadata) -> ProcessResult {
    let mut removed = 0;
    let mut out = data.to_vec();

    for kw in AI_KEYWORDS {
        let kb = kw.as_bytes();
        let mut pos = 0;
        while let Some(idx) = out[pos..].windows(kb.len()).position(|w| w == kb) {
            let abs = pos + idx;
            for b in &mut out[abs..abs + kb.len()] {
                *b = 0;
            }
            removed += 1;
            pos = abs + kb.len();
        }
    }

    ProcessResult {
        output: out,
        removed,
        file_type: "exe".to_string(),
    }
}

// ─── Main Dispatcher ─────────────────────────────────────────────────────────

/// Process a file's bytes and return cleaned bytes + removal count.
pub fn process_file(name: &str, data: &[u8], meta: &Metadata) -> Option<ProcessResult> {
    let ft = detect_file_type(name, data);
    if ft.is_empty() {
        return None;
    }
    let result = match ft {
        "png" => process_png(data, meta),
        "jpg" => process_jpg(data, meta),
        "webp" => process_webp(data, meta),
        "mp4" => process_mp4(data, meta),
        "pdf" => process_pdf(data, meta),
        "ooxml" => process_ooxml(data, meta),
        "mp3" => process_mp3(data, meta),
        "flac" => process_flac(data, meta),
        "exe" => process_exe(data, meta),
        _ => return None,
    };
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_filename() {
        assert_eq!(clean_filename("ChatGPT Image photo.png"), "photo.png");
        assert_eq!(clean_filename("DALL-E 3 art.jpg"), "art.jpg");
    }

    #[test]
    fn test_detect_png() {
        let data = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
        assert_eq!(detect_file_type("test.png", data), "png");
    }

    #[test]
    fn test_has_ai_keywords() {
        assert!(has_ai_keywords(b"This was created by ChatGPT"));
        assert!(!has_ai_keywords(b"Normal file content"));
    }
}
