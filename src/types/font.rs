//! .fnt

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Font {
    unknown0: FixedArray<u8, 18>,
    glyph_count: u32,
    glyphs: Vec<GlyphData>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct GlyphData {
    unknown0: FixedArray<u8, 20>,
    character: char,
    buffer_length: u32,
    buffer: Vec<u8>,
}

impl Binary for Font {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let unknown0 = <FixedArray<u8, 18>>::from_bytes(reader)?;
        let glyph_count = u32::from_bytes(reader)?;
        reader.skip(1024);
        let mut glyphs = Vec::new();

        for _ in 0..glyph_count {
            glyphs.push(GlyphData::from_bytes(reader)?);
        }

        Ok(Self {
            unknown0,
            glyph_count,
            glyphs,
        })
    }

    fn to_bytes(&self, _writer: &mut crate::buffer::BufferWriter) {
        todo!()
    }
}

impl Binary for GlyphData {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let unknown0 = <FixedArray<u8, 20>>::from_bytes(reader)?;
        let character = u8::from_bytes(reader)? as char;
        reader.skip(1);
        let buffer_length = u32::from_bytes(reader)?;
        let buffer = Vec::new();

        // -4 because length number itself is included in the buffer length.
        reader.skip((buffer_length - 4) as usize);
        // For some reason the buffer is null-terminated.
        reader.skip(1);

        Ok(GlyphData {
            unknown0,
            character,
            buffer_length,
            buffer,
        })
    }

    fn to_bytes(&self, _writer: &mut crate::buffer::BufferWriter) {
        todo!()
    }
}

impl Inspector for GlyphData {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("character", &mut self.character, ui);
            property("buffer_length", &mut self.buffer_length, ui);
            property("buffer", &mut self.buffer, ui);
        });
    }
}

impl Inspector for Font {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("glyph_count", &mut self.glyph_count, ui);
            property("glyphs", &mut self.glyphs, ui);
        });
    }
}
