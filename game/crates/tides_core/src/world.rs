//! Persistent world state and the typed effects that change it.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::mind::Meter;

/// Story flags. Typed so a typo is a compile error, not a silent no-op.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Flag {
    AwakenedByKhah,
    MetInjuredDwarf,
    TouchedTotemFragment,
    DefeatedGrom,
    EscapedMine,
}

/// The first permanent choice (design §5): what happened to the dwarf.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DwarfChoice {
    Save,
    Abandon,
    Kill,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldState {
    pub world_tier: u8,
    pub sanity: u8,
    pub corruption: u8,
    pub vessel_awakening: u8,
    pub parasite_load: u8,
    pub gold: u32,
    pub flags: BTreeSet<Flag>,
    pub dwarf_choice: Option<DwarfChoice>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            world_tier: 1,
            sanity: 78,
            corruption: 5,
            vessel_awakening: 0,
            parasite_load: 0,
            gold: 0,
            flags: BTreeSet::new(),
            dwarf_choice: None,
        }
    }
}

impl WorldState {
    pub fn has(&self, flag: Flag) -> bool {
        self.flags.contains(&flag)
    }

    pub fn meter(&self, meter: Meter) -> u8 {
        match meter {
            Meter::Sanity => self.sanity,
            Meter::Corruption => self.corruption,
            Meter::ParasiteLoad => self.parasite_load,
        }
    }

    pub fn meter_mut(&mut self, meter: Meter) -> &mut u8 {
        match meter {
            Meter::Sanity => &mut self.sanity,
            Meter::Corruption => &mut self.corruption,
            Meter::ParasiteLoad => &mut self.parasite_load,
        }
    }
}

/// One change to the world. Meters are always *relative* so an earlier
/// event can never be silently undone (the GDScript bug this replaces).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    SetFlag(Flag),
    DwarfChoice(DwarfChoice),
    /// Vessel awakening only ever rises.
    RaiseVessel(u8),
    Meter(Meter, i32),
}
