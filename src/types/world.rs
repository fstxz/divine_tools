//! world.xN

use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use eframe::egui::{
    self, Color32, ColorImage, CursorIcon, DragValue, Grid, ImageData, Key, Pos2, Rect, ScrollArea,
    Sense, Stroke, StrokeKind, TextureHandle, TextureOptions, pos2, vec2,
};

use crate::{
    buffer::BufferReader,
    editor::{Config, Context, Inspector},
    types::{
        Binary,
        image_list::{
            ImageListIndex, ImageType, decode_opaque_image, decode_transparent_image,
            load_image_list,
        },
    },
};

const WIDTH: usize = 512;
const HEIGHT: usize = 1024;
const TILES_WIDTH: usize = 3;

static WORLD_EDITOR: OnceLock<Arc<Mutex<WorldEditor>>> = OnceLock::new();

#[derive(serde::Serialize, serde::Deserialize)]
pub struct World {
    cells: Box<[Cell]>,
    unknown1: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct Cell {
    image_index: i16,
    overlay_image_index: i16,
    unknown0: u16,
    unknown1: u8,
    unknown2: u32,
    unknown3: u32,
    objects: Vec<Object>,
}

// Objects are encoded as 64 bit values:
//
// (displayed in big-endian for easier understanding)
// ---------- ----------------- ------------ ------------------------ --------------
// | Unknown | Image index     | Height     | Object id              | XY offset    |
// | 8 bits  | 14 bits         | 10 bits    | 20 bits                | 6 bits each  |
//  0000 0001 0000 0000 0101 00 00 0111 0000 0000 0001 0110 1000 1000 0100 0100 0001
//
#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct Object {
    x_offset: u8,
    y_offset: u8,
    object_id: u32,
    height: u16,
    image_index: u16,
    unknown0: u8,
}

impl Binary for World {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        // Row offsets, will be calculated during serialization.
        reader.skip(HEIGHT * 4);

        let mut cells = vec![Cell::default(); WIDTH * HEIGHT];

        for y in 0..HEIGHT {
            // Cell offsets, will be calculated during serialization.
            reader.skip(WIDTH * 2);
            for x in 0..WIDTH {
                cells[y * WIDTH + x] = Cell::from_bytes(reader)?;
            }
        }

        Ok(Self {
            cells: cells.into_boxed_slice(),
            unknown1: reader.read_u32()?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        let mut row_offsets = Vec::<u32>::with_capacity(HEIGHT);
        let mut row_writer = crate::buffer::BufferWriter::new();

        for y in 0..HEIGHT {
            row_offsets.push((HEIGHT * 4 + row_writer.len()) as u32);

            let mut cell_writer = crate::buffer::BufferWriter::new();

            for x in 0..WIDTH {
                (cell_writer.len() as u16).to_bytes(&mut row_writer);

                let cell = &self.cells[y * WIDTH + x];
                cell.to_bytes(&mut cell_writer);
            }

            row_writer.write_bytes(&cell_writer.finish());
        }

        for offset in row_offsets {
            offset.to_bytes(writer);
        }

        writer.write_bytes(&row_writer.finish());
        self.unknown1.to_bytes(writer);
    }
}

impl Binary for Cell {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let image_index = reader.read_i16()?;
        let overlay_image_index = reader.read_i16()?;

        let unknown0 = reader.read_u16()?;

        let objects_len = reader.read_u8()?;
        let unknown1 = reader.read_u8()?;

        let unknown2 = reader.read_u32()?;
        let unknown3 = reader.read_u32()?;

        let mut objects = Vec::new();

        for _ in 0..objects_len {
            objects.push(Object::from_bytes(reader)?);
        }

        Ok(Self {
            image_index,
            overlay_image_index,
            unknown0,
            unknown1,
            unknown2,
            unknown3,
            objects,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.image_index.to_bytes(writer);
        self.overlay_image_index.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        (self.objects.len() as u8).to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.unknown2.to_bytes(writer);
        self.unknown3.to_bytes(writer);

        for object in &self.objects {
            object.to_bytes(writer);
        }
    }
}

impl Binary for Object {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let value = reader.read_u64()?;

