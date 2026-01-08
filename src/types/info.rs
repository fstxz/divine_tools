//! info.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Info {
    unknown0: u32,
    unknown1: u32,
    unknown2: u32,
    current_health: u32,
    current_mana: u32,
    unknown3: u32,
    unknown4: u32,
    unknown5: u32,
}

impl Binary for Info {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: u32::from_bytes(reader)?,
            unknown1: u32::from_bytes(reader)?,
            unknown2: u32::from_bytes(reader)?,
            current_health: u32::from_bytes(reader)?,
            current_mana: u32::from_bytes(reader)?,
            unknown3: u32::from_bytes(reader)?,
            unknown4: u32::from_bytes(reader)?,
            unknown5: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.unknown2.to_bytes(writer);
        self.current_health.to_bytes(writer);
        self.current_mana.to_bytes(writer);
        self.unknown3.to_bytes(writer);
        self.unknown4.to_bytes(writer);
        self.unknown5.to_bytes(writer);
    }
}

impl Inspector for Info {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("unknown1", &mut self.unknown1, ui);
            property("unknown2", &mut self.unknown2, ui);
            property("current_health", &mut self.current_health, ui);
            property("current_mana", &mut self.current_mana, ui);
            property("unknown3", &mut self.unknown3, ui);
            property("unknown4", &mut self.unknown4, ui);
            property("unknown5", &mut self.unknown5, ui);
        });
    }
}
