//! 黑潮矿区 — the opening (design §5): wake, first permanent choice, the
//! totem vision, 黑腕队长·格罗姆, and the way out to 灰灯镇.

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Spot {
    InjuredDwarf,
    TotemFragment,
    Exit,
}

/// Whether a spot still offers an interaction.
pub fn is_active(run: &Run, spot: Spot) -> bool {
    match spot {
        Spot::InjuredDwarf => run.world.dwarf_choice.is_none(),
        Spot::TotemFragment => !run.world.has(Flag::TouchedTotemFragment),
        Spot::Exit => true,
    }
}

pub fn on_enter(run: &Run) -> Beat {
    let text = if run.world.has(Flag::DefeatedGrom) {
        "他空了。你没有。出口在等你。".to_string()
    } else {
        format!("{}，往前。那些矿灯已经死了，但你的血还记得路。", run.profile.name)
    };
    Beat::say(Line::khah(text))
}

pub fn interact(run: &mut Run, spot: Spot) -> Beat {
    match spot {
        Spot::InjuredDwarf => injured_dwarf(run),
        Spot::TotemFragment => totem(run),
        Spot::Exit => exit(run),
    }
}

fn injured_dwarf(run: &mut Run) -> Beat {
    if run.world.dwarf_choice.is_some() {
        return Beat::default();
    }
    let met = request(ChangeKind::KhahWhisper, KHAH, "玩家靠近第一个永久选择", vec![
        Effect::SetFlag(Flag::MetInjuredDwarf),
    ]);
    let _ = run.request(&met);
    Beat {
        choice: Some(Choice {
            title: "第一个永久选择",
            body: "一个矮人倒在铁轨旁，手腕布条下有正在扩散的刺青。低语催你继续走。",
            options: vec![
                ChoiceOption { choice: DwarfChoice::Save, label: "救他", description: "理智稳定，但小镇可能承受感染风险。" },
                ChoiceOption { choice: DwarfChoice::Abandon, label: "抛下他", description: "听从低语，安全离开。" },
                ChoiceOption { choice: DwarfChoice::Kill, label: "终结他", description: "污染上升，猎人会认可这种冷酷。" },
            ],
        }),
        ..Default::default()
    }
}

/// Relative effects per option, so earlier events are never undone.
fn choice_effects(c: DwarfChoice) -> (i32, i32, u32) {
    match c {
        DwarfChoice::Save => (4, 0, 8),
        DwarfChoice::Abandon => (-8, 3, 16),
        DwarfChoice::Kill => (-14, 8, 16),
    }
}

pub fn choose(run: &mut Run, c: DwarfChoice) -> Beat {
    let (sanity, corruption, gold) = choice_effects(c);
    let req = request(ChangeKind::RecordFirstChoice, "受伤矮人", "第一个永久选择", vec![
        Effect::DwarfChoice(c),
        Effect::Meter(Meter::Sanity, sanity),
        Effect::Meter(Meter::Corruption, corruption),
    ]);
    if let Err(refusal) = run.request(&req) {
        return Beat::say(Line::new("受伤矮人", refusal, Tone::Warning));
    }
    run.add_gold(gold);
    Beat::say(match c {
        DwarfChoice::Save => Line::new("受伤矮人", "他还活着。也许这会让灰灯镇多一个问题，也许多一个证人。", Tone::Memory),
        DwarfChoice::Abandon => Line::khah("很好。怜悯会让矿道坍得更慢，但不会让你活得更久。"),
        DwarfChoice::Kill => Line::khah("血没有溅到你身上，它像认识你一样避开了。"),
    })
}

const TOTEM_VISION: &str = "矮人队长打开图腾的一瞬间，你看见紫色皮肤、黑色眼睛，以及一只虫子钻入王冠。";

fn totem(run: &mut Run) -> Beat {
    let req = request(ChangeKind::TouchTotemFragment, "图腾残片", "玩家触碰第一块封印残片", vec![
        Effect::SetFlag(Flag::TouchedTotemFragment),
        Effect::RaiseVessel(2),
        Effect::Meter(Meter::Corruption, 4),
        // Seeing the captain consumed shakes you.
        Effect::Meter(Meter::Sanity, -6),
    ]);
    if run.request(&req).is_err() {
        return Beat::default();
    }
    Beat {
        lines: vec![Line::new("残响", TOTEM_VISION, Tone::Memory)],
        vision: Some(Vision { image: "characters/dwarf_captain_vision.jpg", caption: TOTEM_VISION }),
        drop: Some(DropSource::Totem),
        // The vision calls his corrupted form to guard the exit.
        spawn_boss: true,
        ..Default::default()
    }
}

fn exit(run: &mut Run) -> Beat {
    if run.world.has(Flag::EscapedMine) {
        return Beat { go_to: Some(AreaId::Town), ..Default::default() };
    }
    let req = request(ChangeKind::EscapeMine, "矿井出口", "玩家试图离开黑潮矿区", vec![
        Effect::SetFlag(Flag::EscapedMine),
    ]);
    match run.request(&req) {
        Err(refusal) => Beat::say(Line::new("矿井出口", refusal, Tone::Warning)),
        Ok(()) => Beat {
            lines: vec![Line::khah("很好。现在去找那些还以为灯能挡住海的人。")],
            go_to: Some(AreaId::Town),
            ..Default::default()
        },
    }
}

pub fn grom_defeated(run: &mut Run) -> Beat {
    let req = request(ChangeKind::DefeatGrom, "黑腕队长·格罗姆", "玩家击败了腐化的矮人队长", vec![
        Effect::SetFlag(Flag::DefeatedGrom),
        Effect::RaiseVessel(3),
    ]);
    if run.request(&req).is_err() {
        return Beat::default();
    }
    Beat {
        lines: vec![Line::khah(
            "他空了。你没有。出口的黑潮已经为你让路——去灰灯镇，那里还有人以为灯能挡住海。",
        )],
        drop: Some(DropSource::Elite),
        gold: Some(KillSource::Boss),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_opening_path() {
        let mut run = Run::new("迪丝");
        assert!(on_enter(&run).lines[0].text.starts_with("迪丝"));

        let b = interact(&mut run, Spot::InjuredDwarf);
        assert_eq!(b.choice.unwrap().options.len(), 3);
        choose(&mut run, DwarfChoice::Save);
        assert!(!is_active(&run, Spot::InjuredDwarf));

        let blocked = interact(&mut run, Spot::Exit);
        assert_eq!(blocked.lines[0].tone, Tone::Warning);
        assert_eq!(blocked.go_to, None);

        let t = interact(&mut run, Spot::TotemFragment);
        assert!(t.spawn_boss && t.vision.is_some());
        assert_eq!(run.world.vessel_awakening, 2);

        grom_defeated(&mut run);
        assert_eq!(run.world.vessel_awakening, 3);
        assert_eq!(interact(&mut run, Spot::Exit).go_to, Some(AreaId::Town));
    }

    #[test]
    fn totem_only_once() {
        let mut run = Run::new("t");
        assert!(interact(&mut run, Spot::TotemFragment).spawn_boss);
        assert_eq!(interact(&mut run, Spot::TotemFragment), Beat::default());
    }

    #[test]
    fn kill_is_harsher_than_save() {
        let mut a = Run::new("t");
        let mut b = Run::new("t");
        choose(&mut a, DwarfChoice::Save);
        choose(&mut b, DwarfChoice::Kill);
        assert!(b.world.corruption > a.world.corruption);
        assert!(b.world.sanity < a.world.sanity);
    }
}
