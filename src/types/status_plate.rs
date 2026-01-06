//! statuspl.cmp

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct StatusPlate {
    unknown: FixedArray<FixedArray<u32, 9>, 10>,
}

impl Binary for StatusPlate {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown: <FixedArray<FixedArray<u32, 9>, 10>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown.to_bytes(writer);
    }
}

impl Inspector for StatusPlate {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown", &mut self.unknown, ui);
        });
    }
}
