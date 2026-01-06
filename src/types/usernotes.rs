//! usernotes.bin, mapflags.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, CStringWithLength},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Notes {
    user_notes: Vec<Note>,
    notes: Vec<Note>,
    unknown0: u32,
    unknown1: u32,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Note {
    position_x: i16,
    position_y: i16,
    unknown0: u32,
    is_visible: u32,
    id: u32,
    unknown1: u32,
    text: CStringWithLength,
    unknown2: u32,
}

impl Binary for Notes {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let user_notes_count = u32::from_bytes(reader)?;
        // User notes' buffer length. We will calculate it during serialization.
        reader.skip(4);

        let mut user_notes = Vec::with_capacity(user_notes_count as usize);

        for _ in 0..user_notes_count {
            user_notes.push(Note::from_bytes(reader)?);
        }

        let note_count = reader.read_u32()?;
        // Notes' buffer length. We will calculate it during serialization.
        reader.skip(4);

        let mut notes = Vec::with_capacity(note_count as usize);

        for _ in 0..note_count {
            notes.push(Note::from_bytes(reader)?);
        }

        let unknown0 = reader.read_u32()?;
        let unknown1 = reader.read_u32()?;

        Ok(Self {
            user_notes,
            unknown0,
            unknown1,
            notes,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        (self.user_notes.len() as u32).to_bytes(writer);

        let mut user_notes_buffer_length = 0u32;

        for note in &self.user_notes {
            user_notes_buffer_length += (note.text.inner.count_bytes() + 1 + 28) as u32
        }

        user_notes_buffer_length.to_bytes(writer);

        for note in &self.user_notes {
            note.to_bytes(writer);
        }

        (self.notes.len() as u32).to_bytes(writer);

        let mut notes_buffer_length = 0u32;

        for note in &self.notes {
            notes_buffer_length += (note.text.inner.count_bytes() + 1 + 28) as u32
        }

        notes_buffer_length.to_bytes(writer);

        for note in &self.notes {
            note.to_bytes(writer);
        }

        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
    }
}

impl Binary for Note {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            position_x: i16::from_bytes(reader)?,
            position_y: i16::from_bytes(reader)?,
            unknown0: u32::from_bytes(reader)?,
            is_visible: u32::from_bytes(reader)?,
            id: u32::from_bytes(reader)?,
            unknown1: u32::from_bytes(reader)?,
            text: CStringWithLength::from_bytes(reader)?,
            unknown2: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.position_x.to_bytes(writer);
        self.position_y.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        self.is_visible.to_bytes(writer);
        self.id.to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.text.to_bytes(writer);
        self.unknown2.to_bytes(writer);
    }
}

impl Inspector for Notes {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("user_notes", &mut self.user_notes, ui);
            property("notes", &mut self.notes, ui);
            property("unknown1", &mut self.unknown0, ui);
            property("unknown2", &mut self.unknown1, ui);
        });
    }
}

impl Inspector for Note {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("position_x", &mut self.position_x, ui);
            property("position_y", &mut self.position_y, ui);
            property("unknown0", &mut self.unknown0, ui);
            property("is_visible", &mut self.is_visible, ui);
            property("id", &mut self.id, ui);
            property("unknown1", &mut self.unknown1, ui);
            property("text", &mut self.text, ui);
            property("unknown2", &mut self.unknown2, ui);
        });
    }
}