        let x_offset = (value & 0x3f) as u8;
        let y_offset = ((value >> 6) & 0x3f) as u8;

        let object_id = ((value >> 12) & 0xfffff) as u32;
        let height = ((value >> 32) & 0x3ff) as u16;

        let image_index = ((value >> 42) & 0x3fff) as u16;
        let unknown0 = ((value >> 56) & 0xff) as u8;

        Ok(Self {
            x_offset,
            y_offset,
            object_id,
            height,
            image_index,
            unknown0,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        let mut value = 0u64;

        value += (self.x_offset & 0x3f) as u64;
        value += ((self.y_offset & 0x3f) as u64) << 6;
        value += ((self.object_id & 0xfffff) as u64) << 12;
        value += ((self.height & 0x3ff) as u64) << 32;
        value += ((self.image_index & 0x3fff) as u64) << 42;
        value += (self.unknown0 as u64) << 56;

        writer.write_u64(value);
    }
}

impl Inspector for World {
    fn init(&mut self, ctx: &Context) -> crate::Result<()> {
        let config = Config::get();

        if config.dd_path.as_os_str().is_empty() {
            return Err(
                "Set the path to Divine Divinity directory in preferences and try again".into(),
            );
        }

        if !std::fs::exists(config.dd_path.join("div.exe"))
            .map_err(|e| format!("Failed to read Divinity Divinity directory: {e}"))?
        {
            return Err("Invalid path to Divine Divinity".into());
        }

        let terrain_image_list = load_image_list(
            &config
                .dd_path
                .join(PathBuf::from_iter(["static", "imagelists", "CPackedb.2c"])),
            &config
                .dd_path
                .join(PathBuf::from_iter(["static", "imagelists", "CPackedi.2c"])),
            true,
        )
        .map_err(|e| format!("Failed to load tiles image list: {e}"))?;

        let mut terrain_textures = Vec::new();

        for image in terrain_image_list.images().iter() {
            let image_data = ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(
                [image.width() as usize, image.height() as usize],
                &image
                    .image_data()
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>(),
            )));

            let handle = ctx
                .ui
                .ctx()
                .load_texture("texture", image_data, TextureOptions::NEAREST);

            terrain_textures.push(handle);
        }

        let objects_index_bytes = std::fs::read(config.dd_path.join(PathBuf::from_iter([
            "static",
            "imagelists",
            "Packedi.0",
        ])))
        .map_err(|e| format!("Failed to load objects index: {e}"))?;

        let mut reader = BufferReader::new(&objects_index_bytes);
        let objects_index = ImageListIndex::from_bytes(&mut reader)
            .map_err(|e| format!("Failed to parse objects index: {e}"))?;

        let objects_file = File::open(config.dd_path.join(PathBuf::from_iter([
            "static",
            "imagelists",
            "Packedb.0",
        ])))
        .map_err(|e| format!("Failed to open objects file: {e}"))?;

        let world_editor = Arc::new(Mutex::new(WorldEditor {
            terrain_textures,
            objects_index,
            object_textures: HashMap::new(),
            selected_cell: None,
            objects_file,
            selected_tile: Vec2i::default(),
            selected_overlay_tile: Vec2i::default(),
            dragged_object: None,
            command_history: CommandHistory::default(),
        }));

        let _ = WORLD_EDITOR.set(world_editor);

        Ok(())
    }

    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        let mut world_editor = WORLD_EDITOR.get().unwrap().lock();

        let WorldEditor {
            terrain_textures,
            object_textures,
            objects_index,
            selected_cell,
            objects_file,
            selected_tile,
            selected_overlay_tile,
            dragged_object,
            command_history,
        } = world_editor.as_deref_mut().unwrap();

        ui.input(|i| {
            if i.modifiers.ctrl && i.key_down(Key::Z) {
                if i.modifiers.shift {
                    command_history.redo_action(self);
                } else {
                    command_history.undo_action(self);
                }
            }
        });

