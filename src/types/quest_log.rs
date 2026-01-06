//! quest_log.000
//!
//! DD only.

use std::any::TypeId;

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct QuestLog {
    entries: Vec<QuestLogEntry>,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct QuestLogEntry {
    id: u32,
    unknown0: bool,
    status: QuestStatus,
    unknown1: u32,
    day: i32,
    hour: i32,
    minute: i32,
    day2: i32,
    hour2: i32,
    minute2: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
enum QuestStatus {
    #[default]
    NotStarted,
    InProgress,
    Failed,
    Completed,
}

impl Binary for QuestLog {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        // Header
        reader.skip(6);

        let mut entries = Vec::new();

        while !reader.is_empty() {
            entries.push(QuestLogEntry::from_bytes(reader)?);
        }

        Ok(Self { entries })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        writer.write_bytes("ML3ID".as_bytes());
        writer.pad(1);

        for entry in &self.entries {
            entry.to_bytes(writer);
        }
    }
}

impl Binary for QuestLogEntry {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            id: u32::from_bytes(reader)?,
            unknown0: bool::from_bytes(reader)?,
            status: QuestStatus::from_bytes(reader)?,
            unknown1: u32::from_bytes(reader)?,
            day: i32::from_bytes(reader)?,
            hour: i32::from_bytes(reader)?,
            minute: i32::from_bytes(reader)?,
            day2: i32::from_bytes(reader)?,
            hour2: i32::from_bytes(reader)?,
            minute2: i32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.id.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        self.status.to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.day.to_bytes(writer);
        self.hour.to_bytes(writer);
        self.minute.to_bytes(writer);
        self.day2.to_bytes(writer);
        self.hour2.to_bytes(writer);
        self.minute2.to_bytes(writer);
    }
}

impl Inspector for QuestLog {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("entries", &mut self.entries, ui);
        });
    }
}

impl Inspector for QuestLogEntry {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("id", &mut self.id, ui);
            property("unknown0", &mut self.unknown0, ui);
            property("status", &mut self.status, ui);
            property("unknown1", &mut self.unknown1, ui);
            property("day", &mut self.day, ui);
            property("hour", &mut self.hour, ui);
            property("minute", &mut self.minute, ui);
            property("day2", &mut self.day2, ui);
            property("hour2", &mut self.hour2, ui);
            property("minute2", &mut self.minute2, ui);
        });
    }
}

impl Binary for QuestStatus {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match reader.read_u8()? {
            0 => Ok(Self::NotStarted),
            1 => Ok(Self::InProgress),
            2 => Ok(Self::Failed),
            3 => Ok(Self::Completed),
            _ => Err("invalid value for QuestStatus".into()),
        }
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        writer.write_u8(match self {
            Self::NotStarted => 0,
            Self::InProgress => 1,
            Self::Failed => 2,
            Self::Completed => 3,
        });
    }
}

impl Inspector for QuestStatus {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        let selected_text = match self {
            QuestStatus::NotStarted => "Not started",
            QuestStatus::InProgress => "In progress",
            QuestStatus::Failed => "Failed",
            QuestStatus::Completed => "Completed",
        };

        eframe::egui::ComboBox::from_id_salt(TypeId::of::<QuestStatus>())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Self::NotStarted, "Not started");
                ui.selectable_value(self, Self::InProgress, "In progress");
                ui.selectable_value(self, Self::Failed, "Failed");
                ui.selectable_value(self, Self::Completed, "Completed");
            });
    }
}
