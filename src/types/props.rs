//! props.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, CStringWithLength},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Props {
    props: Vec<Property>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Property {
    name: CStringWithLength,
    values: Vec<u32>,
}

impl Binary for Props {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            props: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.props.to_bytes(writer);
    }
}

impl Binary for Property {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            name: Binary::from_bytes(reader)?,
            values: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.name.to_bytes(writer);
        self.values.to_bytes(writer);
    }
}

impl Inspector for Props {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("props", &mut self.props, ui);
        });
    }
}

impl Inspector for Property {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("name", &mut self.name, ui);
            property("values", &mut self.values, ui);
        });
    }
}
