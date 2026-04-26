use std::{
    any::Any,
    ffi::{CStr, CString},
    path::PathBuf,
};

use eframe::egui;

use crate::{
    buffer::{BufferReader, BufferWriter},
    editor::Inspector,
    types::{
        eggs::Eggs, font::Font, info::Info, itemgen::ItemGen, magic::Magic, music::Music,
        objects::Objects, objects_000::Objects000, osiris_names::OsirisNames,
        osiris_objects::OsirisObjects, persist::Persist, props::Props, quest_log::QuestLog,
        quickinfo::QuickInfo, reverbs::Reverbs, shroud::Shroud, sound::SoundConfig,
        status_plate::StatusPlate, telpstates::TelpStates, text::Text, usernotes::Notes,
        world::World,
    },
};

pub mod data;
pub mod eggs;
pub mod font;
pub mod image_list;
pub mod info;
pub mod itemgen;
pub mod magic;
pub mod music;
pub mod objects;
pub mod objects_000;
pub mod osiris_names;
pub mod osiris_objects;
pub mod packed;
pub mod persist;
pub mod props;
pub mod quest_log;
pub mod quickinfo;
pub mod reverbs;
pub mod shroud;
pub mod sound;
pub mod status_plate;
pub mod telpstates;
pub mod text;
pub mod usernotes;
pub mod world;

type FromBytesFn = fn(&mut BufferReader) -> crate::Result<Box<dyn Binary>>;

pub trait Binary: erased_serde::Serialize + Inspector + Any {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized;

    fn to_bytes(&self, writer: &mut BufferWriter);
}

erased_serde::serialize_trait_object!(Binary);

pub fn from_bytes_dyn<T: Binary>(reader: &mut BufferReader) -> crate::Result<Box<dyn Binary>> {
    T::from_bytes(reader).map(|v| Box::new(v) as Box<dyn Binary>)
}

#[derive(serde::Serialize)]
pub struct Format {
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(skip)]
    pub file_name: Option<PathBuf>,
    format_type: FormatType,
    pub binary: Box<dyn Binary>,
}

impl<'de> serde::Deserialize<'de> for Format {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FormatVisitor;

        impl<'de> serde::de::Visitor<'de> for FormatVisitor {
            type Value = Format;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                if map.next_key::<&str>()? != Some("format_type") {
                    return Err(serde::de::Error::missing_field("format_type"));
                }

                let format_type = map.next_value::<FormatType>()?;

                if map.next_key::<&str>()? != Some("binary") {
                    return Err(serde::de::Error::missing_field("binary"));
                }

                let binary: Box<dyn Binary> = match format_type {
                    FormatType::Eggs => Box::new(map.next_value::<Eggs>()?),
                    FormatType::TelpStates => Box::new(map.next_value::<TelpStates>()?),
                    FormatType::Font => Box::new(map.next_value::<Font>()?),
                    FormatType::Magic => Box::new(map.next_value::<Magic>()?),
                    FormatType::Music => Box::new(map.next_value::<Music>()?),
                    FormatType::SoundConfig => Box::new(map.next_value::<SoundConfig>()?),
                    FormatType::Props => Box::new(map.next_value::<Props>()?),
                    FormatType::StatusPlate => Box::new(map.next_value::<StatusPlate>()?),
                    FormatType::Notes => Box::new(map.next_value::<Notes>()?),
                    FormatType::Reverbs => Box::new(map.next_value::<Reverbs>()?),
                    FormatType::OsirisObjects => Box::new(map.next_value::<OsirisObjects>()?),
                    FormatType::OsirisNames => Box::new(map.next_value::<OsirisNames>()?),
                    FormatType::Persist => Box::new(map.next_value::<Persist>()?),
                    FormatType::Objects000 => Box::new(map.next_value::<Objects000>()?),
                    FormatType::QuestLog => Box::new(map.next_value::<QuestLog>()?),
                    FormatType::QuickInfo => Box::new(map.next_value::<QuickInfo>()?),
                    FormatType::Text => Box::new(map.next_value::<Text>()?),
                    FormatType::Info => Box::new(map.next_value::<Info>()?),
                    FormatType::Shroud => Box::new(map.next_value::<Shroud>()?),
                    FormatType::World => Box::new(map.next_value::<World>()?),
                    FormatType::Objects => Box::new(map.next_value::<Objects>()?),
                    FormatType::ItemGen => Box::new(map.next_value::<ItemGen>()?),
                };

