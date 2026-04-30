//! Image lists.

use std::path::Path;

use crate::{
    buffer::{BufferReader, BufferWriter},
    editor::Inspector,
    types::Binary,
};

const INDEX_ENTRY_SIZE: usize = 56;

pub struct ImageList {
    images: Vec<Image>,
}

impl ImageList {
    pub fn images(&self) -> &[Image] {
        &self.images
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ImageListIndex {
    entries: Vec<IndexEntry>,
}

impl ImageListIndex {
    pub fn get(&self, index: usize) -> Option<&IndexEntry> {
        self.entries.get(index)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct IndexEntry {
    offset: u32,
    width: u32,
    height: u32,
    image_type: ImageType,
    unknown0: u32,
    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
    unknown4: u16,
    unknown5: u16,
    unknown6: u32,
    unknown7: u32,
    unknown8: u32,
    unknown9: u32,
    unknown10: u32,
}

impl IndexEntry {
    pub fn offset(&self) -> u32 {
        self.offset
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn image_type(&self) -> ImageType {
        self.image_type
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Copy)]
pub enum ImageType {
    #[default]
    Opaque = 0,
    Transparent = 1,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Image {
    width: u32,
    height: u32,
    image_data: Vec<[u8; 4]>,
}

impl Image {
    pub fn new(width: u32, height: u32, image_data: Vec<[u8; 4]>) -> Self {
        Self {
            width,
            height,
            image_data,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn image_data(&self) -> &[[u8; 4]] {
        &self.image_data
    }

    pub fn encode_as_transparent(&self) -> Vec<u8> {
        let mut image_writer = BufferWriter::new();

        let mut chunk_writer = BufferWriter::new();
        let mut pixel_writer = BufferWriter::new();

        for row in self.image_data.chunks_exact(self.width as usize) {
            let mut chunks: Vec<(u16, u16)> = Vec::new(); // (pixel_offset, pixel_count)
            let mut pixel_offset = 0;
            let mut pixel_count = 0;

            let mut last_pixel_alpha = row[0][3];

            let row_offset = pixel_writer.len() as u32;

            for (i, color) in row.iter().enumerate() {
                let alpha = color[3];

                if alpha != last_pixel_alpha {
                    if alpha == 0 {
                        chunks.push((pixel_offset, pixel_count));
                        pixel_count = 0;
                    } else {
                        pixel_offset = i as u16;
                    }
                }

                if alpha > 0 {
                    pixel_writer.write_u16(r8g8b8a8_to_r5g6b5(color));
                    pixel_count += 1;
                }

                last_pixel_alpha = alpha;
            }

            // Add remaining chunk.
            if last_pixel_alpha == 255 {
                chunks.push((pixel_offset, pixel_count));
            }

            let chunk_count = chunks.len() as u16;

            chunk_writer.write_u16(chunk_count);

            if chunks.is_empty() {
                chunk_writer.pad(6);
            } else {
                chunk_writer.write_u32(row_offset);

                for (pixel_offset, pixel_count) in chunks {
                    chunk_writer.write_u16(pixel_offset);
                    chunk_writer.write_u16(pixel_count);
                }

                chunk_writer.write_u16(chunk_count);
            }
        }

        // +12 to include the lengths themselves as well as width and height.
        image_writer.write_u32((chunk_writer.len() + pixel_writer.len()) as u32 + 12);
        image_writer.write_u32(chunk_writer.len() as u32 + 12);

        image_writer.write_u16(self.width as u16);
        image_writer.write_u16(self.height as u16);

        image_writer.write_bytes(&chunk_writer.finish());
        image_writer.write_bytes(&pixel_writer.finish());

        image_writer.finish()
    }
}

impl Binary for ImageListIndex {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        if !reader.len().is_multiple_of(INDEX_ENTRY_SIZE) {
            return Err("Invalid index file".into());
        }

        let index_entry_count = reader.len() / INDEX_ENTRY_SIZE;

        let mut entries = Vec::new();

        for _ in 0..index_entry_count {
            let offset = reader.read_u32()?;
            let width = reader.read_u32()?;
            let height = reader.read_u32()?;

            let image_type = match reader.read_u32()? {
                0 => ImageType::Opaque,
                1 => ImageType::Transparent,
                unknown => return Err(format!("Unknown image type: {unknown}").into()),
            };

            let unknown0 = reader.read_u32()?;
            let unknown1 = reader.read_u32()?;

            // width - 1
            let unknown2 = reader.read_u32()?;
            // height - 1
            let unknown3 = reader.read_u32()?;

            let unknown4 = reader.read_u16()?;
            let unknown5 = reader.read_u16()?;

            let unknown6 = reader.read_u32()?;
            let unknown7 = reader.read_u32()?;
            let unknown8 = reader.read_u32()?;
            let unknown9 = reader.read_u32()?;
            let unknown10 = reader.read_u32()?;

            entries.push(IndexEntry {
                offset,
                width,
                height,
                image_type,
                unknown0,
                unknown1,
                unknown2,
                unknown3,
                unknown4,
                unknown5,
                unknown6,
                unknown7,
                unknown8,
                unknown9,
                unknown10,
            });
        }

        Ok(Self { entries })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        for entry in &self.entries {
            entry.offset.to_bytes(writer);
            entry.width.to_bytes(writer);
            entry.height.to_bytes(writer);
            (entry.image_type as u32).to_bytes(writer);
            entry.unknown0.to_bytes(writer);
            entry.unknown1.to_bytes(writer);
            entry.unknown2.to_bytes(writer);
            entry.unknown3.to_bytes(writer);
            entry.unknown4.to_bytes(writer);
            entry.unknown5.to_bytes(writer);
            entry.unknown6.to_bytes(writer);
            entry.unknown7.to_bytes(writer);
            entry.unknown8.to_bytes(writer);
            entry.unknown9.to_bytes(writer);
            entry.unknown10.to_bytes(writer);
        }
    }
}

impl Inspector for ImageListIndex {
    fn show(&mut self, _ui: &mut eframe::egui::Ui) {}
}

pub fn load_image_list(
    image_list_path: &Path,
    index_path: &Path,
    compressed: bool,
) -> crate::Result<ImageList> {
    let image_list_bytes = std::fs::read(image_list_path)?;
    let index_bytes = std::fs::read(index_path)?;

    let mut index_reader = BufferReader::new(&index_bytes);
    let index = ImageListIndex::from_bytes(&mut index_reader)?;

    let mut image_list_reader = BufferReader::new(&image_list_bytes);

    let mut images = Vec::with_capacity(index.entries.len());

    for (i, entry) in index.entries.iter().enumerate() {
        image_list_reader.set_position(entry.offset as usize);

        let next_offset = if let Some(next) = index.entries.get(i + 1) {
            next.offset as usize
        } else {
            image_list_bytes.len()
        };

        if entry.offset as usize > next_offset {
            return Err("Current entry's offset is larger than the next entry's offset".into());
        }

        let size = next_offset - entry.offset as usize;

        let bytes = image_list_reader.read_bytes(size)?;

        let bytes = if compressed {
            &decompress(bytes)?
        } else {
            bytes
        };

        let image = match entry.image_type {
            ImageType::Opaque => decode_opaque_image(bytes, entry.width, entry.height)?,
            ImageType::Transparent => decode_transparent_image(bytes)?,
        };

        images.push(image);
    }

    assert!(index_reader.is_empty());

    Ok(ImageList { images })
}

pub fn decode_opaque_image(bytes: &[u8], width: u32, height: u32) -> crate::Result<Image> {
    let mut reader = BufferReader::new(bytes);
    let mut image_data = Vec::with_capacity((width * height) as usize);

    for _ in 0..width * height {
        let color = r5g6b5_to_r8g8b8(reader.read_u16()?);
        image_data.push([color[0], color[1], color[2], 255]);
    }

    Ok(Image {
        width,
        height,
        image_data,
    })
}

/// Decodes transparent image from `bytes`.
pub fn decode_transparent_image(bytes: &[u8]) -> crate::Result<Image> {
    let mut reader = BufferReader::new(bytes);
    let _size = reader.read_u32()?;

    // Offset to image data.
    let data_offset = reader.read_u32()? as usize;

    // Width and height of the final image.
    let width = reader.read_u16()? as u32;
    let height = reader.read_u16()? as u32;

    // Initialize image data buffer (fully transparent by default).
    let mut image = Image {
        width,
        height,
        image_data: vec![[0u8, 0u8, 0u8, 0u8]; (width * height) as usize],
    };

    // Read chunks row by row.
    for y in 0..height {
        // Read number of chunks for this row.
        let chunk_count = reader.read_u16()? as usize;

        // If this row doesn't have any chunks, we skip 6 bytes and go to the next row.
        if chunk_count == 0 {
            reader.skip(6);
            continue;
        }

        // Offset to the start of image data for this row.
        let row_offset = reader.read_u32()? as usize;

        // Current offset to the image data.
        let mut index = data_offset + row_offset;

        for _ in 0..chunk_count {
            // Offset in pixels from the start of the row for this chunk.
            let pixel_offset = reader.read_u16()? as u32;

            // How many pixels are in this chunk.
            let pixel_count = reader.read_u16()? as u32;

            // Now we read 16 bit color data from current_offset and write
            // it to the sprite's image_data starting at pixel_offset.
            for x in 0..pixel_count {
                // Read color data and convert it from R5G6B5 to R8G8B8.
                let color = &reader.buffer()[index..index + 2];
                let color = r5g6b5_to_r8g8b8(u16::from_le_bytes(color.try_into()?));

                image.image_data[(y * width + (pixel_offset + x)) as usize] =
                    [color[0], color[1], color[2], 255];
                index += 2;
            }
        }

        // I don't know what this is, but it's always the same as chunk_count.
        let unknown = reader.read_u16()? as usize;
        assert!(chunk_count == unknown);
    }

    Ok(image)
}

fn r5g6b5_to_r8g8b8(value: u16) -> [u8; 3] {
    let r = (value & 0xf800) >> 11;
    let g = (value & 0x07e0) >> 5;
    let b = value & 0x1f;

    let r = (r * 527 + 23) >> 6;
    let g = (g * 259 + 33) >> 6;
    let b = (b * 527 + 23) >> 6;

    [r as u8, g as u8, b as u8]
}

fn r8g8b8a8_to_r5g6b5(value: &[u8; 4]) -> u16 {
    let r = ((value[0] >> 3) & 0x1F) as u16;
    let g = ((value[1] >> 2) & 0x3F) as u16;
    let b = ((value[2] >> 3) & 0x1F) as u16;

    (r << 11) | (g << 5) | b
}

fn decompress(input_buffer: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut current_block: u64;

    let size: usize = input_buffer.len();
    let mut in_ptr1 = 0;

    let window_size: u16 = 2048;
    let mut token: u32 = 0;
    let mut length: u32 = 0;
    let mut in_ptr2: usize;

    let decompressed_size: u32 = u32::from_le_bytes(input_buffer[0..4].try_into()?);

    in_ptr1 += 4;

    let mut output = vec![0u8; decompressed_size as usize];

    let mut out_ptr1: usize = 0;
    let mut out_ptr2: usize = 0;

    if input_buffer[in_ptr1] < 18 {
        current_block = 17212426794612558339;
    } else {
        token = (input_buffer[in_ptr1] - 17) as u32;
        in_ptr1 += 1;

        if token < 4 {
            current_block = 3808892272982576282;
        } else {
            loop {
                output[out_ptr2] = input_buffer[in_ptr1];
                out_ptr2 += 1;
                in_ptr1 += 1;
                token -= 1;

                if token == 0 {
                    break;
                }
            }

            current_block = 11042950489265723346
        }
    }

    loop {
        match current_block {
            11042950489265723346 => {
                token = input_buffer[in_ptr1] as u32;
                in_ptr2 = in_ptr1 + 1;
                out_ptr1 = out_ptr2;

                if token > 15 {
                    current_block = 1915145597995923502;
                } else {
                    let offset: i32 =
                        input_buffer[in_ptr2] as i32 * -4 - (input_buffer[in_ptr1] as i32 >> 2);
                    in_ptr1 += 2;
                    output[out_ptr2] = output
                        [out_ptr2.strict_add_signed((offset - (window_size + 1) as i32) as isize)];
                    out_ptr1 = out_ptr2 + 1;
                    out_ptr2 = out_ptr2.strict_add_signed((offset - window_size as i32) as isize);
                    current_block = 15713817579421313723;
                }
            }
            17212426794612558339 => {
                token = input_buffer[in_ptr1] as u32;
                in_ptr2 = in_ptr1 + 1;

                if token < 16 {
                    if token == 0 {
                        let mut offset: i32 = 0;
                        let mut current_1: u8 = input_buffer[in_ptr2];

                        while current_1 == 0 {
                            out_ptr2 = in_ptr2 + 1;
                            offset += 0xff;
                            in_ptr2 += 1;
                            current_1 = input_buffer[out_ptr2];
                        }

                        token = (offset + 0xf + input_buffer[in_ptr2] as i32) as u32;
                        in_ptr2 += 1;
                    }

                    in_ptr1 = in_ptr2 + 4;
                    output[out_ptr1..out_ptr1 + 4]
                        .copy_from_slice(&input_buffer[in_ptr2..in_ptr2 + 4]);
                    out_ptr2 = out_ptr1 + 4;
                    token -= 1;

                    if token != 0 {
                        if token < 4 {
                            loop {
                                output[out_ptr2] = input_buffer[in_ptr1];
                                out_ptr2 += 1;
                                in_ptr1 += 1;
                                token -= 1;

                                if token == 0 {
                                    break;
                                }
                            }
                        } else {
                            loop {
                                token -= 4;
                                output[out_ptr2] = input_buffer[in_ptr1];
                                output[out_ptr2 + 1] = input_buffer[in_ptr1 + 1];
                                output[out_ptr2 + 2] = input_buffer[in_ptr1 + 2];
                                output[out_ptr2 + 3] = input_buffer[in_ptr1 + 3];

                                out_ptr2 += 4;
                                in_ptr1 += 4;

                                if token <= 3 {
                                    break;
                                }
                            }

                            while token != 0 {
                                output[out_ptr2] = input_buffer[in_ptr1];
                                out_ptr2 += 1;
                                in_ptr1 += 1;
                                token -= 1;
                            }
                        }
                    }

                    current_block = 11042950489265723346;
                    continue;
                } else {
                    current_block = 1915145597995923502;
                }
            }
            _ => {
                loop {
                    in_ptr2 = in_ptr1;
                    output[out_ptr1] = input_buffer[in_ptr2];
                    out_ptr1 += 1;
                    token -= 1;
                    in_ptr1 = in_ptr2 + 1;

                    if token == 0 {
                        break;
                    }
                }

                token = input_buffer[in_ptr2 + 1] as u32;
                in_ptr2 += 2;
                current_block = 1915145597995923502;
            }
        }

        if current_block == 1915145597995923502 {
            if token > 63 {
                let tmp: i32 = input_buffer[in_ptr2] as i32 * -8 - (token >> 2 & 7) as i32;
                out_ptr2 = out_ptr1.strict_add_signed(tmp as isize) - 1;
                in_ptr1 = in_ptr2 + 1;
                length = (token >> 5) - 1;
                current_block = 9709537126203772897;
            } else {
                if token < 32 {
                    if token < 16 {
                        let tmp8: i32 = input_buffer[in_ptr2] as i32 * -4 - (token >> 2) as i32;
                        out_ptr2 = out_ptr1.strict_add_signed(tmp8 as isize) - 1;
                        in_ptr1 = in_ptr2 + 1;
                        current_block = 15713817579421313723;
                    } else {
                        length = token & 7;
                        if length == 0 {
                            let mut offset: i32 = 0;
                            let mut current: u8 = input_buffer[in_ptr2];

                            while current == 0 {
                                out_ptr2 = in_ptr2 + 1;
                                offset += 0xff;
                                in_ptr2 += 1;
                                current = input_buffer[out_ptr2];
                            }

                            length = (offset + 7 + input_buffer[in_ptr2] as i32) as u32;
                            in_ptr2 += 1;
                        }

                        in_ptr1 = in_ptr2 + 2;
                        let tmp3: i32 = (token & 8) as i32 * -(window_size as i32)
                            - (read_u16(in_ptr2, input_buffer) >> 2) as i32;

                        if out_ptr1.strict_add_signed(tmp3 as isize) == out_ptr1 {
                            if in_ptr1 == size {
                                return Ok(output);
                            } else {
                                return Err("Failed to decompress".into());
                            }
                        } else {
                            out_ptr2 = out_ptr1
                                .strict_add_signed(tmp3 as isize)
                                .strict_add_signed(-0x4000);
                        }
                        current_block = 200744462051969938;
                    }
                } else {
                    length = token & 0x1f;
                    if length == 0 {
                        let mut offset: i32 = 0;
                        let mut current: u8 = input_buffer[in_ptr2];

                        while current == 0 {
                            out_ptr2 = in_ptr2 + 1;
                            offset += 0xff;
                            in_ptr2 += 1;
                            current = input_buffer[out_ptr2];
                        }

                        length = (offset + 0x1f) as u32 + input_buffer[in_ptr2] as u32;
                        in_ptr2 += 1;
                    }

                    let tmp9: i32 = -1 - (read_u16(in_ptr2, input_buffer) >> 2) as i32;
                    out_ptr2 = out_ptr1.strict_add_signed(tmp9 as isize);
                    in_ptr1 = in_ptr2 + 2;
                    current_block = 200744462051969938;
                }

                if current_block != 15713817579421313723 {
                    if length < 6 || (out_ptr1.strict_sub(out_ptr2)) < 4 {
                        current_block = 9709537126203772897;
                    } else {
                        let mut next: usize = out_ptr2 + 4;

                        output[out_ptr1] = output[out_ptr2];
                        output[out_ptr1 + 1] = output[out_ptr2 + 1];
                        output[out_ptr1 + 2] = output[out_ptr2 + 2];
                        output[out_ptr1 + 3] = output[out_ptr2 + 3];

                        out_ptr1 += 4;
                        length -= 2;

                        loop {
                            length -= 4;
                            output[out_ptr1] = output[next];
                            output[out_ptr1 + 1] = output[next + 1];
                            output[out_ptr1 + 2] = output[next + 2];
                            output[out_ptr1 + 3] = output[next + 3];
                            out_ptr1 += 4;
                            next += 4;

                            if length <= 3 {
                                break;
                            }
                        }

                        while length != 0 {
                            output[out_ptr1] = output[next];
                            out_ptr1 += 1;
                            next += 1;
                            length -= 1;
                        }

                        current_block = 12126080054970298099;
                    }
                }
            }

            match current_block {
                12126080054970298099 => {}
                15713817579421313723 => {}
                _ => {
                    output[out_ptr1] = output[out_ptr2];
                    output[out_ptr1 + 1] = output[out_ptr2 + 1];
                    out_ptr1 += 2;
                    out_ptr2 += 2;

                    loop {
                        output[out_ptr1] = output[out_ptr2];
                        out_ptr1 += 1;
                        out_ptr2 += 1;
                        length -= 1;

                        if length == 0 {
                            break;
                        }
                    }
                    current_block = 12126080054970298099;
                }
            }
        }

        if current_block == 15713817579421313723 {
            output[out_ptr1] = output[out_ptr2];
            output[out_ptr1 + 1] = output[out_ptr2 + 1];
            out_ptr1 += 2;
        }

        token = (input_buffer[in_ptr1 - 2] & 3) as u32;

        if token == 0 {
            current_block = 17212426794612558339;
        } else {
            current_block = 3808892272982576282;
        }
    }
}

fn read_u16(ptr: usize, buf: &[u8]) -> u16 {
    u16::from_le_bytes(buf[ptr..ptr + 2].try_into().unwrap())
}

#[cfg(test)]
mod compressed {
    use std::path::PathBuf;

    use crate::types::image_list::load_image_list;

    fn compressed(index: usize) {
        let path = PathBuf::from_iter([
            env!("CARGO_MANIFEST_DIR"),
            "tmp",
            "divine_divinity",
            "static",
            "imagelists",
        ]);

        let path = std::fs::canonicalize(&path).expect("path to image lists must exist");

        let image_list = load_image_list(
            &path.join(format!("CPackedb.{index}c")),
            &path.join(format!("CPackedi.{index}c")),
            true,
        );

        if index == 7 {
            assert!(image_list.is_err());
        } else {
            assert!(image_list.is_ok());
        }
    }

    macro_rules! test {
        ($fn:ident, $index:expr) => {
            #[test]
            fn $fn() {
                compressed($index);
            }
        };
    }

    test!(i0, 0);
    test!(i1, 1);
    test!(i2, 2);
    test!(i3, 3);
    test!(i4, 4);
    test!(i5, 5);
    test!(i6, 6);
    test!(i7, 7);
    test!(i8, 8);
    test!(i9, 9);
    test!(i10, 10);
    test!(i12, 12);
}
