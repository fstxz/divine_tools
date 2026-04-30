//! .fnt

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, OnceLock},
};

use eframe::egui::{
    Color32, ColorImage, DragValue, FontId, Grid, ImageData, Pos2, Rect, ScrollArea, Sense, Stroke,
    StrokeKind, TextFormat, TextureHandle, TextureOptions, ViewportBuilder, ViewportId,
    text::LayoutJob, vec2,
};

use crate::{
    editor::{
        Config, Inspector, MessageSeverity, property, property_read_only, show_message, struct_ui,
    },
    types::{
        Binary, FixedArray,
        image_list::{Image, decode_transparent_image},
    },
};

const HEADER: &str = "ML_FONT_01";
const GLYPH_HEADER: u64 = 18441921394929718322;

const GLYPH_TEXTURE_SIZE: usize = 64;

static FONT_EDITOR: OnceLock<Arc<Mutex<Option<FontEditor>>>> = OnceLock::new();

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Font {
    unknown: u32,
    height: u32,
    unknown2: FixedArray<u8, 1024>,
    glyphs: Vec<GlyphData>,
    // Maps char to index in the glyphs array.
    glyph_map: HashMap<char, usize>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct GlyphData {
    /// Horizontal offset.
    h_offset: i32,
    /// Vertical offset.
    v_offset: i32,
    /// Horizontal advance. Values higher than image's width have no effect.
    h_advance: u32,
    /// Unicode character that this glyph represents.
    character: char,
    /// Bitmap image of the glyph.
    image: Image,
    unknown: u8,
}

impl Binary for Font {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let header_bytes = reader.read_bytes(10)?;
        let header = str::from_utf8(header_bytes)?;

        if header != HEADER {
            return Err("Unsupported font format".into());
        }

        let unknown = u32::from_bytes(reader)?;

        let height = u32::from_bytes(reader)?;

        let glyph_count = u32::from_bytes(reader)?;
        let unknown2 = FixedArray::from_bytes(reader)?;
        let mut glyphs = Vec::new();
        let mut glyph_map = HashMap::new();

        for _ in 0..glyph_count {
            let glyph_data = GlyphData::from_bytes(reader)?;
            glyph_map.insert(glyph_data.character, glyphs.len());
            glyphs.push(glyph_data);
        }

        Ok(Self {
            unknown,
            height,
            unknown2,
            glyphs,
            glyph_map,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        writer.write_bytes(HEADER.as_bytes());
        self.unknown.to_bytes(writer);
        self.height.to_bytes(writer);
        (self.glyphs.len() as u32).to_bytes(writer);
        self.unknown2.to_bytes(writer);

        for glyph in &self.glyphs {
            glyph.to_bytes(writer);
        }
    }
}

impl Binary for GlyphData {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let header = u64::from_bytes(reader)?;

        if header != GLYPH_HEADER {
            return Err("Wrong glyph header".into());
        }

        let h_offset = i32::from_bytes(reader)?;
        let v_offset = i32::from_bytes(reader)?;
        let h_advance = u32::from_bytes(reader)?;

        let character =
            char::from_u32(u16::from_bytes(reader)? as u32).ok_or("Invalid Unicode value")?;

        let buffer_length = reader.peek_u32()?;

        let image_bytes = reader.read_bytes(buffer_length as usize)?;
        let image = decode_transparent_image(image_bytes)?;

        let unknown = u8::from_bytes(reader)?;

        Ok(GlyphData {
            h_offset,
            v_offset,
            h_advance,
            character,
            image,
            unknown,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        GLYPH_HEADER.to_bytes(writer);
        self.h_offset.to_bytes(writer);
        self.v_offset.to_bytes(writer);
        self.h_advance.to_bytes(writer);
        (self.character as u16).to_bytes(writer);

        writer.write_bytes(&self.image.encode_as_transparent());

        self.unknown.to_bytes(writer);
    }
}

impl Inspector for GlyphData {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}

impl Inspector for Font {
    fn init(&mut self, _ctx: &crate::editor::Context) -> crate::Result<()> {
        match FONT_EDITOR.get() {
            // Drop and replace the old editor if it exists.
            Some(e) => {
                let _ = e.lock().unwrap().replace(FontEditor::default());
            }
            // Otherwise initialize a new one.
            None => {
                let _ = FONT_EDITOR.set(Arc::new(Mutex::new(Some(FontEditor::default()))));
            }
        };

        Ok(())
    }

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        let mut font_editor = FONT_EDITOR.get().unwrap().lock().unwrap();
        let font_editor = font_editor.as_mut().unwrap();

        top_panel(self, font_editor, ui);

        ui.separator();

        ui.horizontal_top(|ui| {
            glyphs_grid(self, font_editor, ui);

            ui.separator();

            side_panel(self, font_editor, ui);
        });

        if font_editor.show_import_glyphs_dialog {
            show_import_glyphs_dialog(font_editor, self, ui);
        }

        if font_editor.show_glyph_rename_dialog {
            show_glyph_rename_dialog(font_editor, self, ui);
        }
    }
}

fn top_panel(font: &mut Font, font_editor: &mut FontEditor, ui: &mut eframe::egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Font height");
        ui.add(DragValue::new(&mut font.height));

        ui.label(format!("Glyphs: {}", font.glyphs.len()));

        ui.separator();

        if ui.button("Export all").clicked() {
            let Some(path) = rfd::FileDialog::new().pick_folder() else {
                return;
            };

            let mut exported = 0;

            for glyph in &font.glyphs {
                let path = path.join(glyph_filename(glyph.character));

                if let Err(e) = export_glyph(glyph, &path) {
                    show_message(
                        &format!("Failed to export glyph \"{}\": {e}", glyph.character),
                        MessageSeverity::Error,
                    );
                } else {
                    exported += 1;
                }
            }

            show_message(
                &format!(
                    "Successfully exported {} glyphs ({} failed)",
                    exported,
                    font.glyphs.len() - exported
                ),
                MessageSeverity::Info,
            );
        }

        if ui.button("Import").clicked() {
            font_editor.show_import_glyphs_dialog = true;
        }
    });
}

