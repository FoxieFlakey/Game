use anyhow::Context;

use crate::{registry::Registry, resources};

pub mod util;

// Contains all builtin registries
// like textures
pub struct Registries {
    pub textures: Registry<wgpu::Texture>,
    pub shaders: Registry<wgpu::ShaderModule>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to load textures")]
    TextureLoad(
        #[from]
        #[source]
        resources::textures::TextureLoadError,
    ),
}

pub async fn load_registries() -> anyhow::Result<Registries> {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    crate::info!("Loading registries");
    Ok(Registries {
        textures: resources::textures::load()
            .await
            .inspect(|_| crate::info!("Textures registry loaded!"))
            .context("Loading textures")?,
        shaders: resources::shaders::load()
            .await
            .inspect(|_| crate::info!("Shaders registry loaded!"))
            .context("Loading shaders")?,
    })
}

pub struct EarlyRegistries {
    pub textures: Registry<wgpu::Texture>,
    pub shaders: Registry<wgpu::ShaderModule>,
}

// Like load_registries but include only bare minimum
// to initialize minimal subsystems. Such as loading screen
// resources, etc
pub async fn load_early_registries() -> anyhow::Result<EarlyRegistries> {
    crate::info!("Loading early registries");
    Ok(EarlyRegistries {
        textures: resources::textures::early_load()
            .await
            .inspect(|_| crate::info!("Early textures registry loaded!"))
            .context("Loading early textures")?,
        shaders: resources::shaders::early_load()
            .await
            .inspect(|_| crate::info!("Early shaders registry loaded!"))
            .context("Loading early shaders")?,
    })
}
