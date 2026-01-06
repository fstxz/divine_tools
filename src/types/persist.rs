//! persist.dat

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Persist {
    elements: Vec<Unknown0>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Unknown0 {
    unknown0: FixedArray<u32, 8>,
}

impl Binary for Persist {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            elements: <Vec<Unknown0>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.elements.to_bytes(writer);
    }
}

impl Binary for Unknown0 {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: <FixedArray<u32, 8>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
    }
}

impl Inspector for Persist {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("elements", &mut self.elements, ui);
        });
    }
}

impl Inspector for Unknown0 {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
        });
    }
}
