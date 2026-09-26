//! AIDLC state-change approval (design §9.4). NPCs — scripted now, AI-driven
//! later — can only *request* world changes; this rule layer decides. A
//! refusal carries an in-character reason for the NPC to say.

use serde::{Deserialize, Serialize};

use crate::world::{Effect, Flag, WorldState};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChangeKind {
    RecordFirstChoice,
    TouchTotemFragment,
    DefeatGrom,
    EscapeMine,
    KhahWhisper,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StateChangeRequest {
    pub kind: ChangeKind,
    pub requested_by: String,
    pub reason: String,
    pub effects: Vec<Effect>,
}

pub fn approve(req: &StateChangeRequest, world: &WorldState) -> Result<(), &'static str> {
    match req.kind {
        ChangeKind::EscapeMine => approve_escape(world),
        ChangeKind::RecordFirstChoice if world.dwarf_choice.is_some() => {
            Err("第一个永久选择已经写入世界。")
        }
        ChangeKind::TouchTotemFragment if world.has(Flag::TouchedTotemFragment) => {
            Err("图腾残片已经沉默了。它给过你它能给的一切。")
        }
        _ => Ok(()),
    }
}

fn approve_escape(world: &WorldState) -> Result<(), &'static str> {
    if !world.has(Flag::TouchedTotemFragment) {
        return Err("玩家还没有触碰图腾残片，矿井出口的黑潮不会退让。");
    }
    if !world.has(Flag::DefeatedGrom) {
        return Err("黑腕队长·格罗姆挡在出口前。先击败他。");
    }
    if world.has(Flag::EscapedMine) {
        return Err("矿井出口已经打开。灰灯镇的路在前方。");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::DwarfChoice;

    fn req(kind: ChangeKind) -> StateChangeRequest {
        StateChangeRequest { kind, requested_by: "测试".into(), reason: String::new(), effects: vec![] }
    }

    #[test]
    fn escape_requires_totem_then_grom_and_only_once() {
        let mut w = WorldState::default();
        assert!(approve(&req(ChangeKind::EscapeMine), &w).is_err());
        w.flags.insert(Flag::TouchedTotemFragment);
        assert_eq!(approve(&req(ChangeKind::EscapeMine), &w), Err("黑腕队长·格罗姆挡在出口前。先击败他。"));
        w.flags.insert(Flag::DefeatedGrom);
        assert!(approve(&req(ChangeKind::EscapeMine), &w).is_ok());
        w.flags.insert(Flag::EscapedMine);
        assert!(approve(&req(ChangeKind::EscapeMine), &w).is_err());
    }

    #[test]
    fn first_choice_is_permanent() {
        let mut w = WorldState::default();
        assert!(approve(&req(ChangeKind::RecordFirstChoice), &w).is_ok());
        w.dwarf_choice = Some(DwarfChoice::Save);
        assert!(approve(&req(ChangeKind::RecordFirstChoice), &w).is_err());
    }
}
