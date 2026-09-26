//! Lazily pixelated copies of painted images (portraits, visions). Hand out a
//! handle immediately; its pixels are filled in once the source has loaded.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::paint::{Canvas, pixelate};

#[derive(Resource, Default)]
pub struct Pixelated {
    ready: HashMap<(String, u32), Handle<Image>>,
    pending: Vec<(Handle<Image>, Handle<Image>, u32)>,
}

impl Pixelated {
    /// A handle that will show `path` downsampled to `low_res` px.
    pub fn get(&mut self, path: &str, low_res: u32, assets: &AssetServer, images: &mut Assets<Image>) -> Handle<Image> {
        let key = (path.to_string(), low_res);
        if let Some(h) = self.ready.get(&key) {
            return h.clone();
        }
        let out = images.add(Canvas::new(1, 1).into_image());
        self.pending.push((assets.load(path.to_string()), out.clone(), low_res));
        self.ready.insert(key, out.clone());
        out
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Pixelated>().add_systems(Update, resolve);
}

fn resolve(mut px: ResMut<Pixelated>, mut images: ResMut<Assets<Image>>) {
    if px.pending.is_empty() {
        return;
    }
    let mut still = Vec::new();
    for (src, out, res) in std::mem::take(&mut px.pending) {
        match images.get(&src).and_then(|img| pixelate(img, res)) {
            Some(small) => {
                let _ = images.insert(&out, small);
            }
            None => still.push((src, out, res)),
        }
    }
    px.pending = still;
}
