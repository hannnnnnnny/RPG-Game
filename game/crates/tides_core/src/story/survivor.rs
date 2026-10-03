//! 布林, the injured dwarf — only if you chose 救他. He limps after you
//! through the rest of the mine, then settles by 老锤's forge in 灰灯镇.
//! What he says tracks how far the story has gone.

use super::*;
use crate::areas::Pt;

pub const NAME: &str = "矮人幸存者·布林";

/// Where he waits in town: near the forge, where dwarves feel at home.
pub const TOWN_POS: Pt = (1000.0, 680.0);

const IN_MINE: &[&str] = &[
    "别管我，我跟得上……腿是瘸了，又不是断了。",
    "前面是队长的地盘。他已经不是他了，别犹豫。",
    "那块图腾……我们本该先查封印的。是我们挖开的。",
];
const IN_TOWN: &[&str] = &[
    "老锤让我帮着拉风箱。手有活干，脑子就不往回想。",
    "深井里还有人。等我腿好了，我得回去。",
    "镇上的人盯着我的手腕看。没刺青——可我梦见过水。",
];

pub fn is_with_you(run: &Run) -> bool {
    run.world.dwarf_choice == Some(DwarfChoice::Save)
}

pub fn talk(run: &Run, times_talked: usize) -> Beat {
    let text = if !run.world.has(Flag::EscapedMine) {
        IN_MINE[times_talked % IN_MINE.len()]
    } else if run.world.has(Flag::DefeatedGrom) && times_talked == 0 {
        "格罗姆队长……谢谢你让他停下来。他要是还清醒，也会这么求你的。"
    } else {
        IN_TOWN[times_talked % IN_TOWN.len()]
    };
    Beat::say(Line::new(NAME, text, Tone::Memory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::{Town, walkable};

    fn saved() -> Run {
        let mut run = Run::new("t");
        run.world.dwarf_choice = Some(DwarfChoice::Save);
        run
    }

    #[test]
    fn only_follows_if_saved() {
        let mut run = Run::new("t");
        assert!(!is_with_you(&run));
        run.world.dwarf_choice = Some(DwarfChoice::Abandon);
        assert!(!is_with_you(&run));
        assert!(is_with_you(&saved()));
    }

    #[test]
    fn mine_and_town_lines_differ() {
        let mut run = saved();
        let mine = talk(&run, 0);
        run.world.flags.insert(Flag::EscapedMine);
        assert_ne!(mine, talk(&run, 0));
    }

    #[test]
    fn thanks_you_for_grom_first() {
        let mut run = saved();
        run.world.flags.insert(Flag::EscapedMine);
        run.world.flags.insert(Flag::DefeatedGrom);
        assert!(talk(&run, 0).lines[0].text.contains("格罗姆"));
        assert!(!talk(&run, 1).lines[0].text.contains("格罗姆"));
    }

    #[test]
    fn waits_somewhere_walkable() {
        assert!(walkable(&Town, TOWN_POS.0, TOWN_POS.1));
    }
}