fn side_panel(font: &mut Font, font_editor: &mut FontEditor, ui: &mut eframe::egui::Ui) {
    let Some(selected) = font_editor.selected_glyph else {
        return;
    };

    ui.vertical(|ui| {
        let glyph_index = font.glyph_map.get(&selected).unwrap();
        let glyph = font.glyphs.get_mut(*glyph_index).unwrap();

        ui.horizontal(|ui| {
            ui.heading(format!("{}", glyph.character));
            ui.separator();

            if ui.button("Change").clicked() {
                font_editor.rename_dialog_glyph_to_rename = selected;
                font_editor.show_glyph_rename_dialog = true;
            }
        });

        ui.separator();

        struct_ui(ui, |ui| {
            property("h_offset", &mut glyph.h_offset, ui);
            property("v_offset", &mut glyph.v_offset, ui);
            property("h_advance", &mut glyph.h_advance, ui);
            property_read_only("width", &mut glyph.image.width(), ui);
            property_read_only("height", &mut glyph.image.height(), ui);
            property("unknown", &mut glyph.unknown, ui);
        });

        ui.separator();

        ui.horizontal(|ui| {
            let glyph_index = font.glyph_map.get(&selected).unwrap();
            let glyph = font.glyphs.get_mut(*glyph_index).unwrap();

            if ui.button("Export").clicked() {
                let Some(path) = rfd::FileDialog::new()
                    .set_file_name(glyph_filename(selected))
                    .save_file()
                else {
                    return;
                };

                match export_glyph(glyph, &path) {
                    Ok(()) => show_message(
                        &format!("Successfully exported glyph \"{}\".", glyph.character),
                        MessageSeverity::Info,
                    ),
                    Err(e) => show_message(
                        &format!("Failed to export glyph \"{}\": {e}", glyph.character),
                        MessageSeverity::Error,
                    ),
                }
            }

            if ui.button("Import").clicked() {
                let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG Image (.png)", &["png"])
                    .pick_file()
                else {
                    return;
                };

                match import_glyph(&path, selected) {
                    Ok(glyph) => {
                        show_message(
                            &format!("Successfully imported glyph \"{}\".", glyph.character),
                            MessageSeverity::Info,
                        );

                        if let Some(glyph_index) = font.glyph_map.get(&selected) {
                            let old_glyph = font.glyphs.get_mut(*glyph_index).unwrap();
                            *old_glyph = glyph;

                            font_editor.textures.remove(&selected);
                        } else {
                            font.glyph_map.insert(selected, font.glyphs.len());
                            font.glyphs.push(glyph);
                        }
                    }
                    Err(e) => show_message(
                        &format!("Failed to import glyph \"{}\": {e}", glyph.character),
                        MessageSeverity::Error,
                    ),
                }
            }
        });
    });
}

fn import_glyph(path: &Path, character: char) -> crate::Result<GlyphData> {
    let dynamic_image = image::load_from_memory(&std::fs::read(path)?)?;

    let image = dynamic_image.to_rgba8();
    let (buffer, _) = image.as_chunks();

    Ok(GlyphData {
        h_offset: 0,
        v_offset: 0,
        // Game's own fonts have h_advance set to width + 1, so we do the same,
        // even though I didn't find any difference compared to setting this to just width.
        h_advance: image.width() + 1,
        character,
        image: Image::new(image.width(), image.height(), buffer.to_vec()),
        // TODO
        unknown: 0,
    })
}

