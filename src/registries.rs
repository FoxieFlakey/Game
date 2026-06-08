use crate::{
    registry::Registry,
    util::error::{CustomError, CustomErrorExt},
};

mod textures;

// Contains all builtin registries
// like textures
pub struct Registries {
    pub textures: Registry<wgpu::Texture>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to load textures")]
    TextureLoad(#[from] CustomError<textures::TextureLoadError>),
}

pub async fn load_registries() -> Result<Registries, CustomError<LoadError>> {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    crate::info!("Loading registries");
    Ok(Registries {
        textures: textures::load()
            .await
            .inspect(|_| crate::info!("Textures registry loaded!"))
            .map_err(|e| e.context("loading textures registry"))
            .map_err(CustomError::convert)?,
    })
}
