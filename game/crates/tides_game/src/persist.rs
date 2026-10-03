//! Where the save lives and when it's written. The format is tides_core's
//! versioned JSON; this module only moves the text.
//!
//! Native: a file in `$TIDES_SAVE_DIR`, else `%APPDATA%/TidesOfKhah`
//! (Windows) or `$XDG_DATA_HOME|~/.local/share/tides_of_khah`. Writes are
//! atomic (temp file + rename) so a crash mid-save never leaves half a file.
//! Web: one `localStorage` key.
//!
//! Autosave: on entering an area, when a window (forge, shop, choice…)
//! closes, and every `AUTOSAVE_SECS`. Staged dev runs never save.

use bevy::prelude::*;
use tides_core::run::Run;
use tides_core::save::{self, LoadError};

use crate::area::Area;
use crate::beats::Modal;
use crate::run_state::RunRes;

const AUTOSAVE_SECS: f32 = 30.0;

#[derive(Debug)]
pub enum SaveError {
    #[cfg(not(target_arch = "wasm32"))]
    Io(std::io::Error),
    /// The browser refused (storage disabled, quota, private mode…).
    #[cfg(target_arch = "wasm32")]
    Web(&'static str),
    Load(LoadError),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            SaveError::Io(e) => write!(f, "读写失败：{e}"),
            #[cfg(target_arch = "wasm32")]
            SaveError::Web(why) => write!(f, "浏览器存储不可用：{why}"),
            SaveError::Load(LoadError::Corrupt(_)) => write!(f, "内容损坏"),
            SaveError::Load(LoadError::TooNew(v)) => write!(f, "来自更新的版本 v{v}"),
        }
    }
}

#[derive(Resource, Clone, Debug)]
pub struct SaveSlot {
    store: store::Store,
}

impl SaveSlot {
    pub fn locate() -> Self {
        Self { store: store::Store::locate() }
    }

    /// `Ok(None)` when there is simply no save yet.
    pub fn read(&self) -> Result<Option<Run>, SaveError> {
        match self.store.read_text()? {
            Some(text) => save::decode(&text).map(Some).map_err(SaveError::Load),
            None => Ok(None),
        }
    }

