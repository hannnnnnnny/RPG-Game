//! 灰灯镇 — the first safe place. Townsfolk, their lines, and the gate
//! warden who reads your 污染 before letting you in.

use super::*;
use crate::areas::Pt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Robe {
    Black,
    Brown,
    Blue,
    Red,
    ForestGreen,
    DarkGray,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Head {
    Hood,
    Plain,
    Long,
    BangsShort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Warden,
    Smith,
    Folk,
}

#[derive(Clone, Copy, Debug)]
pub struct Npc {
    pub name: &'static str,
    pub pos: Pt,
    pub robe: Robe,
    pub head: Head,
    pub role: Role,
    pub seated: bool,
    pub wanders: bool,
    pub lines: &'static [&'static str],
}

const fn folk(name: &'static str, pos: Pt, robe: Robe, head: Head, lines: &'static [&'static str]) -> Npc {
    Npc { name, pos, robe, head, role: Role::Folk, seated: false, wanders: true, lines }
}

const fn seated(name: &'static str, pos: Pt, head: Head, lines: &'static [&'static str]) -> Npc {
    Npc { name, pos, robe: Robe::Brown, head, role: Role::Folk, seated: true, wanders: false, lines }
}

pub const NPCS: [Npc; 12] = [
    Npc { name: "灰灯门卫", pos: (650.0, 660.0), robe: Robe::DarkGray, head: Head::Hood, role: Role::Warden, seated: false, wanders: false, lines: &[] },
    Npc { name: "守夜人", pos: (1010.0, 740.0), robe: Robe::Black, head: Head::Hood, role: Role::Folk, seated: false, wanders: false,
        lines: &["夜里别靠近镇墙。灯一灭，墙外的东西就贴上来。", "你是从矿里出来的？那下面……还有活人吗？"] },
    Npc { name: "铁匠·老锤", pos: (1130.0, 716.0), robe: Robe::Brown, head: Head::Plain, role: Role::Smith, seated: false, wanders: false,
        lines: &["砧子在那儿。强化不会炸，我手稳。重铸嘛……一次只动一条纹路，动多了铁会记仇。",
            "用不上的破烂拿来拆了，拆出来的料比卖给铜婶值钱。",
            "锁一条词条要稀有材料。矿里那些黑东西身上偶尔掉。"] },
    folk("卖灯油的老汉", (560.0, 410.0), Robe::Brown, Head::Plain, &["灯油涨价了，可没人敢不买。黑里头，灯就是命。"]),
    folk("缝补匠", (760.0, 430.0), Robe::ForestGreen, Head::Long, &["你那披风破成这样……坐下，我给你缝两针，不收钱。"]),
    folk("醉汉", (840.0, 470.0), Robe::Brown, Head::BangsShort, &["再来一碗！黑潮要来就来，老子先喝够本……"]),
    folk("传教者", (520.0, 540.0), Robe::Black, Head::Hood, &["克哈不是恶魔，是潮水。潮水来时，聪明人学会游泳。"]),
    folk("巡镇民兵", (880.0, 540.0), Robe::ForestGreen, Head::Plain, &["手别离剑太远。灰灯镇看着太平，太平是装的。"]),
    folk("挑水的少年", (980.0, 420.0), Robe::Blue, Head::BangsShort, &["井水我来挑就好。你是英雄吧？英雄不挑水。"]),
    seated("抱孩子的妇人", (610.0, 500.0), Head::Long, &["嘘，孩子刚睡。他总梦见水……我怕。"]),
    seated("矿工遗孀", (760.0, 500.0), Head::Plain, &["我男人也下了那个矿。你……见过黑腕队长吗？"]),
    seated("歇脚的老妪", (690.0, 520.0), Head::Long, &["老啦，走两步就喘。坐在灯下，听听人声，也算活着。"]),
];

pub fn on_enter(_run: &Run) -> Beat {
    Beat::say(Line::khah("灯还亮着。可惜灯不知道，它照的人里有一个已经属于海。"))
}

/// The warden reads your corruption; everyone else cycles their lines.
pub fn talk(run: &Run, npc: &Npc, times_talked: usize) -> Beat {
    let text = match npc.role {
        Role::Warden => warden_line(run),
        _ if npc.lines.is_empty() => "……".to_string(),
        _ => npc.lines[times_talked % npc.lines.len()].to_string(),
    };
    Beat::say(Line::new(npc.name, text, Tone::Memory))
}

fn warden_line(run: &Run) -> String {
    match run.world.corruption {
        56.. => "站住。你身上有黑潮的味道……灯火认得它。再往里走一步，我就敲钟。".into(),
        26..=55 => "手腕给我看看。没有刺青就放你进——但别和镇民走太近。".into(),
        _ => format!("{}，活着从矿里出来的不多。进来吧，灰灯还亮着。", run.profile.name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::areas::{Town, walkable};

    #[test]
    fn every_npc_stands_somewhere_walkable() {
        for n in NPCS {
            assert!(walkable(&Town, n.pos.0, n.pos.1), "{} at {:?}", n.name, n.pos);
        }
    }

    #[test]
    fn warden_reacts_to_corruption() {
        let mut run = Run::new("迪丝");
        let warden = &NPCS[0];
        assert!(talk(&run, warden, 0).lines[0].text.starts_with("迪丝"));
        run.world.corruption = 60;
        assert!(talk(&run, warden, 0).lines[0].text.starts_with("站住"));
    }

    #[test]
    fn folk_cycle_their_lines() {
        let run = Run::new("t");
        let smith = &NPCS[2];
        assert_ne!(talk(&run, smith, 0), talk(&run, smith, 1));
        assert_eq!(talk(&run, smith, 0), talk(&run, smith, 3));
    }
}

/// A readable prop (notice board, fountain, well): cycles its lines.
pub fn sign(title: &str, lines: &[&str], uses: usize) -> Beat {
    let text = lines.get(uses % lines.len().max(1)).copied().unwrap_or("……");
    Beat::say(Line::new(title, text, Tone::Memory))
}
