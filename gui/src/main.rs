#![windows_subsystem = "windows"]

use eframe::egui::{self, Color32, RichText, Rounding, Stroke, TextureHandle, Vec2};
use removemetadata_engine as engine;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const ACCENT: Color32 = Color32::from_rgb(88, 166, 255);
const GREEN: Color32 = Color32::from_rgb(63, 185, 80);
const RED: Color32 = Color32::from_rgb(248, 81, 73);
const YELLOW: Color32 = Color32::from_rgb(210, 153, 34);
const SURFACE: Color32 = Color32::from_rgb(22, 27, 34);
const BORDER: Color32 = Color32::from_rgb(48, 54, 61);
const BG: Color32 = Color32::from_rgb(13, 17, 23);
const MUTED: Color32 = Color32::from_rgb(139, 148, 158);

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([680.0, 560.0])
            .with_min_inner_size([500.0, 400.0])
            .with_title("RemoveMetaData"),
        ..Default::default()
    };
    eframe::run_native(
        "RemoveMetaData",
        options,
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();
            style.spacing.item_spacing = Vec2::new(8.0, 6.0);
            style.visuals.window_fill = BG;
            style.visuals.panel_fill = BG;
            style.visuals.widgets.noninteractive.bg_fill = SURFACE;
            style.visuals.widgets.inactive.bg_fill = SURFACE;
            style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::WHITE);
            style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 48, 58);
            style.visuals.override_text_color = Some(Color32::WHITE);
            style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 36, 44);
            style.visuals.widgets.active.bg_fill = Color32::from_rgb(40, 48, 58);
            style.visuals.widgets.inactive.rounding = Rounding::same(6.0);
            style.visuals.widgets.hovered.rounding = Rounding::same(6.0);
            style.visuals.widgets.active.rounding = Rounding::same(6.0);
            cc.egui_ctx.set_style(style);
            Ok(Box::new(App::new()))
        }),
    )
}

struct FileEntry {
    path: PathBuf,
    status: FileStatus,
    author: String,
    source: String,
    title: String,
    description: String,
    credit: String,
    keywords: String,
    category: String,
    comments: String,
    use_global_author: bool,
    use_global_source: bool,
    use_global_title: bool,
    use_global_description: bool,
    use_global_credit: bool,
    use_global_keywords: bool,
    use_global_category: bool,
    use_global_comments: bool,
}

enum FileStatus {
    Pending,
    Ok { removed: usize },
    Skipped(String),
    Error(String),
}

#[derive(PartialEq)]
enum ViewMode {
    List,
    Grid,
}

struct App {
    files: Vec<FileEntry>,
    selected: Option<usize>,
    output_dir: String,
    author: String,
    source: String,
    title: String,
    description: String,
    credit: String,
    keywords: String,
    category: String,
    comments: String,
    dry_run: bool,
    rename: bool,
    log: Vec<(String, Color32)>,
    total_removed: usize,
    processing: bool,
    drag_hovered: bool,
    view_mode: ViewMode,
    thumbnails: HashMap<PathBuf, TextureHandle>,
}

