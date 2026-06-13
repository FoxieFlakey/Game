use anyhow::Context;

use crate::registry::Registry;

mod textures;
mod util;

// Contains all builtin registries
// like textures
pub struct Registries {
    pub textures: Registry<wgpu::Texture>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to load textures")]
    TextureLoad(
        #[from]
        #[source]
        textures::TextureLoadError,
    ),
}

pub async fn load_registries() -> anyhow::Result<Registries> {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    crate::info!("Loading registries");
    Ok(Registries {
        textures: textures::load()
            .await
            .inspect(|_| crate::info!("Textures registry loaded!"))
            .context("Loading textures")?,
    })
}
