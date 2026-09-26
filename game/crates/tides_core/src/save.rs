//! Versioned save format. Only (de)serialisation lives here; where the
//! bytes go (a file natively, localStorage on the web) is the engine's job.

use serde::{Deserialize, Serialize};

use crate::run::Run;

pub const SAVE_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    run: Run,
}

#[derive(Debug)]
pub enum LoadError {
    Corrupt(serde_json::Error),
    /// Written by a newer build; refuse rather than silently lose data.
    TooNew(u32),
}

pub fn encode(run: &Run) -> String {
    let file = SaveFile { version: SAVE_VERSION, run: run.clone() };
    // Run is plain data; serialisation cannot fail.
    serde_json::to_string_pretty(&file).expect("Run serialises")
}

pub fn decode(text: &str) -> Result<Run, LoadError> {
    let file: SaveFile = serde_json::from_str(text).map_err(LoadError::Corrupt)?;
    if file.version > SAVE_VERSION {
        return Err(LoadError::TooNew(file.version));
    }
    Ok(file.run)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::Material;
    use crate::loot::{DropSource, generate};

    #[test]
    fn roundtrip_preserves_run() {
        let mut run = Run::new("迪丝");
        let mut rng = fastrand::Rng::with_seed(4);
        let item = generate(DropSource::Elite, 1, &mut rng);
        let id = item.id;
        run.add_item(item);
        run.equip(id).unwrap();
        run.world.gold = 321;
        run.add_materials(&[(Material::Rare, 2)].into());
        run.drain_events();
        assert_eq!(decode(&encode(&run)).unwrap(), run);
    }

    #[test]
    fn rejects_garbage_and_future_versions() {
        assert!(matches!(decode("not json"), Err(LoadError::Corrupt(_))));
        let future = encode(&Run::new("t")).replace("\"version\": 1", "\"version\": 99");
        assert!(matches!(decode(&future), Err(LoadError::TooNew(99))));
    }
}