fn import_glyphs(font: &mut Font, font_editor: &mut FontEditor, overwrite: bool) {
    let Some(paths) = rfd::FileDialog::new()
        .add_filter("PNG Image (.png)", &["png"])
        .pick_files()
    else {
        return;
    };

    let mut imported = 0;

    for path in &paths {
        let character = match parse_char_from_path(path) {
            Ok(ch) => ch,
            Err(e) => {
                show_message(
                    &format!("Failed to import \"{}\": {e}", path.display()),
                    MessageSeverity::Error,
                );
                return;
            }
        };

        match import_glyph(path, character) {
            Ok(glyph) => {
                if let Some(glyph_index) = font.glyph_map.get(&character) {
                    if overwrite {
                        let old_glyph = font.glyphs.get_mut(*glyph_index).unwrap();
                        old_glyph.image = glyph.image;

                        font_editor.textures.remove(&character);

                        imported += 1;
                    }
                } else {
                    font.glyph_map.insert(character, font.glyphs.len());
                    font.glyphs.push(glyph);
                    imported += 1;
                }
            }
            Err(e) => {
                show_message(
                    &format!("Failed to import \"{}\": {e}", path.display()),
                    MessageSeverity::Error,
                );
                return;
            }
        }
    }

    show_message(
        &format!(
            "Successfully imported {} glyphs ({} skipped)",
            imported,
            paths.len() - imported
        ),
        MessageSeverity::Info,
    );
}

fn export_glyph(glyph: &GlyphData, path: &Path) -> crate::Result<()> {
    let buffer = glyph.image.image_data().as_flattened();

    image::save_buffer(
        path,
        buffer,
        glyph.image.width(),
        glyph.image.height(),
        image::ColorType::Rgba8,
    )
    .map_err(|e| e.into())
}

fn glyphs_grid(font: &mut Font, font_editor: &mut FontEditor, ui: &mut eframe::egui::Ui) {
    let uv = Rect {
        min: Pos2::ZERO,
        max: Pos2 { x: 1.0, y: 1.0 },
    };

    let panel_size = if font_editor.selected_glyph.is_some() {
        vec2(200.0, 0.0)
    } else {
        vec2(0.0, 0.0)
    };

    let table_size = (ui.available_size() - panel_size).max(vec2(0.0, 0.0));

    ui.allocate_ui(table_size, |ui| {
        ScrollArea::both()
            .id_salt("world")
            .auto_shrink(false)
            .show_viewport(ui, |ui, viewport| {
                let num_columns =
                    (viewport.size().x as usize / GLYPH_TEXTURE_SIZE).saturating_sub(1);

                Grid::new("glyphs")
                    .striped(true)
                    .num_columns(num_columns)
                    .show(ui, |ui| {
                        let mut col = 0;
                        for glyph in &font.glyphs {
                            ui.vertical(|ui| {
                                let (response, painter) = ui.allocate_painter(
                                    vec2(GLYPH_TEXTURE_SIZE as f32, GLYPH_TEXTURE_SIZE as f32),
                                    Sense::click(),
                                );

                                // Get the glyph texture from the cache or load it.
                                let texture = font_editor
                                    .textures
                                    .entry(glyph.character)
                                    .or_insert_with(|| {
                                        let image_data = ImageData::Color(Arc::new(
                                            ColorImage::from_rgba_unmultiplied(
                                                [
                                                    glyph.image.width() as usize,
                                                    glyph.image.height() as usize,
                                                ],
                                                &glyph
                                                    .image
                                                    .image_data()
                                                    .iter()
                                                    .flatten()
                                                    .cloned()
                                                    .collect::<Vec<_>>(),
                                            ),
                                        ));

                                        ui.ctx().load_texture(
                                            glyph.character,
                                            image_data,
                                            TextureOptions::NEAREST,
                                        )
                                    });

                                painter.image(texture.id(), response.rect, uv, Color32::WHITE);

                                let selected = font_editor
                                    .selected_glyph
                                    .is_some_and(|v| v == glyph.character);

                                if response.clicked() {
                                    font_editor.selected_glyph = Some(glyph.character);
                                } else if response.hovered() || selected {
                                    painter.rect_stroke(
                                        response.rect,
                                        0,
                                        Stroke::new(2.0, Color32::WHITE),
                                        StrokeKind::Inside,
                                    );
                                }
                            });

                            col += 1;

                            if col == num_columns {
                                col = 0;
                                ui.end_row();
                            }
                        }
                    });
            });
    });
}

#[derive(Default)]
struct FontEditor {
    textures: HashMap<char, TextureHandle>,
    selected_glyph: Option<char>,
    show_import_glyphs_dialog: bool,
    show_glyph_rename_dialog: bool,
    rename_dialog_glyph_to_rename: char,
    rename_dialog_char_string: String,
}

