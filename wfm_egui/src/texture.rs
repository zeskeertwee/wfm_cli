use std::path::PathBuf;

use eframe::egui::TextureId;
use eframe::epi;

pub enum TextureSource {
    Url(String),
    Data(Vec<u8>),
    File(PathBuf),
}

pub struct TextureData {
    image: epi::Image,
}

pub struct Texture {
    id: TextureId,
}

async fn get_data(url: String) -> anyhow::Result<Vec<u8>> {
    Ok(reqwest::get(url).await?.bytes().await?.to_vec())
}

pub async fn load_texture(source: TextureSource) -> anyhow::Result<TextureData> {
    let data = match source {
        TextureSource::Url(url) => get_data(url).await?,
        TextureSource::Data(data) => data,
        TextureSource::File(path) => std::fs::read(path)?,
    };

    let image = image::load_from_memory(&data)?.to_rgba8();
    let epi_image = epi::Image::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        &image.into_raw(),
    );

    Ok(TextureData { image: epi_image })
}

pub fn alloc_texture(data: TextureData, frame: &epi::Frame) -> Texture {
    let id = frame.alloc_texture(data.image);
    Texture { id }
}

impl TextureSource {
    pub async fn load(self) -> anyhow::Result<TextureData> {
        load_texture(self).await
    }
}

impl TextureData {
    pub fn allocate(self, frame: &epi::Frame) -> Texture {
        alloc_texture(self, frame)
    }
}

impl Texture {
    pub fn texture_id(&self) -> &TextureId {
        &self.id
    }
}
