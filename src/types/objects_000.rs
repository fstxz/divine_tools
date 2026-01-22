//! objects.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray, FixedCString},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Objects000 {
    objects: Vec<Object>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Object {
    unknown0: FixedArray<u32, 8>,
    name: FixedCString<16>,
    id: u32,
    unknown1: FixedArray<u32, 24>,
}

impl Binary for Objects000 {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut objects = Vec::new();

        // Length is not encoded anywhere, but the size of each
        // object is 148 bytes, so we can use that.
        if !reader.len().is_multiple_of(148) {
            return Err("this file does not appear to be valid".into());
        }

        while !reader.is_empty() {
            objects.push(Object::from_bytes(reader)?);
        }

        Ok(Self { objects })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        for object in &self.objects {
            object.to_bytes(writer);
        }
    }
}

impl Binary for Object {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: <FixedArray<u32, 8>>::from_bytes(reader)?,
            name: FixedCString::from_bytes(reader)?,
            id: u32::from_bytes(reader)?,
            unknown1: <FixedArray<u32, 24>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.name.to_bytes(writer);
        self.id.to_bytes(writer);
        self.unknown1.to_bytes(writer);
    }
}

impl Inspector for Objects000 {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("objects", &mut self.objects, ui);
        });
    }
}

impl Inspector for Object {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("name", &mut self.name, ui);
            property("id", &mut self.id, ui);
            property("unknown1", &mut self.unknown1, ui);
        });
    }
}
