//! data.000
//!
//! WIP.

use crate::types::{
    eggs::Eggs, magic::Magic, osiris_names::OsirisNames, osiris_objects::OsirisObjects,
};

#[expect(unused)]
pub struct Data {
    global_vars: GlobalVars,
    // Alignment
    // Agent variables
    // Agent classes
    // Agents
    eggs: Eggs,
    // MonsterGen
    // Party
    // Skills
    time: Time,
    game_clock: GameClock,
    // Traps
    // Timers
    // Counters
    // Explostions
    // DoorChestList
    // DialogLog
    // NoMagicZones
    magic: Magic,
    // Projectiles
    // Painpoints
    // AnimationEffects
    osiris_objects: OsirisObjects,
    osiris_names: OsirisNames,
}

#[expect(unused)]
struct GlobalVars {
    unknown0: u32,
    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
    variables: Vec<u32>,
}

#[expect(unused)]
struct Time {
    unknown0: u32,
    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
    unknown4: u32,
    unknown5: u32,
}

#[expect(unused)]
struct GameClock {
    unknown0: u32,
    unknown1: u32,
    unknown2: u32,
    unknown3: u32,
    unknown4: u32,
    unknown5: u32,
    unknown6: u32,
    unknown7: u32,
    unknown8: u32,
}
