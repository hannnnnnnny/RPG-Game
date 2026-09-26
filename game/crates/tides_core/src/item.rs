//! Equipment data: slots, qualities, affixes and item instances (design §13).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Slot {
    MainHand,
    OffHand,
    Head,
    Chest,
    Hands,
    Boots,
    Amulet,
    Ring,
    Totem,
}

impl Slot {
    /// Slots that currently drop as loot (off-hand/head/amulet come later).
    pub const DROPPABLE: [Slot; 6] = [
        Slot::MainHand,
        Slot::Chest,
        Slot::Hands,
        Slot::Boots,
        Slot::Ring,
        Slot::Totem,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Slot::MainHand => "主武器",
            Slot::OffHand => "副手",
            Slot::Head => "头部",
            Slot::Chest => "胸甲",
            Slot::Hands => "手套",
            Slot::Boots => "靴子",
            Slot::Amulet => "项链",
            Slot::Ring => "戒指",
            Slot::Totem => "图腾遗物",
        }
    }
}

/// Ordered worst → best, so `quality >= Quality::Rare` reads naturally.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Quality {
    Broken,
    Common,
    Rare,
    Corrupted,
    Relic,
    Mythic,
}

impl Quality {
    pub fn label(self) -> &'static str {
        match self {
            Quality::Broken => "破损",
            Quality::Common => "普通",
            Quality::Rare => "稀有",
            Quality::Corrupted => "污染",
            Quality::Relic => "遗物",
            Quality::Mythic => "神话",
        }
    }

    /// Name prefix so repeated base items stay distinguishable.
    pub fn name_prefix(self) -> &'static str {
        match self {
            Quality::Broken => "残破·",
            Quality::Common => "",
            Quality::Rare => "精制·",
            Quality::Corrupted => "黑潮·",
            Quality::Relic => "【遗世】",
            Quality::Mythic => "【神话】",
        }
    }

    pub fn affix_count(self) -> usize {
        match self {
            Quality::Broken => 1,
            Quality::Common => 2,
            Quality::Rare => 3,
            _ => 4,
        }
    }
}

/// Build families an affix leans toward; used for flavour names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AffixCategory {
    Attack,
    Defense,
    Mobility,
    Forbidden,
    Vessel,
    Economy,
}

impl AffixCategory {
    pub fn flavor(self) -> &'static str {
        match self {
            AffixCategory::Attack => "嗜血",
            AffixCategory::Defense => "坚岩",
            AffixCategory::Mobility => "疾风",
            AffixCategory::Forbidden => "低语",
            AffixCategory::Vessel => "容器",
            AffixCategory::Economy => "贪婪",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AffixKind {
    MeleeDamage,
    CritRate,
    InfectedDamage,
    MaxHealth,
    TideResist,
    PostRollDamage,
    StaminaRegen,
    ForbiddenSpell,
    SanityGuard,
    GoldFind,
}

impl AffixKind {
    pub const ALL: [AffixKind; 10] = [
        AffixKind::MeleeDamage,
        AffixKind::CritRate,
        AffixKind::InfectedDamage,
        AffixKind::MaxHealth,
        AffixKind::TideResist,
        AffixKind::PostRollDamage,
        AffixKind::StaminaRegen,
        AffixKind::ForbiddenSpell,
        AffixKind::SanityGuard,
        AffixKind::GoldFind,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AffixKind::MeleeDamage => "近战伤害",
            AffixKind::CritRate => "暴击率",
            AffixKind::InfectedDamage => "对感染者伤害",
            AffixKind::MaxHealth => "最大生命",
            AffixKind::TideResist => "黑潮抗性",
            AffixKind::PostRollDamage => "翻滚后伤害",
            AffixKind::StaminaRegen => "体力回复",
            AffixKind::ForbiddenSpell => "禁忌法术伤害",
            AffixKind::SanityGuard => "理智稳定",
            AffixKind::GoldFind => "金币掉落",
        }
    }

    pub fn category(self) -> AffixCategory {
        match self {
            AffixKind::MeleeDamage | AffixKind::CritRate | AffixKind::InfectedDamage => {
                AffixCategory::Attack
            }
            AffixKind::MaxHealth | AffixKind::TideResist => AffixCategory::Defense,
            AffixKind::PostRollDamage | AffixKind::StaminaRegen => AffixCategory::Mobility,
            AffixKind::ForbiddenSpell => AffixCategory::Forbidden,
            AffixKind::SanityGuard => AffixCategory::Vessel,
            AffixKind::GoldFind => AffixCategory::Economy,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct AffixId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ItemId(pub u64);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Affix {
    pub id: AffixId,
    pub kind: AffixKind,
    pub value: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub slot: Slot,
    pub quality: Quality,
    pub item_power: i32,
    pub upgrade_level: u8,
    pub reroll_count: u32,
    pub affixes: Vec<Affix>,
    pub locked_affix: Option<AffixId>,
    /// Fixed special effect; never rerolled (design §13.5).
    pub core_effect: Option<String>,
}

impl Item {
    /// "黑潮·灰灯短剑 +3" — the upgrade level is shown wherever an item is named.
    pub fn title(&self) -> String {
        if self.upgrade_level > 0 {
            format!("{} +{}", self.name, self.upgrade_level)
        } else {
            self.name.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_orders_worst_to_best() {
        assert!(Quality::Broken < Quality::Common);
        assert!(Quality::Relic < Quality::Mythic);
    }

    #[test]
    fn title_shows_upgrade_level_only_when_upgraded() {
        let mut item = Item {
            id: ItemId(1),
            name: "灰灯短剑".into(),
            slot: Slot::MainHand,
            quality: Quality::Common,
            item_power: 10,
            upgrade_level: 0,
            reroll_count: 0,
            affixes: vec![],
            locked_affix: None,
            core_effect: None,
        };
        assert_eq!(item.title(), "灰灯短剑");
        item.upgrade_level = 3;
        assert_eq!(item.title(), "灰灯短剑 +3");
    }
}
