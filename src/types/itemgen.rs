//! itemgen.cmp

use std::any::TypeId;

use crate::{
    editor::{Inspector, property, struct_ui},
    types::{Binary, CStringWithLength},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ItemGen {
    items: Vec<Item>,
}

#[derive(Default, serde::Serialize, serde::Deserialize)]
enum Item {
    #[default]
    Base, // 0
    Skill(Skill),           // 1
    Path(Path),             // 2
    Stats(Stats),           // 3
    Effect(Effect),         // 4
    Charm(Charm),           // 5
    Durability(Durability), // 6
    Speed(Speed),           // 7
}

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        core::mem::discriminant(self) == core::mem::discriminant(other)
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct CommonData {
    min_level: i32,
    chance: i32,
    charm_id: i32,
    gold: i32,
    prefix: CStringWithLength,
    suffix: CStringWithLength,
    use_on_armor: bool,
    use_on_weapons: bool,
    use_on_bows: bool,
    use_on_rings: bool,
    id: i32,
    min_value: i32,
    max_value: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Skill {
    common: CommonData,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Path {
    min_level: i32,
    chance: i32,
    charm_id: i32,
    gold: i32,
    prefix: CStringWithLength,
    suffix: CStringWithLength,
    use_on_armor: bool,
    use_on_weapons: bool,
    use_on_bows: bool,
    use_on_rings: bool,
    id: i32,
    max_value: i32,
    min_value: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Stats {
    common: CommonData,
    stat: Stat,
}

#[derive(serde::Serialize, serde::Deserialize, Default, PartialEq, Eq)]
#[repr(i32)]
enum Stat {
    #[default]
    Strength = 0,
    Dexterity = 1,
    Constitution = 2,
    Intelligence = 3,
    Vitality = 4,
    Mana = 5,
    Offense = 6,
    Defense = 7,
    Sight = 8,
    LightningResistance = 9,
    FireResistance = 10,
    SpiritResistance = 11,
    PoisonResistance = 12,
    Armor = 13,
    Damage = 14,
    Hearing = 15,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Effect {
    common: CommonData,
    unknown0: i32,
    effect_name: CStringWithLength,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Charm {
    min_level: i32,
    chance: i32,
    charm_id: i32,
    gold: i32,
    prefix: CStringWithLength,
    suffix: CStringWithLength,
    use_on_armor: bool,
    use_on_weapons: bool,
    use_on_bows: bool,
    use_on_rings: bool,
    id: i32,
    min_value: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Durability {
    common: CommonData,
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct Speed {
    common: CommonData,
}

impl Binary for ItemGen {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let header = reader.read_i32()?;

        if header != -2 {
            return Err("Invalid header".into());
        }

        Ok(Self {
            items: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        writer.write_i32(-2);
        self.items.to_bytes(writer);
    }
}

impl Binary for Item {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        let item_type = reader.read_u32()?;

        match item_type {
            0 => Ok(Self::Base),
            1 => Ok(Self::Skill(Binary::from_bytes(reader)?)),
            2 => Ok(Self::Path(Binary::from_bytes(reader)?)),
            3 => Ok(Self::Stats(Binary::from_bytes(reader)?)),
            4 => Ok(Self::Effect(Binary::from_bytes(reader)?)),
            5 => Ok(Self::Charm(Binary::from_bytes(reader)?)),
            6 => Ok(Self::Durability(Binary::from_bytes(reader)?)),
            7 => Ok(Self::Speed(Binary::from_bytes(reader)?)),
            unknown => Err(format!(
                "Unknown item type: {unknown} (at: {:X})",
                reader.position() - 4
            )
            .into()),
        }
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        let item_type = match self {
            Self::Base => 0,
            Self::Skill(_) => 1,
            Self::Path(_) => 2,
            Self::Stats(_) => 3,
            Self::Effect(_) => 4,
            Self::Charm(_) => 5,
            Self::Durability(_) => 6,
            Self::Speed(_) => 7,
        };

        writer.write_u32(item_type);

        match self {
            Self::Base => (),
            Self::Skill(skill) => skill.to_bytes(writer),
            Self::Path(path) => path.to_bytes(writer),
            Self::Stats(stats) => stats.to_bytes(writer),
            Self::Effect(effect) => effect.to_bytes(writer),
            Self::Charm(charm) => charm.to_bytes(writer),
            Self::Durability(durability) => durability.to_bytes(writer),
            Self::Speed(speed) => speed.to_bytes(writer),
        }
    }
}

impl Binary for CommonData {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            min_level: Binary::from_bytes(reader)?,
            chance: Binary::from_bytes(reader)?,
            charm_id: Binary::from_bytes(reader)?,
            gold: Binary::from_bytes(reader)?,
            prefix: Binary::from_bytes(reader)?,
            suffix: Binary::from_bytes(reader)?,
            use_on_armor: u32::from_bytes(reader)? != 0,
            use_on_weapons: u32::from_bytes(reader)? != 0,
            use_on_bows: u32::from_bytes(reader)? != 0,
            use_on_rings: u32::from_bytes(reader)? != 0,
            id: Binary::from_bytes(reader)?,
            min_value: Binary::from_bytes(reader)?,
            max_value: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.min_level.to_bytes(writer);
        self.chance.to_bytes(writer);
        self.charm_id.to_bytes(writer);
        self.gold.to_bytes(writer);
        self.prefix.to_bytes(writer);
        self.suffix.to_bytes(writer);
        (self.use_on_armor as u32).to_bytes(writer);
        (self.use_on_weapons as u32).to_bytes(writer);
        (self.use_on_bows as u32).to_bytes(writer);
        (self.use_on_rings as u32).to_bytes(writer);
        self.id.to_bytes(writer);
        self.min_value.to_bytes(writer);
        self.max_value.to_bytes(writer);
    }
}

impl Binary for Skill {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            common: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.common.to_bytes(writer);
    }
}

impl Binary for Path {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            min_level: Binary::from_bytes(reader)?,
            chance: Binary::from_bytes(reader)?,
            charm_id: Binary::from_bytes(reader)?,
            gold: Binary::from_bytes(reader)?,
            prefix: Binary::from_bytes(reader)?,
            suffix: Binary::from_bytes(reader)?,
            use_on_armor: u32::from_bytes(reader)? != 0,
            use_on_weapons: u32::from_bytes(reader)? != 0,
            use_on_bows: u32::from_bytes(reader)? != 0,
            use_on_rings: u32::from_bytes(reader)? != 0,
            id: Binary::from_bytes(reader)?,
            max_value: Binary::from_bytes(reader)?,
            min_value: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.min_level.to_bytes(writer);
        self.chance.to_bytes(writer);
        self.charm_id.to_bytes(writer);
        self.gold.to_bytes(writer);
        self.prefix.to_bytes(writer);
        self.suffix.to_bytes(writer);
        (self.use_on_armor as u32).to_bytes(writer);
        (self.use_on_weapons as u32).to_bytes(writer);
        (self.use_on_bows as u32).to_bytes(writer);
        (self.use_on_rings as u32).to_bytes(writer);
        self.id.to_bytes(writer);
        self.max_value.to_bytes(writer);
        self.min_value.to_bytes(writer);
    }
}

impl Binary for Stats {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            common: Binary::from_bytes(reader)?,
            stat: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.common.to_bytes(writer);
        self.stat.to_bytes(writer);
    }
}

impl Binary for Stat {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        match reader.read_i32()? {
            0 => Ok(Self::Strength),
            1 => Ok(Self::Dexterity),
            2 => Ok(Self::Constitution),
            3 => Ok(Self::Intelligence),
            4 => Ok(Self::Vitality),
            5 => Ok(Self::Mana),
            6 => Ok(Self::Offense),
            7 => Ok(Self::Defense),
            8 => Ok(Self::Sight),
            9 => Ok(Self::LightningResistance),
            10 => Ok(Self::FireResistance),
            11 => Ok(Self::SpiritResistance),
            12 => Ok(Self::PoisonResistance),
            13 => Ok(Self::Armor),
            14 => Ok(Self::Damage),
            15 => Ok(Self::Hearing),
            unknown => Err(format!("Unknown stat: {unknown}").into()),
        }
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        let value = match self {
            Stat::Strength => Self::Strength as i32,
            Stat::Dexterity => Self::Dexterity as i32,
            Stat::Constitution => Self::Constitution as i32,
            Stat::Intelligence => Self::Intelligence as i32,
            Stat::Vitality => Self::Vitality as i32,
            Stat::Mana => Self::Mana as i32,
            Stat::Offense => Self::Offense as i32,
            Stat::Defense => Self::Defense as i32,
            Stat::Sight => Self::Sight as i32,
            Stat::LightningResistance => Self::LightningResistance as i32,
            Stat::FireResistance => Self::FireResistance as i32,
            Stat::SpiritResistance => Self::SpiritResistance as i32,
            Stat::PoisonResistance => Self::PoisonResistance as i32,
            Stat::Armor => Self::Armor as i32,
            Stat::Damage => Self::Damage as i32,
            Stat::Hearing => Self::Hearing as i32,
        };

        writer.write_i32(value);
    }
}

impl Binary for Effect {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            common: Binary::from_bytes(reader)?,
            unknown0: Binary::from_bytes(reader)?,
            effect_name: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.common.to_bytes(writer);
        self.unknown0.to_bytes(writer);
        self.effect_name.to_bytes(writer);
    }
}

impl Binary for Charm {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            min_level: Binary::from_bytes(reader)?,
            chance: Binary::from_bytes(reader)?,
            charm_id: Binary::from_bytes(reader)?,
            gold: Binary::from_bytes(reader)?,
            prefix: Binary::from_bytes(reader)?,
            suffix: Binary::from_bytes(reader)?,
            use_on_armor: u32::from_bytes(reader)? != 0,
            use_on_weapons: u32::from_bytes(reader)? != 0,
            use_on_bows: u32::from_bytes(reader)? != 0,
            use_on_rings: u32::from_bytes(reader)? != 0,
            id: Binary::from_bytes(reader)?,
            min_value: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.min_level.to_bytes(writer);
        self.chance.to_bytes(writer);
        self.charm_id.to_bytes(writer);
        self.gold.to_bytes(writer);
        self.prefix.to_bytes(writer);
        self.suffix.to_bytes(writer);
        (self.use_on_armor as u32).to_bytes(writer);
        (self.use_on_weapons as u32).to_bytes(writer);
        (self.use_on_bows as u32).to_bytes(writer);
        (self.use_on_rings as u32).to_bytes(writer);
        self.id.to_bytes(writer);
        self.min_value.to_bytes(writer);
    }
}

impl Binary for Durability {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            common: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.common.to_bytes(writer);
    }
}