        let cell_size = vec2(64.0, 64.0);

        let uv = Rect {
            min: Pos2::ZERO,
            max: Pos2 { x: 1.0, y: 1.0 },
        };

        ui.horizontal_top(|ui| {
            ui.allocate_ui(ui.available_size() - vec2(250.0, 0.0), |ui| {
                ScrollArea::both()
                    .id_salt("world")
                    .show_viewport(ui, |ui, viewport| {
                        let total_size =
                            vec2(WIDTH as f32 * cell_size.x, HEIGHT as f32 * cell_size.y);

                        let (response, painter) = ui.allocate_painter(total_size, Sense::empty());

                        let first_col =
                            ((viewport.min.x / cell_size.x) - 4.0).floor().max(0.0) as usize;
                        let last_col = ((viewport.max.x / cell_size.x) + 4.0)
                            .ceil()
                            .min(WIDTH as f32) as usize;
                        let first_row =
                            ((viewport.min.y / cell_size.y) - 4.0).floor().max(0.0) as usize;
                        let last_row = ((viewport.max.y / cell_size.y) + 4.0)
                            .ceil()
                            .min(HEIGHT as f32) as usize;

                        // Terrain drawing.
                        for row in first_row..last_row {
                            for col in first_col..last_col {
                                let origin = response.rect.min
                                    + vec2(col as f32 * cell_size.x, row as f32 * cell_size.y);
                                let rect = Rect::from_min_size(origin, cell_size);

                                let cell_response = ui.allocate_rect(rect, Sense::click());
                                let ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
                                let x_pressed = ui.input(|i| i.key_down(Key::X));

                                let cell_idx = row * WIDTH + col;

                                if cell_response.clicked() {
                                    if ctrl_pressed {
                                        command_history.do_action(
                                            self,
                                            Command::SetImageIndex {
                                                pos: Vec2i::new(col as u32, row as u32),
                                                old_image: self.cells[cell_idx].image_index,
                                                new_image: (selected_tile.y * TILES_WIDTH as u32
                                                    + selected_tile.x)
                                                    as i16,
                                            },
                                        );
                                    }
                                } else if cell_response.secondary_clicked() {
                                    if ctrl_pressed {
                                        command_history.do_action(
                                            self,
                                            Command::SetOverlayImageIndex {
                                                pos: Vec2i::new(col as u32, row as u32),
                                                old_image: self.cells[cell_idx].overlay_image_index,
                                                new_image: (selected_overlay_tile.y
                                                    * TILES_WIDTH as u32
                                                    + selected_overlay_tile.x)
                                                    as i16,
                                            },
                                        );
                                    } else if x_pressed {
                                        command_history.do_action(
                                            self,
                                            Command::SetOverlayImageIndex {
                                                pos: Vec2i::new(col as u32, row as u32),
                                                old_image: self.cells[cell_idx].overlay_image_index,
                                                new_image: -1,
                                            },
                                        );
                                    }
                                }

                                let cell = &self.cells[cell_idx];
                                if cell.image_index >= 0 {
                                    painter.image(
                                        terrain_textures[cell.image_index as usize].id(),
                                        rect,
                                        uv,
                                        if cell_response.hovered() {
                                            Color32::from_gray(150)
                                        } else {
                                            Color32::WHITE
                                        },
                                    );
                                }

                                if cell.overlay_image_index >= 0 {
                                    painter.image(
                                        terrain_textures[cell.overlay_image_index as usize].id(),
                                        rect,
                                        uv,
                                        Color32::WHITE,
                                    );
                                }

                                if cell_response.clicked() {
                                    *selected_cell = Some(SelectedCell {
                                        cell: Vec2i::new(col as u32, row as u32),
                                        selected_object: None,
                                    });
                                }
                            }
                        }

                        let alt_pressed = ui.ctx().input(|i| i.modifiers.alt);

                        // Object drawing.
                        for row in first_row..last_row {
                            for col in first_col..last_col {
                                let origin = response.rect.min
                                    + vec2(col as f32 * cell_size.x, row as f32 * cell_size.y);

                                let cell = &mut self.cells[row * WIDTH + col];

                                for (i, object) in cell.objects.iter_mut().enumerate() {
                                    if object.image_index == u16::MAX {
                                        continue;
                                    }

                                    let texture = if let Ok(texture) = get_object_texture(
                                        object_textures,
                                        objects_index,
                                        ui.ctx(),
                                        object.image_index,
                                        objects_file,
                                    ) {
                                        texture
                                    } else {
                                        // TODO: add placeholder texture
                                        object_textures.get(&0).unwrap().clone()
                                    };

                                    let rect = Rect::from_min_size(
                                        origin
                                            + vec2(
                                                object.x_offset as f32,
                                                object.y_offset as f32 - object.height as f32,
                                            ),
                                        texture.size_vec2(),
                                    );

                                    let selected = if let Some(selected) = selected_cell
                                        && let Some(obj) = selected.selected_object
                                        && selected.cell.x as usize == col
                                        && selected.cell.y as usize == row
                                        && obj == i
                                    {
                                        true
                                    } else {
                                        false
                                    };

                                    painter.image(texture.id(), rect, uv, Color32::WHITE);

                                    if selected {
                                        let object_response =
                                            ui.allocate_rect(rect, Sense::click_and_drag());

                                        if object_response.drag_started() {
                                            assert!(dragged_object.is_none());

                                            *dragged_object = Some(DraggedObject {
                                                position: rect.min,
                                                starting_cell: Vec2i::new(col as u32, row as u32),
                                                index: i,
                                                stopped: false,
                                            });
                                        } else if object_response.drag_stopped() {
                                            ui.ctx().output_mut(|o| {
                                                o.cursor_icon = CursorIcon::Default;
                                            });

                                            dragged_object.as_mut().unwrap().stopped = true;
                                        } else if object_response.dragged() {
                                            let delta = object_response.drag_delta();
                                            dragged_object.as_mut().unwrap().position += delta;

                                            ui.ctx().output_mut(|o| {
                                                o.cursor_icon = CursorIcon::Grabbing;
                                            });
                                        } else if object_response.hovered() {
                                            ui.ctx().output_mut(|o| {
                                                o.cursor_icon = CursorIcon::Grab;
                                            });
                                        }

                                        painter.rect(
                                            rect,
                                            0.0,
                                            Color32::from_white_alpha(50),
                                            Stroke::new(2.0, Color32::from_white_alpha(100)),
                                            StrokeKind::Middle,
                                        );
                                    }

                                    if alt_pressed {
                                        let object_response =
                                            ui.allocate_rect(rect, Sense::click());

                                        if object_response.clicked() {
                                            *selected_cell = Some(SelectedCell {
                                                cell: Vec2i::new(col as u32, row as u32),
                                                selected_object: Some(i),
                                            });
                                        }

                                        if !selected {
                                            painter.rect(
                                                rect,
                                                0.0,
                                                if object_response.hovered() {
                                                    Color32::from_white_alpha(50)
                                                } else {
                                                    Color32::TRANSPARENT
                                                },
                                                Stroke::new(2.0, Color32::from_white_alpha(100)),
                                                StrokeKind::Middle,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(DraggedObject {
                            position,
                            starting_cell,
                            index,
                            stopped,
                        }) = *dragged_object
                        {
                            let object = &self.cells
                                [starting_cell.y as usize * WIDTH + starting_cell.x as usize]
                                .objects[index];

                            let texture = if let Ok(texture) = get_object_texture(
                                object_textures,
                                objects_index,
                                ui.ctx(),
                                object.image_index,
                                objects_file,
                            ) {
                                texture
                            } else {
                                // TODO: add placeholder texture
                                object_textures.get(&0).unwrap().clone()
                            };

                            let rect = Rect::from_min_size(position, texture.size_vec2());

                            painter.image(texture.id(), rect, uv, Color32::WHITE);

                            if stopped {
                                let global_pos = pos2(
                                    -response.rect.min.x + position.x,
                                    -response.rect.min.y + position.y + object.height as f32,
                                );

                                let new_cell = Vec2i::new(
                                    (global_pos.x / cell_size.x).floor().max(first_col as f32)
                                        as u32,
                                    (global_pos.y / cell_size.y).floor().max(first_row as f32)
                                        as u32,
                                );

                                let new_cell_f32 = vec2(
                                    new_cell.x as f32 * cell_size.x,
                                    new_cell.y as f32 * cell_size.y,
                                );

                                let new_offset = vec2(
                                    global_pos.x - new_cell_f32.x,
                                    global_pos.y - new_cell_f32.y,
                                );

                                let new_cell_object_index = self.cells
                                    [new_cell.y as usize * WIDTH + new_cell.x as usize]
                                    .objects
                                    .len();

                                command_history.do_action(
                                    self,
                                    Command::MoveObject {
                                        old_cell: starting_cell,
                                        new_cell,
                                        old_index: index,
                                        old_offset: (object.x_offset, object.y_offset),
                                        new_index: new_cell_object_index,
                                        new_offset: (new_offset.x as u8, new_offset.y as u8),
                                    },
                                );

                                *dragged_object = None;
                                *selected_cell = Some(SelectedCell {
                                    cell: new_cell,
                                    selected_object: if new_cell == starting_cell {
                                        Some(index)
                                    } else {
                                        Some(new_cell_object_index)
                                    },
                                });
                            };
                        }
                    });
            });

            ui.vertical(|ui| {
                if let Some(selected_cell) = selected_cell {
                    let mut object_to_remove = None;

                    let coords =
                        selected_cell.cell.y as usize * WIDTH + selected_cell.cell.x as usize;
                    let cell = &mut self.cells[coords];

                    ui.heading("Selected cell");
                    ui.label(format!(
                        "Position: x: {}, y:{}",
                        selected_cell.cell.x, selected_cell.cell.y,
                    ));

                    ui.label(format!("Image index: {}", cell.image_index));
                    ui.label(format!("Overlay image index: {}", cell.overlay_image_index));
                    ui.label(format!("Object count: {}", cell.objects.len()));

                    for (i, object) in cell.objects.iter_mut().enumerate() {
                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("❌").on_hover_text("Remove the object").clicked() {
                                object_to_remove = Some(i);
                            }
                            ui.label(format!("Object {i}"));
                        });

                        Grid::new(i).num_columns(2).striped(true).show(ui, |ui| {
                            ui.label("Object id");
                            ui.add(DragValue::new(&mut object.object_id));
                            ui.end_row();
                            ui.label("Height");
                            ui.add(DragValue::new(&mut object.height));
                            ui.end_row();
                            ui.label("Image index");
                            ui.add(DragValue::new(&mut object.image_index));
                            ui.end_row();
                            ui.label("Unknown");
                            ui.add(DragValue::new(&mut object.unknown0));
                            ui.end_row();
                        });
                    }

                    if let Some(index) = object_to_remove {
                        command_history.do_action(
                            self,
                            Command::RemoveObject {
                                pos: selected_cell.cell,
                                index,
                                object: None,
                            },
                        );
                    }

                    ui.separator();
                }

                ui.heading("Tiles");

                ScrollArea::both()
                    .id_salt("tiles")
                    .show_viewport(ui, |ui, viewport| {
                        let tile_size = vec2(64.0, 64.0);
                        let margin = 4.0;
                        let textures_count = terrain_textures.len();
                        let total_size = vec2(
                            TILES_WIDTH as f32 * (tile_size.x + margin),
                            textures_count as f32 / TILES_WIDTH as f32 * (tile_size.y + margin),
                        );

                        let (response, painter) = ui.allocate_painter(total_size, Sense::empty());

                        let first_row =
                            (viewport.min.y / (tile_size.y + margin)).floor().max(0.0) as usize;
                        let last_row = (viewport.max.y / (tile_size.y + margin))
                            .ceil()
                            .min(textures_count as f32 / TILES_WIDTH as f32)
                            as usize;

                        for row in first_row..last_row {
                            for col in 0..TILES_WIDTH {
                                let origin = response.rect.min
                                    + vec2(
                                        col as f32 * (tile_size.x + margin),
                                        row as f32 * (tile_size.y + margin),
                                    );
                                let rect = Rect::from_min_size(origin, tile_size);

                                let tile_response = ui.allocate_rect(rect, Sense::click());

                                painter.image(
                                    terrain_textures[row * TILES_WIDTH + col].id(),
                                    rect,
                                    uv,
                                    Color32::WHITE,
                                );

                                if tile_response.clicked() {
                                    *selected_tile = Vec2i::new(col as u32, row as u32);
                                } else if tile_response.secondary_clicked() {
                                    *selected_overlay_tile = Vec2i::new(col as u32, row as u32);
                                }

                                if col == selected_tile.x as usize
                                    && row == selected_tile.y as usize
                                {
                                    painter.rect_stroke(
                                        rect,
                                        0.0,
                                        Stroke::new(2.0, Color32::LIGHT_GREEN),
                                        StrokeKind::Inside,
                                    );
                                } else if col == selected_overlay_tile.x as usize
                                    && row == selected_overlay_tile.y as usize
                                {
                                    painter.rect_stroke(
                                        rect,
                                        0.0,
                                        Stroke::new(2.0, Color32::LIGHT_RED),
                                        StrokeKind::Inside,
                                    );
                                }
                            }
                        }
                    });
            });
        });
    }
}

impl Inspector for Cell {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}

impl Inspector for Object {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}

struct WorldEditor {
    terrain_textures: Vec<TextureHandle>,
    objects_index: ImageListIndex,
    object_textures: HashMap<u16, TextureHandle>,
    selected_cell: Option<SelectedCell>,
    objects_file: File,
    selected_tile: Vec2i,
    selected_overlay_tile: Vec2i,
    dragged_object: Option<DraggedObject>,
    command_history: CommandHistory,
}

#[derive(Default)]
struct CommandHistory {
    undo: Vec<Command>,
    redo: Vec<Command>,
}

impl CommandHistory {
    fn do_action(&mut self, world: &mut World, mut command: Command) {
        command.do_action(world);
        self.undo.push(command);
        self.redo.clear();
    }

    fn undo_action(&mut self, world: &mut World) {
        let Some(mut command) = self.undo.pop() else {
            return;
        };

        command.undo_action(world);
        self.redo.push(command);
    }

    fn redo_action(&mut self, world: &mut World) {
        let Some(mut command) = self.redo.pop() else {
            return;
        };

        command.do_action(world);
        self.undo.push(command);
    }
}

enum Command {
    SetImageIndex {
        pos: Vec2i,
        old_image: i16,
        new_image: i16,
    },
    SetOverlayImageIndex {
        pos: Vec2i,
        old_image: i16,
        new_image: i16,
    },
    RemoveObject {
        pos: Vec2i,
        index: usize,
        object: Option<Object>,
    },
    MoveObject {
        old_cell: Vec2i,
        new_cell: Vec2i,
        old_index: usize,
        new_index: usize,
        old_offset: (u8, u8),
        new_offset: (u8, u8),
    },
}

impl Command {
    fn do_action(&mut self, world: &mut World) {
        match self {
            Command::SetImageIndex { pos, new_image, .. } => {
                world.cells[pos.y as usize * WIDTH + pos.x as usize].image_index = *new_image;
            }
            Command::SetOverlayImageIndex { pos, new_image, .. } => {
                world.cells[pos.y as usize * WIDTH + pos.x as usize].overlay_image_index =
                    *new_image;
            }
            Command::RemoveObject { pos, index, object } => {
                let cell = &mut world.cells[pos.y as usize * WIDTH + pos.x as usize];
                let removed_object = cell.objects.remove(*index);
                *object = Some(removed_object);
            }
            Command::MoveObject {
                old_cell,
                old_index,
                new_cell,
                new_offset,
                ..
            } => {
                if old_cell != new_cell {
                    let mut object = world.cells[old_cell.y as usize * WIDTH + old_cell.x as usize]
                        .objects
                        .remove(*old_index);

                    object.x_offset = new_offset.0;
                    object.y_offset = new_offset.1;

                    let cell = &mut world.cells[new_cell.y as usize * WIDTH + new_cell.x as usize];
                    cell.objects.push(object);
                } else {
                    let object = &mut world.cells
                        [old_cell.y as usize * WIDTH + old_cell.x as usize]
                        .objects[*old_index];

                    object.x_offset = new_offset.0;
                    object.y_offset = new_offset.1;
                }
            }
        }
    }

    fn undo_action(&mut self, world: &mut World) {
        match self {
            Command::SetImageIndex { pos, old_image, .. } => {
                world.cells[pos.y as usize * WIDTH + pos.x as usize].image_index = *old_image;
            }
            Command::SetOverlayImageIndex { pos, old_image, .. } => {
                world.cells[pos.y as usize * WIDTH + pos.x as usize].overlay_image_index =
                    *old_image;
            }
            Command::RemoveObject { pos, index, object } => {
                world.cells[pos.y as usize * WIDTH + pos.x as usize]
                    .objects
                    .insert(*index, object.take().expect("object must exist"));
            }
            Command::MoveObject {
                old_cell,
                new_cell,
                old_index,
                new_index,
                old_offset,
                ..
            } => {
                if old_cell != new_cell {
                    let mut object = world.cells[new_cell.y as usize * WIDTH + new_cell.x as usize]
                        .objects
                        .remove(*new_index);

                    object.x_offset = old_offset.0;
                    object.y_offset = old_offset.1;

                    world.cells[old_cell.y as usize * WIDTH + old_cell.x as usize]
                        .objects
                        .insert(*old_index, object);
                } else {
                    let object = &mut world.cells
                        [old_cell.y as usize * WIDTH + old_cell.x as usize]
                        .objects[*old_index];

                    object.x_offset = old_offset.0;
                    object.y_offset = old_offset.1;
                }
            }
        }
    }
}

fn get_object_texture(
    textures: &mut HashMap<u16, TextureHandle>,
    index: &ImageListIndex,
    ctx: &egui::Context,
    image_index: u16,
    file: &mut File,
) -> crate::Result<TextureHandle> {
    let Some(texture) = textures.get(&image_index) else {
        let entry = index
            .get(image_index as usize)
            .ok_or(String::from("index out of range"))?;

        let offset = entry.offset() as usize;
        let next_offset = if let Some(next) = index.get(image_index as usize + 1) {
            next.offset() as usize
        } else {
            file.metadata()?.len() as usize
        };

        let mut bytes = vec![0u8; next_offset - offset];

        file.seek(SeekFrom::Start(offset as u64))?;
        file.read_exact(&mut bytes)?;

        let image = match entry.image_type() {
            ImageType::Opaque => decode_opaque_image(&bytes, entry.width(), entry.height())?,
            ImageType::Transparent => decode_transparent_image(&bytes)?,
        };

        let image_data = ImageData::Color(Arc::new(ColorImage::from_rgba_unmultiplied(
            [image.width() as usize, image.height() as usize],
            &image
                .image_data()
                .iter()
                .flatten()
                .cloned()
                .collect::<Vec<_>>(),
        )));

        let texture = ctx.load_texture("texture", image_data, TextureOptions::NEAREST);
        textures.insert(image_index, texture.clone());

        return Ok(texture);
    };

    Ok(texture.clone())
}

#[derive(Default, Clone, Copy)]
struct Vec2i {
    x: u32,
    y: u32,
}

impl PartialEq for Vec2i {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Vec2i {
    fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

struct SelectedCell {
    cell: Vec2i,
    selected_object: Option<usize>,
}

struct DraggedObject {
    position: Pos2,
    starting_cell: Vec2i,
    index: usize,
    stopped: bool,
}
