//! sound.cfg
//!
//! DD only.

use crate::{
    editor::{Inspector, property, property_tooltip, struct_ui},
    types::Binary,
};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct SoundConfig {
    unknown0: u32,
    /// Maximum number of sounds that can play at the same time.
    max_sounds: u32,
    unknown2: u32,
    sound_effects_volume: f32,
    voice_volume: f32,
    music_volume: f32,
    unknown6: u32,
    ambient_volume: f32,
}

impl Binary for SoundConfig {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: u32::from_bytes(reader)?,
            max_sounds: u32::from_bytes(reader)?,
            unknown2: u32::from_bytes(reader)?,
            sound_effects_volume: f32::from_bytes(reader)?,
            voice_volume: f32::from_bytes(reader)?,
            music_volume: f32::from_bytes(reader)?,
            unknown6: u32::from_bytes(reader)?,
            ambient_volume: f32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.max_sounds.to_bytes(writer);
        self.unknown2.to_bytes(writer);
        self.sound_effects_volume.to_bytes(writer);
        self.voice_volume.to_bytes(writer);
        self.music_volume.to_bytes(writer);
        self.unknown6.to_bytes(writer);
        self.ambient_volume.to_bytes(writer);
    }
}

impl Inspector for SoundConfig {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property_tooltip(
                "max_sounds",
                "Maximum number of sounds that can play at the same time.",
                &mut self.max_sounds,
                ui,
            );
            property("unknown2", &mut self.unknown2, ui);
            property("sound_effects_volume", &mut self.sound_effects_volume, ui);
            property("voice_volume", &mut self.voice_volume, ui);
            property("music_volume", &mut self.music_volume, ui);
            property("unknown6", &mut self.unknown6, ui);
            property("ambient_volume", &mut self.ambient_volume, ui);
        });
    }
}
