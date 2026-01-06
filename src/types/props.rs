//! props.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Props {
    props: Vec<Unknown0>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Unknown0 {
    // TODO: null-terminated, even though it has a length
    // check if the game cares
    unknown0: String,
    unknown1: Vec<u32>,
}

impl Binary for Props {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            props: <Vec<Unknown0>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.props.to_bytes(writer);
    }
}

impl Binary for Unknown0 {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: String::from_bytes(reader)?,
            unknown1: <Vec<u32>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
    }
}

impl Inspector for Props {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("props", &mut self.props, ui);
        });
    }
}

impl Inspector for Unknown0 {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("unknown1", &mut self.unknown1, ui);
        });
    }
}