impl Binary for Speed {
    fn from_bytes(reader: &mut crate::buffer::BufferReader) -> crate::Result<Self>
    where
        Self: Sized,
    {
        Ok(Self {
            common: Binary::from_bytes(reader)?,
        })
    }

    fn to_bytes(&self, writer: &mut crate::buffer::BufferWriter) {
        self.common.to_bytes(writer);
    }
}

impl Inspector for ItemGen {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            property("items", &mut self.items, ui);
        });
    }
}

impl Inspector for CommonData {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        property("min_level", &mut self.min_level, ui);
        property("chance", &mut self.chance, ui);
        property("charm_id", &mut self.charm_id, ui);
        property("gold", &mut self.gold, ui);
        property("prefix", &mut self.prefix, ui);
        property("suffix", &mut self.suffix, ui);
        property("use_on_armor", &mut self.use_on_armor, ui);
        property("use_on_weapons", &mut self.use_on_weapons, ui);
        property("use_on_bows", &mut self.use_on_bows, ui);
        property("use_on_rings", &mut self.use_on_rings, ui);
        property("id", &mut self.id, ui);
        property("min_value", &mut self.min_value, ui);
        property("max_value", &mut self.max_value, ui);
    }
}

impl Inspector for Item {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        struct_ui(ui, |ui| {
            ui.label("Item type");
            ui.push_id("item_type", |ui| {
                let selected_text = match self {
                    Self::Base => "Base",
                    Self::Skill(_) => "Skill",
                    Self::Path(_) => "Path",
                    Self::Stats(_) => "Stats",
                    Self::Effect(_) => "Effect",
                    Self::Charm(_) => "Charm",
                    Self::Durability(_) => "Durability",
                    Self::Speed(_) => "Speed",
                };

                eframe::egui::ComboBox::from_id_salt(TypeId::of::<Self>())
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(self, Self::Base, "Base");
                        ui.selectable_value(self, Self::Skill(Skill::default()), "Skill");
                        ui.selectable_value(self, Self::Path(Path::default()), "Path");
                        ui.selectable_value(self, Self::Stats(Stats::default()), "Stats");
                        ui.selectable_value(self, Self::Effect(Effect::default()), "Effect");
                        ui.selectable_value(self, Self::Charm(Charm::default()), "Charm");
                        ui.selectable_value(
                            self,
                            Self::Durability(Durability::default()),
                            "Durability",
                        );
                        ui.selectable_value(self, Self::Speed(Speed::default()), "Speed");
                    });
            });
            ui.end_row();

