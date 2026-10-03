//! Three-axis mind system (design §10): 理智, 污染, 容器觉醒.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Meter {
    Sanity,
    Corruption,
    ParasiteLoad,
}

impl Meter {
    pub fn label(self) -> &'static str {
        match self {
            Meter::Sanity => "理智",
            Meter::Corruption => "污染",
            Meter::ParasiteLoad => "寄生",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SanityTier {
    Stable,
    Shaken,
    Fractured,
    Collapsing,
}

impl SanityTier {
    pub fn of(sanity: u8) -> Self {
        match sanity {
            80.. => SanityTier::Stable,
            50..=79 => SanityTier::Shaken,
            20..=49 => SanityTier::Fractured,
            _ => SanityTier::Collapsing,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SanityTier::Stable => "稳定",
            SanityTier::Shaken => "动摇",
            SanityTier::Fractured => "破裂",
            SanityTier::Collapsing => "濒临崩溃",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            SanityTier::Stable => "NPC 更信任你，但有些真相藏在你看不见的地方。",
            SanityTier::Shaken => "你开始看见刺青的异样，和残歌里的碎片。",
            SanityTier::Fractured => "隐藏的门在墙上浮现。克哈的痕迹无处不在。",
            SanityTier::Collapsing => "真相涌进来——混着幻听和谎言。",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CorruptionTier {
    Clear,
    TideStained,
    BlackVein,
    NearGod,
}

impl CorruptionTier {
    pub fn of(corruption: u8) -> Self {
        match corruption {
            81.. => CorruptionTier::NearGod,
            56..=80 => CorruptionTier::BlackVein,
            26..=55 => CorruptionTier::TideStained,
            _ => CorruptionTier::Clear,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CorruptionTier::Clear => "清醒之身",
            CorruptionTier::TideStained => "染潮",
            CorruptionTier::BlackVein => "黑脉",
            CorruptionTier::NearGod => "近神之壳",
        }
    }
}

pub const VESSEL_STAGES: [&str; 6] =
    ["空壳未醒", "低语入梦", "印痕显现", "双容器共鸣", "承灾之器", "终局容器"];

pub fn vessel_stage_name(stage: u8) -> &'static str {
    VESSEL_STAGES[usize::from(stage).min(VESSEL_STAGES.len() - 1)]
}

/// 理智稳定 softens sanity *losses* with diminishing returns; gains are
/// untouched and a loss is never fully erased (guard 100 → half).
pub fn soften_loss(delta: i32, sanity_guard: i32) -> i32 {
    if delta >= 0 {
        return delta;
    }
    let factor = 100.0 / (100.0 + sanity_guard.max(0) as f32);
    ((delta as f32 * factor).round() as i32).min(-1)
}

/// Apply a delta to a 0–100 meter.
pub fn apply(value: u8, delta: i32) -> u8 {
    (i32::from(value) + delta).clamp(0, 100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanity_tiers() {
        assert_eq!(SanityTier::of(80), SanityTier::Stable);
        assert_eq!(SanityTier::of(79), SanityTier::Shaken);
        assert_eq!(SanityTier::of(20), SanityTier::Fractured);
        assert_eq!(SanityTier::of(19), SanityTier::Collapsing);
    }

    #[test]
    fn corruption_tiers() {
        assert_eq!(CorruptionTier::of(25).name(), "清醒之身");
        assert_eq!(CorruptionTier::of(26).name(), "染潮");
        assert_eq!(CorruptionTier::of(56).name(), "黑脉");
        assert_eq!(CorruptionTier::of(81).name(), "近神之壳");
    }

    #[test]
    fn vessel_names_clamp() {
        assert_eq!(vessel_stage_name(1), "低语入梦");
        assert_eq!(vessel_stage_name(99), "终局容器");
    }

    #[test]
    fn guard_softens_losses_only() {
        assert_eq!(soften_loss(-10, 0), -10);
        assert_eq!(soften_loss(-10, 100), -5);
        assert_eq!(soften_loss(8, 100), 8);
        assert_eq!(soften_loss(-1, 1000), -1);
    }

    #[test]
    fn meters_clamp() {
        assert_eq!(apply(96, 10), 100);
        assert_eq!(apply(5, -20), 0);
    }
}
