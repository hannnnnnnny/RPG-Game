//! The quest journal, derived from world flags so it can never disagree
//! with the story state.

use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Objective {
    pub text: &'static str,
    pub done: bool,
}

const fn obj(text: &'static str, done: bool) -> Objective {
    Objective { text, done }
}

/// (chapter title, objectives) — completed steps stay listed, ticked.
pub fn current(run: &Run) -> (&'static str, Vec<Objective>) {
    let w = &run.world;
    if !w.has(Flag::EscapedMine) {
        let touched = w.has(Flag::TouchedTotemFragment);
        let grom = w.has(Flag::DefeatedGrom);
        let mut list = vec![
            obj("处理受伤矮人的命运", w.dwarf_choice.is_some()),
            obj("触碰图腾残片", touched),
        ];
        if touched {
            list.push(obj("击败黑腕队长·格罗姆", grom));
        }
        if grom {
            list.push(obj("前往矿井出口", false));
        }
        return ("序章 · 逃出黑潮矿区", list);
    }
    ("第一章 · 灰灯镇", vec![
        obj("逃出黑潮矿区", true),
        obj("与灰灯镇的镇民交谈", false),
        obj("找铁匠老锤看看你的装备", false),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_follows_the_opening() {
        let mut run = Run::new("t");
        let (chapter, list) = current(&run);
        assert!(chapter.starts_with("序章"));
        assert_eq!(list.len(), 2);
        mine::interact(&mut run, mine::Spot::TotemFragment);
        assert_eq!(current(&run).1.len(), 3);
        mine::grom_defeated(&mut run);
        mine::interact(&mut run, mine::Spot::Exit);
        assert!(current(&run).0.starts_with("第一章"));
    }
}
