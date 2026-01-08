//! quickinfo.000

use crate::{
    editor::{Inspector, property, property_read_only, struct_ui},
    types::{Binary, FixedCString},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QuickInfo {
    // The game always sets this to 0 when saving.
    unknown0: u32,
    player_name: FixedCString<64>,
    thumbnail_width: u32,
    thumbnail_height: u32,
    game_version: FixedCString<64>,
    save_version: FixedCString<64>,
    // Either RGB555 or RGB565, not sure.
    thumbnail_image_data: Vec<u16>,
}

impl Binary for QuickInfo {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let unknown0 = u32::from_bytes(reader)?;
        let player_name = FixedCString::from_bytes(reader)?;
        let thumbnail_width = u32::from_bytes(reader)?;
        let thumbnail_height = u32::from_bytes(reader)?;
        let game_version = FixedCString::from_bytes(reader)?;
        let save_version = FixedCString::from_bytes(reader)?;

        let mut thumbnail_image_data =
            Vec::with_capacity((thumbnail_width * thumbnail_height) as usize);

        for _ in 0..thumbnail_width * thumbnail_height {
            thumbnail_image_data.push(reader.read_u16()?);
        }

        Ok(Self {
            unknown0,
            player_name,
            thumbnail_width,
            thumbnail_height,
            game_version,
            save_version,
            thumbnail_image_data,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.player_name.to_bytes(writer);
        self.thumbnail_width.to_bytes(writer);
        self.thumbnail_height.to_bytes(writer);
        self.game_version.to_bytes(writer);
        self.save_version.to_bytes(writer);

        for color in &self.thumbnail_image_data {
            color.to_bytes(writer);
        }
    }
}

impl Inspector for QuickInfo {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("player_name", &mut self.player_name, ui);
            property_read_only("thumbnail_width", &mut self.thumbnail_width, ui);
            property_read_only("thumbnail_height", &mut self.thumbnail_height, ui);
            property("game_version", &mut self.game_version, ui);
            property("save_version", &mut self.save_version, ui);
            property_read_only("thumbnail_image_data", &mut self.thumbnail_image_data, ui);
        });
    }
}
