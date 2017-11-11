use std::collections::HashMap;
use piston_window::G2dTexture;

pub struct AssetManager {
    textures: HashMap<String, G2dTexture>
}

impl AssetManager {
    pub fn new() -> AssetManager {
        AssetManager { textures: HashMap::new() }
    }

    pub fn add_texture(&mut self, key: &'static str, texture: G2dTexture) {
        self.textures.insert(key.to_string(), texture);
    }

    pub fn get_texture(&self, name: &'static str) -> Option<&G2dTexture> {
        self.textures.get(&name.to_string())
    }

    pub fn get_texture_mut(&mut self, name: &'static str) -> Option<&mut G2dTexture> {
        self.textures.get_mut(&name.to_string())
    }
}