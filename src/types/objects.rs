//! objects.xN

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray},
};

const OBJECT_SIZE: usize = 28;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Objects {
    objects: Vec<Object>,
}

impl Objects {
    pub fn get(&self, index: usize) -> Option<&Object> {
        self.objects.get(index)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct Object {
    unknown0: FixedArray<u8, 26>,
    image_index: u16,
}

impl Object {
    pub fn image_index(&self) -> u16 {
        self.image_index
    }
}

impl Binary for Objects {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        if !reader.len().is_multiple_of(OBJECT_SIZE) {
            return Err("Wrong format".into());
        }

        let object_count = reader.len() / OBJECT_SIZE;
        let mut objects = Vec::with_capacity(object_count);

        for _ in 0..object_count {
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
            unknown0: FixedArray::from_bytes(reader)?,
            image_index: u16::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.image_index.to_bytes(writer);
    }
}

impl Inspector for Objects {
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
            property("image_index", &mut self.image_index, ui);
        });
    }
}
