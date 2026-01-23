//! world.xN

use crate::{editor::Inspector, types::Binary};

const WIDTH: usize = 512;
const HEIGHT: usize = 1024;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct World {
    cells: Box<[Cell]>,
    unknown1: u32,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct Cell {
    image_index: i16,
    overlay_image_index: i16,
    unknown0: u16,
    unknown1: u8,
    unknown2: u32,
    unknown3: u32,
    objects: Vec<Object>,
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
struct Object {
    unknown0: u16,
    unknown1: u16,
    unknown2: u16,
    unknown3: u16,
}

impl Binary for World {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        // Row offsets, will be calculated during serialization.
        reader.skip(HEIGHT * 4);

        let mut cells = vec![Cell::default(); WIDTH * HEIGHT];

        for y in 0..HEIGHT {
            // Cell offsets, will be calculated during serialization.
            reader.skip(WIDTH * 2);
            for x in 0..WIDTH {
                cells[y * WIDTH + x] = Cell::from_bytes(reader)?;
            }
        }

        Ok(Self {
            cells: cells.into_boxed_slice(),
            unknown1: reader.read_u32()?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        let mut row_offsets = Vec::<u32>::with_capacity(HEIGHT);
        let mut row_writer = crate::buffer::BufferWriter::new();

        for y in 0..HEIGHT {
            row_offsets.push((HEIGHT * 4 + row_writer.len()) as u32);

            let mut cell_writer = crate::buffer::BufferWriter::new();

            for x in 0..WIDTH {
                (cell_writer.len() as u16).to_bytes(&mut row_writer);

                let cell = &self.cells[y * WIDTH + x];
                cell.to_bytes(&mut cell_writer);
            }

            row_writer.write_bytes(&cell_writer.finish());
        }

        for offset in row_offsets {
            offset.to_bytes(writer);
        }

        writer.write_bytes(&row_writer.finish());
        self.unknown1.to_bytes(writer);
    }
}

impl Binary for Cell {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let image_index = reader.read_i16()?;
        let overlay_image_index = reader.read_i16()?;

        let unknown0 = reader.read_u16()?;

        let objects_len = reader.read_u8()?;
        let unknown1 = reader.read_u8()?;

        let unknown2 = reader.read_u32()?;
        let unknown3 = reader.read_u32()?;

        let mut objects = Vec::with_capacity(objects_len as usize);

        for _ in 0..objects_len {
            objects.push(Object::from_bytes(reader)?);
        }

        Ok(Self {
            image_index,
            overlay_image_index,
            unknown0,
            unknown1,
            unknown2,
            unknown3,
            objects,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.image_index.to_bytes(writer);
        self.overlay_image_index.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        (self.objects.len() as u8).to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.unknown2.to_bytes(writer);
        self.unknown3.to_bytes(writer);

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
            unknown0: reader.read_u16()?,
            unknown1: reader.read_u16()?,
            unknown2: reader.read_u16()?,
            unknown3: reader.read_u16()?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.unknown0.to_bytes(writer);
        self.unknown1.to_bytes(writer);
        self.unknown2.to_bytes(writer);
        self.unknown3.to_bytes(writer);
    }
}

impl Inspector for World {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}

impl Inspector for Cell {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}

impl Inspector for Object {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}
