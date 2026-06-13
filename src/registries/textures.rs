use std::fmt::Display;

use anyhow::Context;

use crate::{
    error, registries::util, registry::Registry, runtimes, states, util::identifier::Identifier,
};

#[derive(Debug, thiserror::Error)]
pub enum SingleTextureLoadError {
    #[error("Cannot load image")]
    FailedToLoadImage {
        path: String,
        #[source]
        error: anyhow::Error,
    },
}

#[derive(Debug)]
pub struct TextureLoadError {
    pub failures: Vec<(Identifier, anyhow::Error)>,
}

impl std::error::Error for TextureLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.failures.iter().map(|x| x.1.as_ref()).next()
    }
}

impl Display for TextureLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There are {} textures failed to load",
            self.failures.len()
        )
    }
}

pub async fn load() -> anyhow::Result<Registry<wgpu::Texture>> {
    let textures: [(&str, &[u8]); 1] = [("background", include_bytes!("../resources/image.png"))];

    util::build_registry(textures.into_iter(), |(path, bytes)| async move {
        let identifier = Identifier::new(path);
        match runtimes::compute::exec(move || {
            let image = image::load_from_memory(bytes).with_context(|| format!("Reading image"))?;
            Ok(states::data_loader::get().load_texture(image))
        })
        .await
        {
            Err(e) => Err(SingleTextureLoadError::FailedToLoadImage {
                path: identifier.to_string(),
                error: e,
            })
            .with_context(|| format!("Loading texture {identifier}"))
            .map_err(|e| (identifier, e)),

            Ok(tex) => Ok((Identifier::new(path), tex)),
        }
    })
    .await
    .inspect_err(|failures| {
        error!("Errors while these textures");

        for (i, (identifier, error)) in failures.iter().enumerate() {
            error!("{i}: {identifier}: {error:#}");
        }
    })
    .map_err(|failures| TextureLoadError { failures }.into())
}
