//! Story beats. Each function applies its AIDLC requests to the `Run` and
//! returns a `Beat` describing what the player should see next; the engine
//! only presents beats. Keeping this pure makes the plot unit-testable.

use crate::aidlc::{ChangeKind, StateChangeRequest};
use crate::combat::KillSource;
use crate::loot::DropSource;
use crate::mind::Meter;
use crate::run::Run;
use crate::world::{DwarfChoice, Effect, Flag};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// 克哈 in your head.
    Whisper,
    /// A refusal or danger.
    Warning,
    /// Echoes, memories, narration.
    Memory,
}

pub const KHAH: &str = "克哈低语";

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub speaker: String,
    pub text: String,
    pub tone: Tone,
}

impl Line {
    pub fn new(speaker: &str, text: impl Into<String>, tone: Tone) -> Self {
        Self { speaker: speaker.to_string(), text: text.into(), tone }
    }

    pub fn khah(text: impl Into<String>) -> Self {
        Self::new(KHAH, text, Tone::Whisper)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceOption {
    pub choice: DwarfChoice,
    pub label: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub title: &'static str,
    pub body: &'static str,
    pub options: Vec<ChoiceOption>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vision {
    /// Asset path of the full-screen image.
    pub image: &'static str,
    pub caption: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AreaId {
    Mine,
    Town,
}

/// Everything a story moment asks the presentation layer to do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Beat {
    pub lines: Vec<Line>,
    pub choice: Option<Choice>,
    pub vision: Option<Vision>,
    pub drop: Option<DropSource>,
    pub gold: Option<KillSource>,
    pub spawn_boss: bool,
    pub go_to: Option<AreaId>,
}

impl Beat {
    fn say(line: Line) -> Self {
        Self { lines: vec![line], ..Default::default() }
    }
}

fn request(kind: ChangeKind, by: &str, reason: &str, effects: Vec<Effect>) -> StateChangeRequest {
    StateChangeRequest { kind, requested_by: by.to_string(), reason: reason.to_string(), effects }
}

pub mod mine;
pub mod town;
