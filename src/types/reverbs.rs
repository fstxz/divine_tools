//! reverbs.dat
//!
//! DD only.

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Reverbs {
    elements: Vec<Unknown0>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Unknown0 {
    name: String,
    unknown0: u32,
    unknown1: u32,
    unknown2: u32,
    unknown3: f32,
    unknown4: f32,
    unknown5: f32,
    unknown6: f32,
    unknown7: f32,
    unknown8: f32,
    unknown9: f32,
    unknown10: f32,
    unknown11: f32,
    unknown12: f32,
    unknown13: Vec<String>,
}

impl Binary for Reverbs {
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
            name: String::from_bytes(reader)?,
            unknown0: u32::from_bytes(reader)?,
            unknown1: u32::from_bytes(reader)?,
            unknown2: u32::from_bytes(reader)?,
            unknown3: f32::from_bytes(reader)?,
            unknown4: f32::from_bytes(reader)?,
            unknown5: f32::from_bytes(reader)?,
            unknown6: f32::from_bytes(reader)?,
            unknown7: f32::from_bytes(reader)?,
            unknown8: f32::from_bytes(reader)?,
            unknown9: f32::from_bytes(reader)?,
            unknown10: f32::from_bytes(reader)?,
            unknown11: f32::from_bytes(reader)?,
            unknown12: f32::from_bytes(reader)?,
            unknown13: <Vec<String>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.name.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.unknown2.to_bytes(writer);
        self.unknown3.to_bytes(writer);
        self.unknown4.to_bytes(writer);
        self.unknown5.to_bytes(writer);
        self.unknown6.to_bytes(writer);
        self.unknown7.to_bytes(writer);
        self.unknown8.to_bytes(writer);
        self.unknown9.to_bytes(writer);
        self.unknown10.to_bytes(writer);
        self.unknown11.to_bytes(writer);
        self.unknown12.to_bytes(writer);
        self.unknown13.to_bytes(writer);
    }
}

impl Inspector for Reverbs {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("elements", &mut self.elements, ui);
        });
    }
}

impl Inspector for Unknown0 {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("name", &mut self.name, ui);
            property("unknown0", &mut self.unknown0, ui);
            property("unknown1", &mut self.unknown1, ui);
            property("unknown2", &mut self.unknown2, ui);
            property("unknown3", &mut self.unknown3, ui);
            property("unknown4", &mut self.unknown4, ui);
            property("unknown5", &mut self.unknown5, ui);
            property("unknown6", &mut self.unknown6, ui);
            property("unknown7", &mut self.unknown7, ui);
            property("unknown8", &mut self.unknown8, ui);
            property("unknown9", &mut self.unknown9, ui);
            property("unknown10", &mut self.unknown10, ui);
            property("unknown11", &mut self.unknown11, ui);
            property("unknown12", &mut self.unknown12, ui);
            property("unknown13", &mut self.unknown13, ui);
        });
    }
}