                Ok(Format {
                    path: None,
                    file_name: None,
                    format_type,
                    binary,
                })
            }
        }

        deserializer.deserialize_map(FormatVisitor)
    }
}

impl Format {
    pub(crate) fn from_file(path: &PathBuf) -> crate::Result<Self> {
        let file_name = path
            .file_name()
            .expect("must have file name")
            .to_string_lossy()
            .to_string();

        let (format_type, load): (FormatType, FromBytesFn) = match file_name.as_str() {
            "music.dat" => (FormatType::Music, from_bytes_dyn::<Music>),
            "sound.cfg" => (FormatType::SoundConfig, from_bytes_dyn::<SoundConfig>),
            "props.000" => (FormatType::Props, from_bytes_dyn::<Props>),
            "magic.cmp" => (FormatType::Magic, from_bytes_dyn::<Magic>),
            "statuspl.cmp" => (FormatType::StatusPlate, from_bytes_dyn::<StatusPlate>),
            "usernotes.bin" | "mapflags.000" => (FormatType::Notes, from_bytes_dyn::<Notes>),
            "eggs.000" => (FormatType::Eggs, from_bytes_dyn::<Eggs>),
            "reverbs.dat" => (FormatType::Reverbs, from_bytes_dyn::<Reverbs>),
            "osiobjects.000" => (FormatType::OsirisObjects, from_bytes_dyn::<OsirisObjects>),
            "osinames.000" => (FormatType::OsirisNames, from_bytes_dyn::<OsirisNames>),
            "persist.dat" => (FormatType::Persist, from_bytes_dyn::<Persist>),
            "objects.000" => (FormatType::Objects000, from_bytes_dyn::<Objects000>),
            "telpstates.000" => (FormatType::TelpStates, from_bytes_dyn::<TelpStates>),
            "quest_log.000" => (FormatType::QuestLog, from_bytes_dyn::<QuestLog>),
            "quickinfo.000" => (FormatType::QuickInfo, from_bytes_dyn::<QuickInfo>),
            "text.cmp" => (FormatType::Text, from_bytes_dyn::<Text>),
            "info.000" => (FormatType::Info, from_bytes_dyn::<Info>),
            "itemgen.cmp" => (FormatType::ItemGen, from_bytes_dyn::<ItemGen>),
            _ => {
                let Some(extension) = path.extension() else {
                    return Err("Unknown file format".into());
                };

                if extension == "fnt" {
                    (FormatType::Font, from_bytes_dyn::<Font>)
                } else if let Some(stem) = path.file_stem() {
                    if stem == "shroud" {
                        (FormatType::Shroud, from_bytes_dyn::<Shroud>)
                    } else if stem == "world" {
                        (FormatType::World, from_bytes_dyn::<World>)
                    } else if stem == "objects" {
                        (FormatType::Objects, from_bytes_dyn::<Objects>)
                    } else {
                        return Err("Unknown file format".into());
                    }
                } else {
                    return Err("Unknown file format".into());
                }
            }
        };

        let file = std::fs::read(path)?;
        let mut reader = BufferReader::new(&file);
        let binary = load(&mut reader)?;

        if !reader.is_empty() {
            return Err(format!("buffer is not empty: {}", reader.position()).into());
        }

        Ok(Self {
            path: Some(path.to_owned()),
            file_name: Some(PathBuf::from(file_name)),
            format_type,
            binary,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
enum FormatType {
    Music,
    SoundConfig,
    Props,
    Magic,
    StatusPlate,
    Notes,
    Eggs,
    Reverbs,
    OsirisObjects,
    OsirisNames,
    Persist,
    Objects000,
    TelpStates,
    Font,
    QuestLog,
    QuickInfo,
    Text,
    Info,
    Shroud,
    World,
    Objects,
    ItemGen,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct FixedArray<T: Binary, const N: usize> {
    elements: Vec<T>,
}

/// Null-terminated CString with a fixed length.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct FixedCString<const N: usize> {
    inner: CString,
}

impl<const N: usize> FixedCString<N> {
    fn new(bytes: &[u8; N]) -> crate::Result<Self> {
        Ok(Self {
            inner: CStr::from_bytes_until_nul(bytes)?.to_owned(),
        })
    }
}

impl<const N: usize> Binary for FixedCString<N> {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Self::new(&reader.read_bytes(N)?.try_into()?)
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        let bytes = self.inner.as_bytes_with_nul();
        writer.write_bytes(bytes);
        writer.pad(N - bytes.len());
    }
}

impl<const N: usize> Inspector for FixedCString<N> {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.label(self.inner.to_str().unwrap());
    }
}

impl<T: Binary + Default + serde::Serialize> Binary for Vec<T> {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let len = reader.read_u32()? as usize;
        let mut array = Vec::new();

        for _ in 0..len {
            array.push(T::from_bytes(reader)?);
        }

        Ok(array)
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_u32(self.len() as u32);

        for element in self {
            element.to_bytes(writer);
        }
    }
}

impl<T: Binary + serde::Serialize + Default, const N: usize> Binary for FixedArray<T, N> {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let mut elements = Vec::with_capacity(N);

