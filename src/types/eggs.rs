//! eggs.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Eggs {
    eggs: Vec<Egg>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Egg {
    unknown0: FixedArray<u32, 23>,
}

impl Binary for Eggs {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            eggs: <Vec<Egg>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.eggs.to_bytes(writer);
    }
}

impl Binary for Egg {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: <FixedArray<u32, 23>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
    }
}

impl Inspector for Eggs {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("eggs", &mut self.eggs, ui);
        });
    }
}

impl Inspector for Egg {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
        });
    }
}
