//! osiobjects.000
//!
//! In BD it only appears in data.000.

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OsirisObjects {
    objects: Vec<Object>,
    // Appears to be the number of objects, but no idea why it's here
    // or what purpose it has.
    unknown0: u32,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Object {
    index: u32,
    id: u32,
}

impl Binary for OsirisObjects {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            objects: <Vec<Object>>::from_bytes(reader)?,
            unknown0: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.objects.to_bytes(writer);
        self.unknown0.to_bytes(writer);
    }
}

impl Binary for Object {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            index: u32::from_bytes(reader)?,
            id: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.index.to_bytes(writer);
        self.id.to_bytes(writer);
    }
}

impl Inspector for OsirisObjects {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("objects", &mut self.objects, ui);
            property("unknown0", &mut self.unknown0, ui);
        });
    }
}

impl Inspector for Object {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("index", &mut self.index, ui);
            property("id", &mut self.id, ui);
        });
    }
}
