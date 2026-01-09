//! shroud.xN

use crate::{editor::Inspector, types::Binary};

const WIDTH: usize = 513;
const HEIGHT: usize = 1025;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Shroud {
    unknown0: u8,
    // 513x1025 array (row-major).
    cells: Vec<u8>,
}

impl Binary for Shroud {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        // Every shroud file seems to be 525826 bytes long.
        if reader.len() != 525826 {
            return Err("Corrupted shroud file (or not a shroud file at all)".into());
        }

        let unknown0 = u8::from_bytes(reader)?;

        let mut cells = Vec::with_capacity(WIDTH * HEIGHT);

        for _ in 0..WIDTH * HEIGHT {
            cells.push(reader.read_u8()?);
        }

        Ok(Self { unknown0, cells })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);

        for cell in &self.cells {
            cell.to_bytes(writer);
        }
    }
}

impl Inspector for Shroud {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        if ui.button("Remove fog of war").clicked() {
            self.cells.fill(0);
        }
    }
}