fn show_import_glyphs_dialog(
    font_editor: &mut FontEditor,
    font: &mut Font,
    ui: &mut eframe::egui::Ui,
) {
    let viewport_id = ViewportId::from_hash_of("import_glyphs_dialog");
    ui.ctx().show_viewport_immediate(
        viewport_id,
        ViewportBuilder::default()
            .with_title("Import glyphs")
            .with_inner_size(vec2(200.0, 100.0)),
        |ctx, _| {
            eframe::egui::TopBottomPanel::bottom("import_glyphs_bottom_panel").show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    if ui.button("Select files and import").clicked() {
                        import_glyphs(
                            font,
                            font_editor,
                            Config::get().font_import_glyphs_overwrite,
                        );
                    }
                });
            });

            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                let overwrite = &mut Config::get().font_import_glyphs_overwrite;

                ui.checkbox(overwrite, "Overwrite image data of existing glyphs");
            });

            ctx.input(|i| {
                if i.viewport().close_requested() {
                    font_editor.show_import_glyphs_dialog = false;
                }
            });
        },
    );
}

fn show_glyph_rename_dialog(
    font_editor: &mut FontEditor,
    font: &mut Font,
    ui: &mut eframe::egui::Ui,
) {
    let viewport_id = ViewportId::from_hash_of("rename_glyph_dialog");
    ui.ctx().show_viewport_immediate(
        viewport_id,
        ViewportBuilder::default()
            .with_title("Rename")
            .with_inner_size(vec2(200.0, 100.0)),
        |ctx, _| {
            let mut problems = Vec::new();

            if let Some(new_char) = font_editor.rename_dialog_char_string.chars().next() {
                if font.glyph_map.contains_key(&new_char) {
                    problems.push("This character already exists.");
                }
            } else {
                problems.push("Character can not be empty.");
            }

            eframe::egui::TopBottomPanel::bottom("rename_glyph_dialog_bottom_panel").show(
                ctx,
                |ui| {
                    ui.vertical_centered(|ui| {
                        if ui
                            .add_enabled(problems.is_empty(), eframe::egui::Button::new("Rename"))
                            .clicked()
                        {
                            let new_char = font_editor
                                .rename_dialog_char_string
                                .chars()
                                .next()
                                .unwrap();

                            let texture = font_editor
                                .textures
                                .remove(&font_editor.rename_dialog_glyph_to_rename)
                                .unwrap();
                            font_editor.textures.insert(new_char, texture);

                            let glyph_index = *font
                                .glyph_map
                                .get(&font_editor.rename_dialog_glyph_to_rename)
                                .unwrap();

                            font.glyph_map
                                .remove(&font_editor.rename_dialog_glyph_to_rename);
                            font.glyph_map.insert(new_char, glyph_index);

                            let glyph = font.glyphs.get_mut(glyph_index).unwrap();
                            glyph.character = new_char;

                            font_editor.selected_glyph = Some(new_char);
                        }
                    });
                },
            );

            eframe::egui::CentralPanel::default().show(ctx, |ui| {
                let color = if ui.visuals().dark_mode {
                    Color32::WHITE
                } else {
                    Color32::BLACK
                };

                for problem in problems {
                    let mut job = LayoutJob::default();
                    job.append(
                        problem,
                        0.0,
                        TextFormat {
                            font_id: FontId::default(),
                            color,
                            background: Color32::RED,
                            ..Default::default()
                        },
                    );
                    ui.label(job);
                    ui.separator();
                }

                ui.horizontal(|ui| {
                    ui.label("Character");
                    ui.add(
                        eframe::egui::TextEdit::singleline(
                            &mut font_editor.rename_dialog_char_string,
                        )
                        .id_source("rename_glyph_dialog_text_edit"),
                    );
                });
            });

            ctx.input(|i| {
                if i.viewport().close_requested() {
                    font_editor.show_glyph_rename_dialog = false;
                }
            });
        },
    );
}

fn glyph_filename(ch: char) -> String {
    format!("U+{:04X}.png", ch as u32)
}

fn parse_char_from_path(path: &Path) -> crate::Result<char> {
    let file_stem = path
        .file_stem()
        .ok_or_else(|| String::from("Path must have file stem"))?
        .to_string_lossy();

    let Some(code_point_str) = file_stem.strip_prefix("U+") else {
        return Err("Invalid file name format\n\n \
                        File name should be in U+0000.png format, where 0000 is a Unicode code point in hexadecimal".into());
    };

    let code_point = u32::from_str_radix(code_point_str, 16)?;

    char::from_u32(code_point).ok_or_else(|| "Could not convert code point to char".into())
}