            match self {
                Self::Base => {
                    ui.label("Base item is not editable.\nThe game won't load it.");
                }
                Self::Skill(skill) => skill.show(ui),
                Self::Path(path) => path.show(ui),
                Self::Stats(stats) => stats.show(ui),
                Self::Effect(effect) => effect.show(ui),
                Self::Charm(charm) => charm.show(ui),
                Self::Durability(durability) => durability.show(ui),
                Self::Speed(speed) => speed.show(ui),
            }
        });
    }
}

impl Inspector for Skill {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.common.show(ui);
    }
}

impl Inspector for Path {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        property("min_level", &mut self.min_level, ui);
        property("chance", &mut self.chance, ui);
        property("charm_id", &mut self.charm_id, ui);
        property("gold", &mut self.gold, ui);
        property("prefix", &mut self.prefix, ui);
        property("suffix", &mut self.suffix, ui);
        property("use_on_armor", &mut self.use_on_armor, ui);
        property("use_on_weapons", &mut self.use_on_weapons, ui);
        property("use_on_bows", &mut self.use_on_bows, ui);
        property("use_on_rings", &mut self.use_on_rings, ui);
        property("id", &mut self.id, ui);
        property("max_value", &mut self.max_value, ui);
        property("min_value", &mut self.min_value, ui);
    }
}

