//! magic.cmp

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Magic {
    spells: Vec<SpellData>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct SpellData {
    id: u32,
    min_level: u32,
    max_level: u32,
    cast: u32,
    connect: u32,
    execute: u32,
    after: u32,
}

impl Binary for Magic {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            spells: <Vec<SpellData>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.spells.to_bytes(writer);
    }
}

impl Binary for SpellData {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            id: u32::from_bytes(reader)?,
            min_level: u32::from_bytes(reader)?,
            max_level: u32::from_bytes(reader)?,
            cast: u32::from_bytes(reader)?,
            connect: u32::from_bytes(reader)?,
            execute: u32::from_bytes(reader)?,
            after: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.id.to_bytes(writer);
        self.min_level.to_bytes(writer);
        self.max_level.to_bytes(writer);
        self.cast.to_bytes(writer);
        self.connect.to_bytes(writer);
        self.execute.to_bytes(writer);
        self.after.to_bytes(writer);
    }
}

impl Inspector for Magic {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("spells", &mut self.spells, ui);
        });
    }
}

impl Inspector for SpellData {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("id", &mut self.id, ui);
            property("min_level", &mut self.min_level, ui);
            property("max_level", &mut self.max_level, ui);
            property("cast", &mut self.cast, ui);
            property("connect", &mut self.connect, ui);
            property("execute", &mut self.execute, ui);
            property("after", &mut self.after, ui);
        });
    }
}