    pub fn write(&self, run: &Run) -> Result<(), SaveError> {
        self.store.write_text(&save::encode(run))
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod store {
    use std::path::PathBuf;
    use std::{fs, io};

    use super::SaveError;

    const FILE: &str = "save.json";

    #[derive(Clone, Debug)]
    pub struct Store {
        pub dir: PathBuf,
    }

    impl Store {
        /// The platform's per-user data directory (see module docs).
        pub fn locate() -> Self {
            let env = |k: &str| std::env::var_os(k).map(PathBuf::from);
            let dir = env("TIDES_SAVE_DIR")
                .or_else(|| env("APPDATA").map(|d| d.join("TidesOfKhah")))
                .or_else(|| env("XDG_DATA_HOME").map(|d| d.join("tides_of_khah")))
                .or_else(|| env("HOME").map(|d| d.join(".local/share/tides_of_khah")))
                .unwrap_or_else(|| PathBuf::from("."));
            Self { dir }
        }

        pub fn path(&self) -> PathBuf {
            self.dir.join(FILE)
        }

        pub fn read_text(&self) -> Result<Option<String>, SaveError> {
            match fs::read_to_string(self.path()) {
                Ok(text) => Ok(Some(text)),
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(e) => Err(SaveError::Io(e)),
            }
        }

        pub fn write_text(&self, text: &str) -> Result<(), SaveError> {
            let tmp = self.dir.join(format!("{FILE}.tmp"));
            fs::create_dir_all(&self.dir)
                .and_then(|_| fs::write(&tmp, text))
                .and_then(|_| fs::rename(&tmp, self.path()))
                .map_err(SaveError::Io)
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod store {
    use super::SaveError;

    const KEY: &str = "tides_of_khah.save";

    #[derive(Clone, Debug)]
    pub struct Store;

    fn local_storage() -> Result<web_sys::Storage, SaveError> {
        web_sys::window()
            .ok_or(SaveError::Web("没有 window"))?
            .local_storage()
            .map_err(|_| SaveError::Web("访问被拒绝"))?
            .ok_or(SaveError::Web("localStorage 已关闭"))
    }

    impl Store {
        pub fn locate() -> Self {
            Store
        }

        pub fn read_text(&self) -> Result<Option<String>, SaveError> {
            local_storage()?.get_item(KEY).map_err(|_| SaveError::Web("读取失败"))
        }

        pub fn write_text(&self, text: &str) -> Result<(), SaveError> {
            local_storage()?.set_item(KEY, text).map_err(|_| SaveError::Web("写入失败（空间不足？）"))
        }
    }
}

/// Set while the world is a real playthrough (not the title, not a dev stage).
#[derive(Resource)]
pub struct Autosave {
    pub enabled: bool,
    timer: f32,
}

pub fn plugin(app: &mut App) {
    app.insert_resource(SaveSlot::locate())
        .insert_resource(Autosave { enabled: !dev_run(), timer: 0.0 })
        .add_systems(OnEnter(Area::Mine), save_now)
        .add_systems(OnEnter(Area::Town), save_now)
        .add_systems(Update, (save_on_close, save_on_timer));
}

/// Staged / screenshot runs are fake worlds; they must not touch the save.
pub fn dev_run() -> bool {
    std::env::var_os("TIDES_STAGE").is_some() || std::env::var_os("TIDES_CAPTURE").is_some()
}

fn store(slot: &SaveSlot, auto: &Autosave, run: &Run) {
    if !auto.enabled {
        return;
    }
    if let Err(e) = slot.write(run) {
        warn!("autosave failed: {e}");
    }
}

fn save_now(slot: Res<SaveSlot>, auto: Res<Autosave>, run: Res<RunRes>) {
    store(&slot, &auto, &run);
}

/// A window just closed: something was decided or bought, keep it.
fn save_on_close(modal: Res<Modal>, mut was_open: Local<bool>, slot: Res<SaveSlot>, auto: Res<Autosave>, run: Res<RunRes>) {
    let open = modal.is_open();
    if *was_open && !open {
        store(&slot, &auto, &run);
    }
    *was_open = open;
}

fn save_on_timer(time: Res<Time>, mut auto: ResMut<Autosave>, slot: Res<SaveSlot>, run: Res<RunRes>) {
    auto.timer += time.delta_secs();
    if auto.timer >= AUTOSAVE_SECS {
        auto.timer = 0.0;
        store(&slot, &auto, &run);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::fs;

    use super::*;

    fn temp_slot(name: &str) -> SaveSlot {
        let dir = std::env::temp_dir().join(format!("tides_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        SaveSlot { store: store::Store { dir } }
    }

    #[test]
    fn missing_save_is_none_not_an_error() {
        let slot = temp_slot("missing");
        assert!(!slot.store.path().is_file());
        assert!(matches!(slot.read(), Ok(None)));
    }

    #[test]
    fn write_then_read_roundtrips() {
        let slot = temp_slot("roundtrip");
        let mut run = Run::new("迪丝");
        run.world.gold = 77;
        run.drain_events();
        slot.write(&run).unwrap();
        assert!(slot.store.path().is_file());
        assert_eq!(slot.read().unwrap(), Some(run));
        assert!(!slot.store.dir.join("save.json.tmp").exists());
        fs::remove_dir_all(&slot.store.dir).unwrap();
    }

    #[test]
    fn corrupt_save_is_reported() {
        let slot = temp_slot("corrupt");
        fs::create_dir_all(&slot.store.dir).unwrap();
        fs::write(slot.store.path(), "{ nope").unwrap();
        assert!(matches!(slot.read(), Err(SaveError::Load(LoadError::Corrupt(_)))));
        fs::remove_dir_all(&slot.store.dir).unwrap();
    }
}
