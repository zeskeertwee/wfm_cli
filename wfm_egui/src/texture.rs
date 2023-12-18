use std::path::PathBuf;

use eframe::egui::{ImageSource, TextureId};
use eframe::egui::{Image, Frame};

pub enum TextureSource {
    Url(String),
    Data(Vec<u8>),
    File(PathBuf),
}

pub struct Texture {
    id: TextureId,
}

async fn get_data(url: String) -> anyhow::Result<Vec<u8>> {
    Ok(reqwest::get(url).await?.bytes().await?.to_vec())
}

pub fn load_texture(source: ImageSource) -> TextureData {
    return TextureData {
        image: Image::new(source),
    };
}

pub fn alloc_texture(data: TextureData, frame: &Frame) -> Texture {
    let id = frame.alloc_texture(data.image);
    Texture { id }
}

impl TextureSource {
    pub async fn load(self) -> anyhow::Result<TextureData> {
        load_texture(self).await
    }
}

impl TextureData {
    pub fn allocate(self, frame: &Frame) -> Texture {
        alloc_texture(self, frame)
    }
}

impl Texture {
    pub fn texture_id(&self) -> &TextureId {
        &self.id
    }
}