        for _ in 0..N {
            elements.push(T::from_bytes(reader)?);
        }

        Ok(Self { elements })
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        for element in &self.elements {
            element.to_bytes(writer);
        }
    }
}

impl<T: Binary + Default, const N: usize> Inspector for FixedArray<T, N> {
    fn show(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(format!("Fixed array ({N})"))
            .show_background(true)
            .show(ui, |ui| {
                egui::Grid::new(ui.id())
                    .num_columns(1)
                    .spacing([40.0, 8.0])
                    .show(ui, |ui| {
                        for (i, element) in self.elements.iter_mut().enumerate() {
                            ui.push_id(i, |ui| {
                                egui::collapsing_header::CollapsingState::load_with_default_open(
                                    ui.ctx(),
                                    ui.id(),
                                    true,
                                )
                                .show_header(ui, |ui| {
                                    ui.label(format!("{i}"));
                                })
                                .body_unindented(|ui| {
                                    element.show(ui);
                                });
                            });
                            ui.end_row();
                        }
                    });
            });
    }
}

impl Binary for u32 {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_u32()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_u32(*self);
    }
}

impl Binary for String {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_string()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_string(self);
    }
}

impl Binary for f32 {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_f32()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_f32(*self);
    }
}

impl Binary for u8 {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_u8()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_u8(*self);
    }
}

impl Binary for i16 {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_i16()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_i16(*self);
    }
}

impl Binary for u16 {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_u16()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_u16(*self);
    }
}

/// Null-terminated string with a length.
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct CStringWithLength {
    inner: CString,
}

impl Binary for CStringWithLength {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let length = u32::from_bytes(reader)? as usize;

        let bytes = reader.read_bytes(length)?.to_vec();

        Ok(Self {
            inner: if length > 0 {
                CString::from_vec_with_nul(bytes)?
            } else {
                CString::default()
            },
        })
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        if self.inner.count_bytes() > 0 {
            let bytes = self.inner.as_bytes_with_nul();
            writer.write_u32(bytes.len() as u32);
            writer.write_bytes(bytes);
        } else {
            writer.write_u32(0);
        }
    }
}

impl Inspector for CStringWithLength {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut s = self.inner.to_string_lossy().to_string();
        ui.text_edit_multiline(&mut s);
        self.inner = CString::new(s.into_bytes()).unwrap();
    }
}

impl Binary for bool {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_bool()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_bool(*self);
    }
}

impl Binary for i32 {
    fn from_bytes(reader: &mut BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        reader.read_i32()
    }

    fn to_bytes(&self, writer: &mut BufferWriter) {
        writer.write_i32(*self);
    }
}

/// Tests loading and saving binary files. When saving a file without making any changes,
/// it should produce the exact same binary as the input file.
///
/// Assumes that the game directory is located at "tmp/divine_diviniy/"
/// relative to the project root directory (can be a symlink).
///
/// Note: Objects000 and OsirisNames are not tested because original files contain
/// garbage in fixed strings (after the null terminator) that gets cleaned after saving.
#[cfg(test)]
mod load_save {
    use std::path::PathBuf;

    use crate::{
        buffer::{BufferReader, BufferWriter},
        types::{
            Binary, eggs::Eggs, image_list::ImageListIndex, info::Info, itemgen::ItemGen,
            magic::Magic, music::Music, objects::Objects, osiris_objects::OsirisObjects,
            persist::Persist, props::Props, quest_log::QuestLog, quickinfo::QuickInfo,
            reverbs::Reverbs, shroud::Shroud, sound::SoundConfig, status_plate::StatusPlate,
            telpstates::TelpStates, text::Text, usernotes::Notes, world::World,
        },
    };