impl App {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            selected: None,
            output_dir: "out".to_string(),
            author: engine::DEFAULT_AUTHOR.to_string(),
            source: engine::DEFAULT_SOURCE.to_string(),
            title: String::new(),
            description: String::new(),
            credit: String::new(),
            keywords: String::new(),
            category: String::new(),
            comments: String::new(),
            dry_run: false,
            rename: true,
            log: Vec::new(),
            total_removed: 0,
            processing: false,
            drag_hovered: false,
            view_mode: ViewMode::List,
            thumbnails: HashMap::new(),
        }
    }

    fn add_files(&mut self, paths: Vec<PathBuf>) {
        for p in paths {
            if !self.files.iter().any(|f| f.path == p) {
                self.files.push(FileEntry {
                    path: p,
                    status: FileStatus::Pending,
                    author: String::new(),
                    source: String::new(),
                    title: String::new(),
                    description: String::new(),
                    credit: String::new(),
                    keywords: String::new(),
                    category: String::new(),
                    comments: String::new(),
                    use_global_author: true,
                    use_global_source: true,
                    use_global_title: true,
                    use_global_description: true,
                    use_global_credit: true,
                    use_global_keywords: true,
                    use_global_category: true,
                    use_global_comments: true,
                });
            }
        }
    }

    fn ensure_thumbnails(&mut self, ctx: &egui::Context) {
        let thumb_size: u32 = 64;
        for entry in &self.files {
            if self.thumbnails.contains_key(&entry.path) {
                continue;
            }
            let ext = entry
                .path
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_uppercase();
            let is_image = matches!(ext.as_str(), "PNG" | "JPG" | "JPEG" | "WEBP" | "GIF");
            if !is_image {
                continue;
            }
            if let Ok(bytes) = fs::read(&entry.path) {
                if let Ok(img) = image::load_from_memory(&bytes) {
                    let resized = img.resize(
                        thumb_size,
                        thumb_size,
                        image::imageops::FilterType::Lanczos3,
                    );
                    let rgba = resized.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let pixels: Vec<u8> = rgba.into_raw();
                    let color_image =
                        egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);
                    let tex = ctx.load_texture(
                        entry.path.to_string_lossy().to_string(),
                        color_image,
                        egui::TextureOptions::default(),
                    );
                    self.thumbnails.insert(entry.path.clone(), tex);
                }
            }
        }
    }

    fn process(&mut self) {
        self.log.clear();
        self.total_removed = 0;
        self.processing = true;
        let out = PathBuf::from(&self.output_dir);
        let _ = fs::create_dir_all(&out);

        for entry in &mut self.files {
            let name = entry
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            match fs::read(&entry.path) {
                Ok(data) => {
                    let ft = engine::detect_file_type(&name, &data);
                    if ft.is_empty() {
                        entry.status = FileStatus::Skipped("Unsupported".into());
                        self.log
                            .push((format!("  ⏭ {name} — unsupported format"), MUTED));
                        continue;
                    }
                    let meta = engine::Metadata {
                        author: if entry.use_global_author {
                            self.author.clone()
                        } else {
                            entry.author.clone()
                        },
                        source: if entry.use_global_source {
                            self.source.clone()
                        } else {
                            entry.source.clone()
                        },
                        title: if entry.use_global_title {
                            self.title.clone()
                        } else {
                            entry.title.clone()
                        },
                        description: if entry.use_global_description {
                            self.description.clone()
                        } else {
                            entry.description.clone()
                        },
                        credit: if entry.use_global_credit {
                            self.credit.clone()
                        } else {
                            entry.credit.clone()
                        },
                        keywords: if entry.use_global_keywords {
                            self.keywords.clone()
                        } else {
                            entry.keywords.clone()
                        },
                        category: if entry.use_global_category {
                            self.category.clone()
                        } else {
                            entry.category.clone()
                        },
                        comments: if entry.use_global_comments {
                            self.comments.clone()
                        } else {
                            entry.comments.clone()
                        },
                    };
                    match engine::process_file(&name, &data, &meta) {
                        Some(result) => {
                            let out_name = if self.rename {
                                engine::clean_filename(&name)
                            } else {
                                name.clone()
                            };
                            let out_path = out.join(&out_name);
                            if !self.dry_run {
                                if let Err(e) = fs::write(&out_path, &result.output) {
                                    entry.status = FileStatus::Error(e.to_string());
                                    self.log.push((format!("  ✗ {name} — {e}"), RED));
                                    continue;
                                }
                            }
                            self.total_removed += result.removed;
                            entry.status = FileStatus::Ok {
                                removed: result.removed,
                            };
                            if result.removed > 0 {
                                self.log.push((
                                    format!("  ✓ {name} — removed {} AI tag(s)", result.removed),
                                    GREEN,
                                ));
                            } else {
                                self.log
                                    .push((format!("  ✓ {name} — clean, no AI tags",), ACCENT));
                            }
                        }
                        None => {
                            entry.status = FileStatus::Skipped("No processor".into());
                            self.log.push((format!("  ⏭ {name} — no processor"), MUTED));
                        }
                    }
                }
                Err(e) => {
                    entry.status = FileStatus::Error(e.to_string());
                    self.log.push((format!("  ✗ {name} — {e}"), RED));
                }
            }
        }
        let ok_count = self
            .files
            .iter()
            .filter(|f| matches!(f.status, FileStatus::Ok { .. }))
            .count();
        self.log.push((
            format!(
                "Done — {ok_count}/{ } processed, {} AI tag(s) removed",
                self.files.len(),
                self.total_removed
            ),
            GREEN,
        ));
        self.processing = false;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle drag-and-drop
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });
        if !dropped.is_empty() {
            self.add_files(dropped);
        }
        self.drag_hovered = ctx.input(|i| !i.raw.hovered_files.is_empty());

        // Load thumbnails for image files
        self.ensure_thumbnails(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("RemoveMetaData").size(22.0).strong().color(ACCENT));
                ui.label(RichText::new("v1.0").size(12.0).color(MUTED));
            });
            ui.label(RichText::new("Remove AI metadata from images, documents, video & audio").size(12.0).color(MUTED));
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Settings row
            egui::Grid::new("settings").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                ui.label(RichText::new("Author").color(MUTED));
                ui.add(egui::TextEdit::singleline(&mut self.author).desired_width(200.0).text_color(Color32::BLACK));
                ui.end_row();
                ui.label(RichText::new("Source").color(MUTED));
                ui.add(egui::TextEdit::singleline(&mut self.source).desired_width(200.0).text_color(Color32::BLACK));
                ui.end_row();
                ui.label(RichText::new("Output").color(MUTED));
                ui.horizontal(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.output_dir).desired_width(160.0).text_color(Color32::BLACK));
                    if ui.small_button("Browse…").clicked() {
                        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                            self.output_dir = dir.to_string_lossy().to_string();
                        }
                    }
                });
                ui.end_row();
            });

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.checkbox(&mut self.dry_run, RichText::new("Dry run").color(MUTED));
                ui.checkbox(&mut self.rename, RichText::new("Clean filenames").color(MUTED));
            });
            ui.add_space(4.0);

            // Drop zone / file list
            let available = ui.available_size();
            let drop_height = (available.y - 120.0).max(100.0);

            egui::Frame::none()
                .fill(if self.drag_hovered { Color32::from_rgb(20, 30, 48) } else { SURFACE })
                .rounding(Rounding::same(10.0))
                .stroke(Stroke::new(
                    if self.drag_hovered { 2.0 } else { 1.0 },
                    if self.drag_hovered { ACCENT } else { BORDER },
                ))
                .inner_margin(8.0)
                .show(ui, |ui| {
                    if self.files.is_empty() {
                        // Drop zone placeholder
                        ui.set_min_size(Vec2::new(available.x - 20.0, drop_height));
                        ui.vertical_centered(|ui| {
                            ui.add_space(drop_height / 2.0 - 20.0);
                            ui.label(RichText::new("📂").size(32.0));
                            ui.add_space(4.0);
                            ui.label(RichText::new("Drag & drop files here").size(14.0).color(if self.drag_hovered { ACCENT } else { MUTED }));
                            ui.label(RichText::new("or click \"Add Files\" below").size(11.0).color(MUTED));
                        });
                    } else {
                        // View toggle
                        ui.horizontal(|ui| {
                            let list_btn = ui.selectable_label(self.view_mode == ViewMode::List, RichText::new("☰ List").size(11.0));
                            let grid_btn = ui.selectable_label(self.view_mode == ViewMode::Grid, RichText::new("⊞ Grid").size(11.0));
                            if list_btn.clicked() { self.view_mode = ViewMode::List; }
                            if grid_btn.clicked() { self.view_mode = ViewMode::Grid; }
                        });
                        ui.add_space(4.0);
                        ui.set_min_size(Vec2::new(available.x - 20.0, drop_height));

                        let mut to_remove = None;
                        let mut to_select = None;

                        if self.view_mode == ViewMode::List {
                            // List view
                            egui::ScrollArea::vertical().max_height(drop_height).show(ui, |ui| {
                                for (i, entry) in self.files.iter().enumerate() {
                                    let name = entry.path.file_name().unwrap_or_default().to_string_lossy();
                                    let ext = entry.path.extension().unwrap_or_default().to_string_lossy().to_uppercase();
                                    let (status_text, status_color) = match &entry.status {
                                        FileStatus::Pending => ("pending".into(), YELLOW),
                                        FileStatus::Ok { removed } => {
                                            if *removed > 0 { (format!("✓ {removed} removed"), GREEN) } else { ("✓ clean".into(), ACCENT) }
                                        }
                                        FileStatus::Skipped(msg) => (format!("⏭ {msg}"), MUTED),
                                        FileStatus::Error(msg) => (format!("✗ {msg}"), RED),
                                    };
                                    let is_selected = self.selected == Some(i);
                                    let row_bg = if is_selected { Color32::from_rgb(25, 35, 50) } else { Color32::TRANSPARENT };
                                    egui::Frame::none().fill(row_bg).rounding(Rounding::same(4.0)).inner_margin(4.0).show(ui, |ui| {
                                        let row_resp = ui.horizontal(|ui| {
                                            ui.label(RichText::new(&ext).size(10.0).strong().color(ACCENT));
                                            ui.label(RichText::new(&*name).size(12.0));
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                if ui.small_button("✕").on_hover_text("Remove").clicked() { to_remove = Some(i); }
                                                ui.label(RichText::new(&status_text).size(11.0).color(status_color));
                                            });
                                        }).response;
                                        if row_resp.interact(egui::Sense::click()).clicked() { to_select = Some(i); }
                                    });
                                    ui.separator();
                                }
                            });
                        } else {
                            // Grid view
                            egui::ScrollArea::vertical().max_height(drop_height).show(ui, |ui| {
                                let card_w = 130.0;
                                let card_h = 130.0;
                                let spacing = 8.0;
                                let cols = ((available.x - 20.0) / (card_w + spacing)).floor().max(1.0) as usize;
                                let files_snapshot: Vec<_> = self.files.iter().enumerate().collect();
                                for chunk in files_snapshot.chunks(cols) {
                                    ui.horizontal(|ui| {
                                        for &(i, entry) in chunk {
                                            let name = entry.path.file_name().unwrap_or_default().to_string_lossy();
                                            let ext = entry.path.extension().unwrap_or_default().to_string_lossy().to_uppercase();
                                            let (status_text, status_color) = match &entry.status {
                                                FileStatus::Pending => ("pending".into(), YELLOW),
                                                FileStatus::Ok { removed } => {
                                                    if *removed > 0 { (format!("✓ {removed}"), GREEN) } else { ("✓ clean".into(), ACCENT) }
                                                }
                                                FileStatus::Skipped(msg) => (format!("⏭ {msg}"), MUTED),
                                                FileStatus::Error(msg) => (format!("✗ {msg}"), RED),
                                            };
                                            let is_selected = self.selected == Some(i);
                                            let card_bg = if is_selected { Color32::from_rgb(25, 35, 50) } else { BG };

                                            let resp = egui::Frame::none()
                                                .fill(card_bg)
                                                .rounding(Rounding::same(6.0))
                                                .stroke(Stroke::new(1.0, if is_selected { ACCENT } else { BORDER }))
                                                .inner_margin(6.0)
                                                .show(ui, |ui| {
                                                    ui.set_min_size(Vec2::new(card_w, card_h));
                                                    ui.set_max_width(card_w);
                                                    ui.vertical(|ui| {
                                                        // Thumbnail area
                                                        let thumb_size = Vec2::new(card_w - 12.0, 72.0);
                                                        let is_image = matches!(ext.as_str(), "PNG" | "JPG" | "JPEG" | "WEBP" | "GIF");
                                                        if is_image {
                                                            if let Some(tex) = self.thumbnails.get(&entry.path) {
                                                                ui.image((tex.id(), thumb_size));
                                                            } else {
                                                                ui.allocate_ui(thumb_size, |ui| {
                                                                    ui.centered_and_justified(|ui| {
                                                                        ui.label(RichText::new("🖼").size(28.0).color(MUTED));
                                                                    });
                                                                });
                                                            }
                                                        } else {
                                                            // Non-image: show large extension icon
                                                            let icon = match ext.as_str() {
                                                                "PDF" => "📄",
                                                                "MP4" | "MOV" => "🎬",
                                                                "MP3" | "FLAC" => "🎵",
                                                                "DOCX" => "📝",
                                                                "XLSX" => "📊",
                                                                "PPTX" => "📽",
                                                                "EXE" | "DLL" => "⚙",
                                                                _ => "📁",
                                                            };
                                                            ui.allocate_ui(thumb_size, |ui| {
                                                                ui.centered_and_justified(|ui| {
                                                                    ui.label(RichText::new(icon).size(28.0));
                                                                });
                                                            });
                                                        }
                                                        ui.add_space(2.0);
                                                        // File name (truncated)
                                                        ui.label(RichText::new(&ext).size(9.0).strong().color(ACCENT));
                                                        let display_name = if name.len() > 16 { format!("{}…", &name[..15]) } else { name.to_string() };
                                                        ui.label(RichText::new(&display_name).size(10.0).color(Color32::WHITE));
                                                        ui.add_space(2.0);
                                                        ui.horizontal(|ui| {
                                                            ui.label(RichText::new(&status_text).size(8.0).color(status_color));
                                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                                                                if ui.small_button("✕").on_hover_text("Remove").clicked() { to_remove = Some(i); }
                                                            });
                                                        });
                                                    });
                                                }).response;
                                            if resp.interact(egui::Sense::click()).clicked() { to_select = Some(i); }
                                            ui.add_space(spacing);
                                        }
                                    });
                                    ui.add_space(spacing);
                                }
                            });
                        }

                        if let Some(idx) = to_remove {
                            self.files.remove(idx);
                            if self.selected == Some(idx) { self.selected = None; }
                            else if let Some(s) = self.selected { if s > idx { self.selected = Some(s - 1); } }
                        }
                        if let Some(idx) = to_select {
                            self.selected = if self.selected == Some(idx) { None } else { Some(idx) };
                        }
                    }
                });

            ui.add_space(6.0);

            // Per-file property editor — as a popup window
            let mut close_props = false;
            if let Some(idx) = self.selected {
                if idx < self.files.len() {
                    let fname = self.files[idx].path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    let mut open = true;
                    egui::Window::new("Properties")
                        .open(&mut open)
                        .collapsible(false)
                        .resizable(false)
                        .default_width(360.0)
                        .frame(egui::Frame::none()
                            .fill(SURFACE)
                            .rounding(Rounding::same(8.0))
                            .stroke(Stroke::new(1.0, ACCENT))
                            .inner_margin(12.0))
                        .show(ctx, |ui| {
                            ui.label(RichText::new(&fname).size(13.0).strong().color(ACCENT));
                            ui.add_space(6.0);

                            let file = &mut self.files[idx];
                            egui::Grid::new("file_props").num_columns(3).spacing([8.0, 4.0]).show(ui, |ui| {
                                // Author
                                ui.checkbox(&mut file.use_global_author, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Author").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_author, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.author).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file author"))
                                });
                                ui.end_row();
                                // Source
                                ui.checkbox(&mut file.use_global_source, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Source").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_source, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.source).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file source"))
                                });
                                ui.end_row();
                                // Title
                                ui.checkbox(&mut file.use_global_title, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Title").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_title, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.title).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file title"))
                                });
                                ui.end_row();
                                // Description
                                ui.checkbox(&mut file.use_global_description, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Description").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_description, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.description).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file description"))
                                });
                                ui.end_row();
                                // Credit
                                ui.checkbox(&mut file.use_global_credit, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Credit").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_credit, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.credit).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file credit"))
                                });
                                ui.end_row();
                                // Keywords
                                ui.checkbox(&mut file.use_global_keywords, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Keywords").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_keywords, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.keywords).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file keywords"))
                                });
                                ui.end_row();
                                // Category
                                ui.checkbox(&mut file.use_global_category, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Category").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_category, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.category).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file category"))
                                });
                                ui.end_row();
                                // Comments
                                ui.checkbox(&mut file.use_global_comments, RichText::new("Global").color(MUTED));
                                ui.label(RichText::new("Comments").color(MUTED));
                                ui.add_enabled_ui(!file.use_global_comments, |ui| {
                                    ui.add(egui::TextEdit::singleline(&mut file.comments).desired_width(180.0).text_color(Color32::BLACK).hint_text("per-file comments"))
                                });
                                ui.end_row();
                            });
                        });
                    if !open {
                        close_props = true;
                    }
                }
            }

            if close_props {
                self.selected = None;
            }

            // Action buttons
            ui.horizontal(|ui| {
                let can_process = !self.files.is_empty() && !self.processing;

                let add_btn = ui.add_enabled(true, egui::Button::new(
                    RichText::new("＋ Add Files").color(Color32::WHITE)
                ).fill(Color32::from_rgb(48, 54, 61)).rounding(Rounding::same(6.0)));
                if add_btn.clicked() {
                    if let Some(paths) = rfd::FileDialog::new()
                        .add_filter("All Supported", &["png","jpg","jpeg","webp","gif","mp3","flac","mp4","mov","pdf","docx","xlsx","pptx","exe","dll"])
                        .pick_files()
                    {
                        self.add_files(paths);
                    }
                }

                let clear_btn = ui.add_enabled(!self.files.is_empty(), egui::Button::new(
                    RichText::new("Clear").color(Color32::WHITE)
                ).fill(Color32::from_rgb(48, 54, 61)).rounding(Rounding::same(6.0)));
                if clear_btn.clicked() {
                    self.files.clear();
                    self.log.clear();
                    self.total_removed = 0;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let process_btn = ui.add_enabled(can_process, egui::Button::new(
                        RichText::new("Process ▶").strong().color(Color32::WHITE)
                    ).fill(if can_process { ACCENT } else { Color32::from_rgb(40, 45, 52) }).rounding(Rounding::same(6.0)).min_size(Vec2::new(100.0, 30.0)));
                    if process_btn.clicked() {
                        self.process();
                    }

                    ui.label(RichText::new(format!("{} file(s)", self.files.len())).color(MUTED).size(12.0));
                });
            });

            // Log area
            if !self.log.is_empty() {
                ui.add_space(4.0);
                ui.separator();
                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                    for (line, color) in &self.log {
                        ui.label(RichText::new(line).monospace().size(11.0).color(*color));
                    }
                });
            }
        });
    }
}
