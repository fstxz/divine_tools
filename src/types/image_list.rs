//! Image lists.

use std::path::Path;

use crate::{buffer::BufferReader, editor::Inspector, types::Binary};

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

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct IndexEntry {
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

#[derive(serde::Serialize, serde::Deserialize, Default, Clone, Copy)]
enum ImageType {
    #[default]
    Opaque = 0,
    Transparent = 1,
}

pub struct Image {
    width: u32,
    height: u32,
    image_data: Vec<[u8; 4]>,
}

impl Image {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
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

fn decode_opaque_image(bytes: &[u8], width: u32, height: u32) -> crate::Result<Image> {
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

fn decode_transparent_image(bytes: &[u8]) -> crate::Result<Image> {
    let mut reader = BufferReader::new(bytes);
    let _size = reader.read_u32()?;
    let data_offset = reader.read_u32()? as usize;

    let width = reader.read_u16()? as u32;
    let height = reader.read_u16()? as u32;

    let mut image = Image {
        width,
        height,
        image_data: vec![[0u8, 0u8, 0u8, 0u8]; (width * height) as usize],
    };

    for y in 0..height {
        let chunk_count = reader.read_u16()? as usize;

        if chunk_count == 0 {
            reader.skip(6);
            continue;
        }

        let row_offset = reader.read_u32()? as usize;

        let mut index = data_offset + row_offset;

        for _ in 0..chunk_count {
            let pixel_offset = reader.read_u16()? as u32;
            let pixel_count = reader.read_u16()? as u32;

            for x in 0..pixel_count {
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