    fn load_save<T: Binary>(path: PathBuf) {
        let dd_path = PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "tmp", "divine_divinity"]);
        std::fs::canonicalize(&dd_path).expect("path to Divine Divinity must exist");
        let path = dd_path.join(path);

        let input_file = std::fs::read(&path).expect("must be able to read the file");

        let mut reader = BufferReader::new(&input_file);
        let binary = T::from_bytes(&mut reader).expect("must be able to read binary file");

        let mut writer = BufferWriter::new();
        binary.to_bytes(&mut writer);

        let output_file = writer.finish();

        if input_file != output_file {
            panic!(
                "input file at path \"{}\" differs from the output file",
                path.display()
            );
        }
    }

    macro_rules! test {
        ($ty:ty, $fn:ident, $path:expr) => {
            #[test]
            fn $fn() {
                load_save::<$ty>(PathBuf::from_iter($path.split("/")));
            }
        };
    }

    test!(World, world_x0, "main/startup/world.x0");
    test!(World, world_x1, "main/startup/world.x1");
    test!(World, world_x2, "main/startup/world.x2");
    test!(World, world_x3, "main/startup/world.x3");
    test!(World, world_x4, "main/startup/world.x4");
    test!(Objects, objects_x0, "main/startup/objects.x0");
    test!(Objects, objects_x1, "main/startup/objects.x1");
    test!(Objects, objects_x2, "main/startup/objects.x2");
    test!(Objects, objects_x3, "main/startup/objects.x3");
    test!(Objects, objects_x4, "main/startup/objects.x4");
    test!(Shroud, shroud_x0, "main/startup/shroud.x0");
    test!(Shroud, shroud_x1, "main/startup/shroud.x1");
    test!(Shroud, shroud_x2, "main/startup/shroud.x2");
    test!(Shroud, shroud_x3, "main/startup/shroud.x3");
    test!(Shroud, shroud_x4, "main/startup/shroud.x4");
    test!(TelpStates, telpstates, "main/startup/telpstates.000");
    test!(Info, info, "main/startup/info.000");
    test!(QuickInfo, quickinfo, "main/startup/quickinfo.000");
    test!(QuestLog, quest_log, "main/startup/quest_log.000");
    test!(
        OsirisObjects,
        osiobjects,
        "main/startup/static/osiobjects.000"
    );
    test!(Magic, magic, "dat/magic.cmp");
    test!(Notes, usernotes, "dat/usernotes.bin");
    test!(Props, props, "dat/props.000");
    test!(Persist, persist, "persist.dat");
    test!(Music, music, "sound/music.dat");
    test!(Reverbs, reverbs, "sound/reverbs.dat");
    test!(SoundConfig, sound, "sound.cfg");
    test!(Text, text, "localizations/english/text.cmp");
    test!(StatusPlate, statuspl, "dat/statuspl.cmp");
    test!(Eggs, eggs, "global/eggs.000");
    test!(ImageListIndex, cpackedi_0c, "static/imagelists/CPackedi.0c");
    test!(ImageListIndex, cpackedi_1c, "static/imagelists/CPackedi.1c");
    test!(ImageListIndex, cpackedi_2c, "static/imagelists/CPackedi.2c");
    test!(ImageListIndex, cpackedi_3c, "static/imagelists/CPackedi.3c");
    test!(ImageListIndex, cpackedi_4c, "static/imagelists/CPackedi.4c");
    test!(ImageListIndex, cpackedi_5c, "static/imagelists/CPackedi.5c");
    test!(ImageListIndex, cpackedi_6c, "static/imagelists/CPackedi.6c");
    // TODO: implement image type 9
    // test!(ImageListIndex, "static/imagelists/CPackedi.7c");
    test!(ImageListIndex, cpackedi_8c, "static/imagelists/CPackedi.8c");
    test!(ImageListIndex, cpackedi_9c, "static/imagelists/CPackedi.9c");
    test!(
        ImageListIndex,
        cpackedi_10c,
        "static/imagelists/CPackedi.10c"
    );
    test!(
        ImageListIndex,
        cpackedi_11c,
        "static/imagelists/CPackedi.11c"
    );
    test!(
        ImageListIndex,
        cpackedi_12c,
        "static/imagelists/CPackedi.12c"
    );
    // TODO: implement
    // test!(Font, "fonts/dialog_white.fnt");

    test!(ItemGen, itemgen, "dat/English/itemgen.cmp");
}
