//! osinames.000
//!
//! DD only.

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedCString},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct OsirisNames {
    names: Vec<Name>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Name {
    id: u32,
    name: FixedCString<32>,
}

impl Binary for OsirisNames {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            names: <Vec<Name>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.names.to_bytes(writer);
    }
}

impl Binary for Name {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            id: u32::from_bytes(reader)?,
            name: FixedCString::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.id.to_bytes(writer);
        self.name.to_bytes(writer);
    }
}

impl Inspector for OsirisNames {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("names", &mut self.names, ui);
        });
    }
}

impl Inspector for Name {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("id", &mut self.id, ui);
            property("name", &mut self.name, ui);
        });
    }
}