impl Inspector for Stats {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.common.show(ui);
        property("stat", &mut self.stat, ui);
    }
}

impl Inspector for Stat {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        let selected_text = match self {
            Stat::Strength => "Strength",
            Stat::Dexterity => "Dexterity",
            Stat::Constitution => "Constitution",
            Stat::Intelligence => "Intelligence",
            Stat::Vitality => "Vitality",
            Stat::Mana => "Mana",
            Stat::Offense => "Offense",
            Stat::Defense => "Defense",
            Stat::Sight => "Sight",
            Stat::LightningResistance => "LightningResistance",
            Stat::FireResistance => "FireResistance",
            Stat::SpiritResistance => "SpiritResistance",
            Stat::PoisonResistance => "PoisonResistance",
            Stat::Armor => "Armor",
            Stat::Damage => "Damage",
            Stat::Hearing => "Hearing",
        };

        eframe::egui::ComboBox::from_id_salt(TypeId::of::<Self>())
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.selectable_value(self, Stat::Strength, "Strength");
                ui.selectable_value(self, Stat::Dexterity, "Dexterity");
                ui.selectable_value(self, Stat::Constitution, "Constitution");
                ui.selectable_value(self, Stat::Intelligence, "Intelligence");
                ui.selectable_value(self, Stat::Vitality, "Vitality");
                ui.selectable_value(self, Stat::Mana, "Mana");
                ui.selectable_value(self, Stat::Offense, "Offense");
                ui.selectable_value(self, Stat::Defense, "Defense");
                ui.selectable_value(self, Stat::Sight, "Sight");
                ui.selectable_value(self, Stat::LightningResistance, "LightningResistance");
                ui.selectable_value(self, Stat::FireResistance, "FireResistance");
                ui.selectable_value(self, Stat::SpiritResistance, "SpiritResistance");
                ui.selectable_value(self, Stat::PoisonResistance, "PoisonResistance");
                ui.selectable_value(self, Stat::Armor, "Armor");
                ui.selectable_value(self, Stat::Damage, "Damage");
                ui.selectable_value(self, Stat::Hearing, "Hearing");
            });
    }
}

impl Inspector for Effect {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.common.show(ui);
        property("unknown0", &mut self.unknown0, ui);
        property("effect_name", &mut self.effect_name, ui);
    }
}

impl Inspector for Charm {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        property("min_level", &mut self.min_level, ui);
        property("chance", &mut self.chance, ui);
        property("charm_id", &mut self.charm_id, ui);
        property("gold", &mut self.gold, ui);
        property("prefix", &mut self.prefix, ui);
        property("suffix", &mut self.suffix, ui);
        property("use_on_armor", &mut self.use_on_armor, ui);
        property("use_on_weapons", &mut self.use_on_weapons, ui);
        property("use_on_bows", &mut self.use_on_bows, ui);
        property("use_on_rings", &mut self.use_on_rings, ui);
        property("id", &mut self.id, ui);
        property("min_value", &mut self.min_value, ui);
    }
}

impl Inspector for Durability {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.common.show(ui);
    }
}

impl Inspector for Speed {
    fn show(&mut self, ui: &mut eframe::egui::Ui) {
        self.common.show(ui);
    }
}
