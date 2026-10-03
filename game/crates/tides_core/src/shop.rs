//! 铜婶's general store (design §14): supplies and materials for gold, and
//! gear bought back at the sell price. Supplies are carried in the run and
//! drunk with a hotkey; what each one does is described by `SupplyEffect`.

use serde::{Deserialize, Serialize};

use crate::forge::Material;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Supply {
    /// Closes wounds: heals a chunk of health.
    HealingDraught,
    /// Bitter grey-lamp tea: steadies the mind.
    CalmingTea,
}

/// What drinking a supply does. Health lives in the engine (it's a combat
/// value), sanity lives in the run, so the caller applies `Heal` itself.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SupplyEffect {
    Heal(f32),
    Sanity(i32),
}

impl Supply {
    pub const ALL: [Supply; 2] = [Supply::HealingDraught, Supply::CalmingTea];

    pub fn label(self) -> &'static str {
        match self {
            Supply::HealingDraught => "愈伤药水",
            Supply::CalmingTea => "灰灯宁神茶",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Supply::HealingDraught => "回复 40 点生命。按 Q 饮用。",
            Supply::CalmingTea => "理智 +8。按 R 饮用。",
        }
    }

    pub fn effect(self) -> SupplyEffect {
        match self {
            Supply::HealingDraught => SupplyEffect::Heal(40.0),
            Supply::CalmingTea => SupplyEffect::Sanity(8),
        }
    }
}

/// One line on the shop board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ware {
    Supply(Supply),
    Material(Material),
}

impl Ware {
    pub fn label(self) -> &'static str {
        match self {
            Ware::Supply(s) => s.label(),
            Ware::Material(m) => m.label(),
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Ware::Supply(s) => s.blurb(),
            Ware::Material(Material::Common) => "铁屑和皮绳，铁匠用得上。",
            Ware::Material(Material::Rare) => "锁定词条要用它。铜婶只剩这点存货。",
            Ware::Material(Material::CorruptResidue) => "铜婶不卖这个。",
        }
    }
}

/// The store's board, cheapest first. Corrupt residue is never for sale.
pub const STOCK: [(Ware, u32); 4] = [
    (Ware::Supply(Supply::HealingDraught), 30),
    (Ware::Supply(Supply::CalmingTea), 45),
    (Ware::Material(Material::Common), 20),
    (Ware::Material(Material::Rare), 120),
];

pub fn price(ware: Ware) -> Option<u32> {
    STOCK.iter().find(|(w, _)| *w == ware).map(|(_, p)| *p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stock_is_cheapest_supplies_first_and_never_sells_residue() {
        assert!(price(Ware::Material(Material::CorruptResidue)).is_none());
        assert!(STOCK.iter().all(|(_, p)| *p > 0));
        assert_eq!(STOCK[0].0, Ware::Supply(Supply::HealingDraught));
    }

    #[test]
    fn every_supply_is_stocked_and_does_something() {
        for s in Supply::ALL {
            assert!(price(Ware::Supply(s)).is_some(), "{s:?}");
            let fx = s.effect();
            assert!(matches!(fx, SupplyEffect::Heal(h) if h > 0.0) || matches!(fx, SupplyEffect::Sanity(n) if n > 0));
        }
    }
}
