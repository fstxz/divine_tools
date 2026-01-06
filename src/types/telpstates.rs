//! telpstates.000

use crate::{
    editor::{Inspector, property, struct_ui},
    types::Binary,
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TelpStates {
    states: Vec<State>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
struct State {
    unknown0: u32,
    unknown1: u32,
}

impl Binary for TelpStates {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            states: <Vec<State>>::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.states.to_bytes(writer);
    }
}

impl Binary for State {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            unknown0: u32::from_bytes(reader)?,
            unknown1: u32::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
    }
}

impl Inspector for TelpStates {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("states", &mut self.states, ui);
        });
    }
}

impl Inspector for State {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("unknown0", &mut self.unknown0, ui);
            property("unknown1", &mut self.unknown1, ui);
        });
    }
}
