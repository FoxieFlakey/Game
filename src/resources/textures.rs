use anyhow::{Context, anyhow};

use crate::{
    error, registries::util, registry::Registry, runtimes, screen, states,
    util::identifier::Identifier,
};

struct Texture {
    identifier: Identifier,
    raw_bytes: &'static [u8],
}

impl Texture {
    pub fn new(identifier: Identifier, bytes: &'static [u8]) -> Self {
        Self {
            identifier: identifier,
            raw_bytes: bytes,
        }
    }
}

async fn load_list(textures: &[Texture]) -> anyhow::Result<Registry<wgpu::Texture>> {
    util::build_registry(textures.into_iter(), |description| async move {
        let identifier = description.identifier.clone();
        let raw_bytes = description.raw_bytes;
        match runtimes::compute::exec(move || {
            let image =
                image::load_from_memory(raw_bytes).with_context(|| format!("Reading image"))?;
            Ok::<_, anyhow::Error>(states::data_loader::get().load_texture(image))
        })
        .await
        {
            Err(e) => Err((identifier, e)),
            Ok(tex) => Ok((description.identifier.clone(), tex)),
        }
    })
    .await
    .inspect_err(|failures| {
        error!("Errors while these textures");

        for (i, (identifier, error)) in failures.iter().enumerate() {
            error!("{i}: {identifier}: {error:#}");
        }
    })
    .map_err(|_| anyhow!("Cannot load some textures, see logs"))
}

#[rustfmt::skip]
pub async fn load() -> anyhow::Result<Registry<wgpu::Texture>> {
    load_list(&[
        Texture::new(Identifier::new("ui/image"), include_bytes!("ui/image.png"))
    ]).await
}

#[rustfmt::skip]
pub async fn early_load() -> anyhow::Result<Registry<wgpu::Texture>> {
    load_list(&[
        Texture::new(
            screen::LoadingScreen::ICON_TEXTURE_ID,
            include_bytes!("early/loading_icon.png"),
        )
    ])
    .await
}
