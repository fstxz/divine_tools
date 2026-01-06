//! music.dat

use eframe::egui;

use crate::{
    buffer::{BufferReader, BufferWriter},
    editor::{Inspector, property, struct_ui},
    types::{Binary, FixedArray},
};

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Music {
    music_tracks: Vec<MusicTrack>,
    ambient_tracks: Vec<AmbientTrack>,
    regions: Vec<Region>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct MusicTrack {
    title: String,
    file_name: String,
    unknown: u32,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct AmbientTrack {
    title: String,
    file_name: String,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct Region {
    region_name: String,
    unknown0: FixedArray<Vec<RegionTrack>, 5>,
    unknown1: u32,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct RegionTrack {
    title: String,
    volume: f32,
}

impl Inspector for Music {
    fn show(&mut self, ui: &mut egui::Ui) {
        struct_ui(ui, |ui| {
            property("music_tracks", &mut self.music_tracks, ui);
            property("ambient_tracks", &mut self.ambient_tracks, ui);
            property("regions", &mut self.regions, ui);
        });
    }
}

impl Inspector for MusicTrack {
    fn show(&mut self, ui: &mut egui::Ui) {
        struct_ui(ui, |ui| {
            property("title", &mut self.title, ui);
            property("file_name", &mut self.file_name, ui);
            property("unknown", &mut self.unknown, ui);
        });
    }
}

impl Inspector for AmbientTrack {
    fn show(&mut self, ui: &mut egui::Ui) {
        struct_ui(ui, |ui| {
            property("title", &mut self.title, ui);
            property("file_name", &mut self.file_name, ui);
        });
    }
}

impl Inspector for Region {
    fn show(&mut self, ui: &mut egui::Ui) {
        struct_ui(ui, |ui| {
            property("region_name", &mut self.region_name, ui);
            property("unknown0", &mut self.unknown0, ui);
            property("unknown1", &mut self.unknown1, ui);
        });
    }
}

impl Inspector for RegionTrack {
    fn show(&mut self, ui: &mut egui::Ui) {
        struct_ui(ui, |ui| {
            property("title", &mut self.title, ui);
            property("volume", &mut self.volume, ui);
        });
    }
}

impl Binary for Music {
    fn to_bytes(&self, writer: &mut BufferWriter) {
        self.music_tracks.to_bytes(writer);
        self.ambient_tracks.to_bytes(writer);
        self.regions.to_bytes(writer);
    }

    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Music> {
        Ok(Music {
            music_tracks: <Vec<MusicTrack>>::from_bytes(reader)?,
            ambient_tracks: <Vec<AmbientTrack>>::from_bytes(reader)?,
            regions: <Vec<Region>>::from_bytes(reader)?,
        })
    }
}

impl Binary for MusicTrack {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(MusicTrack {
            title: String::from_bytes(reader)?,
            file_name: String::from_bytes(reader)?,
            unknown: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        self.title.to_bytes(writer);
        self.file_name.to_bytes(writer);
        self.unknown.to_bytes(writer);
    }
}

impl Binary for AmbientTrack {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(AmbientTrack {
            title: String::from_bytes(reader)?,
            file_name: String::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        self.title.to_bytes(writer);
        self.file_name.to_bytes(writer);
    }
}

impl Binary for Region {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Region {
            region_name: String::from_bytes(reader)?,
            unknown0: <FixedArray<Vec<RegionTrack>, 5>>::from_bytes(reader)?,
            unknown1: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        self.region_name.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
    }
}

impl Binary for RegionTrack {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(RegionTrack {
            title: String::from_bytes(reader)?,
            volume: f32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        self.title.to_bytes(writer);
        self.volume.to_bytes(writer);
    }
}
