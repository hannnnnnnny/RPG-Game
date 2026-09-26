//! One playthrough: profile, world, bag, loadout, materials and log.
//! Every mutation goes through a method here so rules are enforced in one
//! place; the engine layer only reads it and drains `events`.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::aidlc::{self, StateChangeRequest};
use crate::combat::{self, KillSource};
use crate::forge::{self, Materials};
use crate::item::{Affix, Item, ItemId, Slot};
use crate::mind::{self, CorruptionTier, Meter, SanityTier};
use crate::stats::{self, Stats};
use crate::world::{Effect, Flag, WorldState};

const LOG_CAP: usize = 16;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
}

/// Things the presentation layer should react to (drained each frame).
#[derive(Clone, Debug, PartialEq)]
pub enum RunEvent {
    Logged(String),
    TierCrossed { meter: Meter, tier_name: &'static str },
    ItemGained(ItemId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionError {
    NoSuchItem,
    NotEnoughGold,
    NotEnoughMaterials,
    NotAllowed,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Run {
    pub profile: Profile,
    pub world: WorldState,
    pub inventory: Vec<Item>,
    pub equipped: BTreeMap<Slot, ItemId>,
    pub materials: Materials,
    pub log: VecDeque<String>,
    #[serde(skip)]
    pub events: Vec<RunEvent>,
}

impl Run {
    /// A fresh run: 克哈 has already woken the player (vessel stage 1).
    pub fn new(name: &str) -> Self {
        let mut world = WorldState::default();
        world.flags.insert(Flag::AwakenedByKhah);
        world.vessel_awakening = 1;
        let mut run = Self {
            profile: Profile { name: name.to_string() },
            world,
            inventory: Vec::new(),
            equipped: BTreeMap::new(),
            materials: Materials::new(),
            log: VecDeque::new(),
            events: Vec::new(),
        };
        run.log(format!("{name} 在黑潮矿区苏醒。"));
        run
    }

    pub fn log(&mut self, entry: String) {
        self.log.push_front(entry.clone());
        self.log.truncate(LOG_CAP);
        self.events.push(RunEvent::Logged(entry));
    }

    pub fn drain_events(&mut self) -> Vec<RunEvent> {
        std::mem::take(&mut self.events)
    }

    // ---------------- Items ----------------

    pub fn find_item(&self, id: ItemId) -> Option<&Item> {
        self.inventory.iter().find(|i| i.id == id)
    }

    pub fn equipped_items(&self) -> impl Iterator<Item = &Item> + Clone {
        self.inventory.iter().filter(|i| self.equipped.get(&i.slot) == Some(&i.id))
    }

    pub fn is_equipped(&self, item: &Item) -> bool {
        self.equipped.get(&item.slot) == Some(&item.id)
    }

    pub fn stats(&self) -> Stats {
        stats::derive(self.equipped_items())
    }

    pub fn add_item(&mut self, item: Item) {
        self.log(format!("获得装备：{}", item.title()));
        self.events.push(RunEvent::ItemGained(item.id));
        self.inventory.insert(0, item);
    }

    pub fn equip(&mut self, id: ItemId) -> Result<(), ActionError> {
        let item = self.find_item(id).ok_or(ActionError::NoSuchItem)?;
        let (slot, title) = (item.slot, item.title());
        self.equipped.insert(slot, id);
        self.log(format!("装备：{title}"));
        Ok(())
    }

    pub fn unequip(&mut self, slot: Slot) {
        if let Some(id) = self.equipped.remove(&slot) {
            let title = self.find_item(id).map(Item::title).unwrap_or_default();
            self.log(format!("卸下：{title}"));
        }
    }

    pub fn remove_item(&mut self, id: ItemId) -> Option<Item> {
        let idx = self.inventory.iter().position(|i| i.id == id)?;
        let item = self.inventory.remove(idx);
        if self.equipped.get(&item.slot) == Some(&id) {
            self.equipped.remove(&item.slot);
        }
        Some(item)
    }

    fn replace_item(&mut self, updated: Item) {
        if let Some(slot) = self.inventory.iter_mut().find(|i| i.id == updated.id) {
            *slot = updated;
        }
    }

    // ---------------- Gold & materials ----------------

    pub fn add_gold(&mut self, amount: u32) {
        self.world.gold += amount;
        self.log(format!("获得 {amount} 金币。"));
    }

    pub fn spend_gold(&mut self, amount: u32) -> Result<(), ActionError> {
        if self.world.gold < amount {
            return Err(ActionError::NotEnoughGold);
        }
        self.world.gold -= amount;
        Ok(())
    }

    pub fn add_kill_gold(&mut self, source: KillSource, rng: &mut fastrand::Rng) -> u32 {
        let base = combat::gold_for_kill(source, self.world.world_tier, rng);
        let gold = combat::apply_gold_find(base, &self.stats());
        self.add_gold(gold);
        gold
    }

    pub fn add_materials(&mut self, gained: &Materials) {
        for (m, n) in gained {
            *self.materials.entry(*m).or_default() += n;
        }
    }

    pub fn has_materials(&self, cost: &Materials) -> bool {
        cost.iter().all(|(m, n)| self.materials.get(m).copied().unwrap_or(0) >= *n)
    }

    /// All-or-nothing.
    pub fn spend_materials(&mut self, cost: &Materials) -> Result<(), ActionError> {
        if !self.has_materials(cost) {
            return Err(ActionError::NotEnoughMaterials);
        }
        for (m, n) in cost {
            *self.materials.entry(*m).or_default() -= n;
        }
        Ok(())
    }

    /// Light death (design §12): lose 10% gold, keep every item.
    pub fn on_player_down(&mut self) -> u32 {
        let lost = combat::death_gold_loss(self.world.gold);
        self.world.gold -= lost;
        self.log(format!("倒下了。遗失 {lost} 金币。"));
        lost
    }
}

// ---------------- Forge actions ----------------

impl Run {
    pub fn forge_upgrade(&mut self, id: ItemId) -> Result<(), ActionError> {
        let item = self.find_item(id).ok_or(ActionError::NoSuchItem)?.clone();
        if !forge::can_upgrade(&item) {
            return Err(ActionError::NotAllowed);
        }
        self.spend_gold(forge::upgrade_cost(&item))?;
        let out = forge::upgraded(&item);
        self.log(format!("强化 {} → +{}", item.name, out.upgrade_level));
        self.replace_item(out);
        Ok(())
    }

    /// Pays for a reroll and returns the offer; resolve it with
    /// `forge_resolve_reroll` (keep old or take new).
    pub fn forge_reroll_offer(
        &mut self,
        id: ItemId,
        index: usize,
        rng: &mut fastrand::Rng,
    ) -> Result<Affix, ActionError> {
        let item = self.find_item(id).ok_or(ActionError::NoSuchItem)?.clone();
        if !forge::can_reroll(&item, index) {
            return Err(ActionError::NotAllowed);
        }
        self.spend_gold(forge::reroll_cost(&item))?;
        Ok(forge::reroll_offer(&item, rng))
    }

    pub fn forge_resolve_reroll(&mut self, id: ItemId, index: usize, offer: Affix, accept: bool) {
        let Some(item) = self.find_item(id).cloned() else { return };
        self.replace_item(forge::resolve_reroll(&item, index, offer, accept));
        let what = if accept { "接受新词条" } else { "保留旧词条" };
        self.log(format!("重铸 {}：{what}", item.name));
    }

    pub fn forge_lock(&mut self, id: ItemId, index: usize) -> Result<(), ActionError> {
        let item = self.find_item(id).ok_or(ActionError::NoSuchItem)?.clone();
        self.spend_materials(&forge::lock_cost())?;
        self.replace_item(forge::locked(&item, index));
        self.log(format!("锁定 {} 的一条词条。", item.name));
        Ok(())
    }

    pub fn salvage(&mut self, id: ItemId) -> Result<Materials, ActionError> {
        let item = self.remove_item(id).ok_or(ActionError::NoSuchItem)?;
        let gained = forge::salvage_yield(&item);
        self.add_materials(&gained);
        self.log(format!("分解 {}。", item.name));
        Ok(gained)
    }
}

// ---------------- World changes ----------------

impl Run {
    /// Clamp, soften sanity loss by 理智稳定, and report tier crossings.
    pub fn change_meter(&mut self, meter: Meter, delta: i32) {
        let delta = if meter == Meter::Sanity {
            mind::soften_loss(delta, self.stats().sanity_guard)
        } else {
            delta
        };
        let before = self.world.meter(meter);
        let after = mind::apply(before, delta);
        *self.world.meter_mut(meter) = after;
        self.report_tier_crossing(meter, before, after);
    }

    fn report_tier_crossing(&mut self, meter: Meter, before: u8, after: u8) {
        let (from, to, hint) = match meter {
            Meter::Sanity => {
                let t = SanityTier::of(after);
                (SanityTier::of(before).name(), t.name(), t.hint())
            }
            Meter::Corruption => {
                (CorruptionTier::of(before).name(), CorruptionTier::of(after).name(), "")
            }
            Meter::ParasiteLoad => return,
        };
        if from == to {
            return;
        }
        let worse = (meter == Meter::Sanity) == (after < before);
        let dir = if worse { "跌入" } else { "回到" };
        self.log(format!("{}{dir}「{to}」。{hint}", meter.label()));
        self.events.push(RunEvent::TierCrossed { meter, tier_name: to });
    }

    fn apply_effect(&mut self, effect: &Effect) {
        match effect {
            Effect::SetFlag(f) => {
                self.world.flags.insert(*f);
            }
            Effect::DwarfChoice(c) => self.world.dwarf_choice = Some(*c),
            Effect::RaiseVessel(v) => {
                self.world.vessel_awakening = self.world.vessel_awakening.max(*v);
            }
            Effect::Meter(m, d) => self.change_meter(*m, *d),
        }
    }

    /// AIDLC gate: approve, then apply. Err is the NPC's in-character refusal.
    pub fn request(&mut self, req: &StateChangeRequest) -> Result<(), &'static str> {
        aidlc::approve(req, &self.world)?;
        for effect in &req.effects {
            self.apply_effect(effect);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aidlc::ChangeKind;
    use crate::forge::Material;
    use crate::item::{AffixId, AffixKind, Quality};
    use crate::world::DwarfChoice;

    fn weapon(value: i32) -> Item {
        Item {
            id: ItemId(7),
            name: "测试剑".into(),
            slot: Slot::MainHand,
            quality: Quality::Rare,
            item_power: 20,
            upgrade_level: 0,
            reroll_count: 0,
            affixes: vec![Affix { id: AffixId(1), kind: AffixKind::MeleeDamage, value }],
            locked_affix: None,
            core_effect: None,
        }
    }

    fn req(kind: ChangeKind, effects: Vec<Effect>) -> StateChangeRequest {
        StateChangeRequest { kind, requested_by: "测试".into(), reason: String::new(), effects }
    }

    #[test]
    fn new_run_is_awakened() {
        let run = Run::new("迪丝");
        assert!(run.world.has(Flag::AwakenedByKhah));
        assert_eq!(run.world.vessel_awakening, 1);
    }

    #[test]
    fn equipped_weapon_raises_attack() {
        let mut run = Run::new("t");
        run.add_item(weapon(5));
        assert_eq!(run.stats().attack, stats::BASE_ATTACK);
        run.equip(ItemId(7)).unwrap();
        assert_eq!(run.stats().attack, stats::BASE_ATTACK + 5);
    }

    #[test]
    fn unequip_clears_slot_and_stats() {
        let mut run = Run::new("t");
        run.add_item(weapon(5));
        run.equip(ItemId(7)).unwrap();
        run.unequip(Slot::MainHand);
        assert_eq!(run.stats().attack, stats::BASE_ATTACK);
        assert_eq!(run.inventory.len(), 1, "item stays in the bag");
    }

    #[test]
    fn upgrade_pays_and_scales_equipped_stats() {
        let mut run = Run::new("t");
        run.add_item(weapon(10));
        run.equip(ItemId(7)).unwrap();
        run.world.gold = 9999;
        for _ in 0..5 {
            run.forge_upgrade(ItemId(7)).unwrap();
        }
        assert_eq!(run.stats().attack, stats::BASE_ATTACK + 15);
        assert!(run.world.gold < 9999);
    }

    #[test]
    fn broke_player_cannot_upgrade() {
        let mut run = Run::new("t");
        run.add_item(weapon(10));
        assert_eq!(run.forge_upgrade(ItemId(7)), Err(ActionError::NotEnoughGold));
    }

    #[test]
    fn lock_needs_rare_material_and_salvage_grants() {
        let mut run = Run::new("t");
        run.add_item(weapon(10));
        assert_eq!(run.forge_lock(ItemId(7), 0), Err(ActionError::NotEnoughMaterials));
        run.salvage(ItemId(7)).unwrap();
        assert!(run.find_item(ItemId(7)).is_none());
        assert_eq!(run.materials[&Material::Rare], 1);
    }

    #[test]
    fn death_costs_gold_never_gear() {
        let mut run = Run::new("t");
        run.add_item(weapon(5));
        run.equip(ItemId(7)).unwrap();
        run.world.gold = 300;
        assert_eq!(run.on_player_down(), 30);
        assert_eq!(run.world.gold, 270);
        assert!(run.is_equipped(&run.inventory[0]));
    }

    /// Regression: the Godot choice panel overwrote 污染 with its "delta".
    #[test]
    fn first_choice_after_totem_never_lowers_corruption() {
        let mut run = Run::new("t");
        run.request(&req(ChangeKind::TouchTotemFragment, vec![
            Effect::SetFlag(Flag::TouchedTotemFragment),
            Effect::Meter(Meter::Corruption, 4),
        ]))
        .unwrap();
        let after_totem = run.world.corruption;
        run.request(&req(ChangeKind::RecordFirstChoice, vec![
            Effect::DwarfChoice(DwarfChoice::Save),
            Effect::Meter(Meter::Corruption, 0),
        ]))
        .unwrap();
        assert_eq!(run.world.corruption, after_totem);
    }

    #[test]
    fn refused_request_changes_nothing() {
        let mut run = Run::new("t");
        let before = run.world.clone();
        let err = run.request(&req(ChangeKind::EscapeMine, vec![Effect::SetFlag(Flag::EscapedMine)]));
        assert!(err.is_err());
        assert_eq!(run.world, before);
    }

    #[test]
    fn tier_crossing_is_logged_and_evented() {
        let mut run = Run::new("t");
        run.drain_events();
        run.world.sanity = 50;
        run.change_meter(Meter::Sanity, -1);
        assert!(run.log[0].starts_with("理智跌入「破裂」"), "{}", run.log[0]);
        assert!(run.drain_events().contains(&RunEvent::TierCrossed {
            meter: Meter::Sanity,
            tier_name: "破裂"
        }));
    }

    #[test]
    fn vessel_only_rises() {
        let mut run = Run::new("t");
        run.world.vessel_awakening = 3;
        run.apply_effect(&Effect::RaiseVessel(2));
        assert_eq!(run.world.vessel_awakening, 3);
    }
}
