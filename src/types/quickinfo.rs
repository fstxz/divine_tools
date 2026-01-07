//! quickinfo.000

use crate::{
    editor::{Inspector, property, property_read_only, struct_ui},
    types::{Binary, FixedCString},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QuickInfo {
    unknown0: u32,
    hero_name: FixedCString<64>,
    thumbnail_size_x: u32,
    thumbnail_size_y: u32,
    divinity_version: FixedCString<64>,
    savegame_version: FixedCString<64>,
    thumbnail_image_data: Vec<u16>,
}

impl Binary for QuickInfo {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let unknown0 = u32::from_bytes(reader)?;
        let hero_name = FixedCString::from_bytes(reader)?;
        let thumbnail_size_x = u32::from_bytes(reader)?;
        let thumbnail_size_y = u32::from_bytes(reader)?;
        let divinity_version = FixedCString::from_bytes(reader)?;
        let savegame_version = FixedCString::from_bytes(reader)?;

        let mut thumbnail_image_data =
            Vec::with_capacity((thumbnail_size_x * thumbnail_size_y) as usize);

        for _ in 0..thumbnail_size_x * thumbnail_size_y {
            thumbnail_image_data.push(reader.read_u16()?);
        }

        Ok(Self {
            unknown0,
            hero_name,
            thumbnail_size_x,
            thumbnail_size_y,
            divinity_version,
            savegame_version,
            thumbnail_image_data,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.hero_name.to_bytes(writer);
        self.thumbnail_size_x.to_bytes(writer);
        self.thumbnail_size_y.to_bytes(writer);
        self.divinity_version.to_bytes(writer);
        self.savegame_version.to_bytes(writer);

        for color in &self.thumbnail_image_data {
            color.to_bytes(writer);
        }
    }
}

impl Inspector for QuickInfo {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("hero_name", &mut self.hero_name, ui);
            property_read_only("thumbnail_size_x", &mut self.thumbnail_size_x, ui);
            property_read_only("thumbnail_size_y", &mut self.thumbnail_size_y, ui);
            property("divinity_version", &mut self.divinity_version, ui);
            property("savegame_version", &mut self.savegame_version, ui);
            property_read_only("thumbnail_image_data", &mut self.thumbnail_image_data, ui);
        });
    }
}
