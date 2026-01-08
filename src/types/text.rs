//! text.cmp

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Text {
    entries: Vec<TextEntry>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct TextEntry {
    unknown0: u32,
    entries: Vec<TextEntry2>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct TextEntry2 {
    id: u32,
    text1: String,
    text2: String,
}

impl Binary for Text {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            entries: Vec::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.entries.to_bytes(writer);
    }
}

impl Binary for TextEntry {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(TextEntry {
            unknown0: u32::from_bytes(reader)?,
            entries: Vec::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.entries.to_bytes(writer);
    }
}

impl Binary for TextEntry2 {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let id = u32::from_bytes(reader)?;

        // Strings are null-terminated, but the null byte is not included in the length.
        let text1 = String::from_bytes(reader)?;

        if text1.len() > 0 {
            // There is no null byte if the string is empty.
            reader.skip(1);
        }

        let text2 = String::from_bytes(reader)?;

        if text2.len() > 0 {
            reader.skip(1);
        }

        Ok(Self { id, text1, text2 })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.id.to_bytes(writer);

        self.text1.to_bytes(writer);

        if self.text1.len() > 0 {
            writer.pad(1);
        }

        self.text2.to_bytes(writer);

        if self.text2.len() > 0 {
            writer.pad(1);
        }
    }
}

impl Inspector for Text {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("entries", &mut self.entries, ui);
        });
    }
}

impl Inspector for TextEntry {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("entries", &mut self.entries, ui);
        });
    }
}

impl Inspector for TextEntry2 {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("id", &mut self.id, ui);
            property("text1", &mut self.text1, ui);
            property("text2", &mut self.text2, ui);
        });
    }
}
