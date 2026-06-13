use std::fmt::Display;

use crate::{
    registries::util,
    registry::Registry,
    runtimes, states,
    util::identifier::Identifier,
};

#[derive(Debug, thiserror::Error)]
pub enum SingleTextureLoadError {
    #[error("Cannot load image at {path}")]
    FailedToLoadImage {
        path: String,
        #[source]
        error: image::ImageError,
    },
}

#[derive(thiserror::Error, Debug)]
pub struct TextureLoadError {
    pub failures: Vec<anyhow::Error>,
}

impl Display for TextureLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There are {} textures failed to load (",
            self.failures.len()
        )?;
        for failure in &self.failures {
            match failure.downcast_ref::<SingleTextureLoadError>().unwrap() {
                SingleTextureLoadError::FailedToLoadImage { path, error } => {
                    write!(f, "texture {path} failed to load because of '{error}', ")?;
                }
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

pub async fn load() -> anyhow::Result<Registry<wgpu::Texture>> {
    let textures: [(&str, &[u8]); 1] = [("background", include_bytes!("../resources/image.png"))];

    util::build_registry(textures.into_iter(), |(path, bytes)| async move {
        let identifier = Identifier::new(path);
        match runtimes::compute::exec(move || {
            Ok(states::data_loader::get().load_texture(image::load_from_memory(bytes)?))
        })
        .await
        {
            Err(e) => Err(SingleTextureLoadError::FailedToLoadImage {
                path: identifier.to_string(),
                error: e,
            }.into()),

            Ok(tex) => Ok((Identifier::new(path), tex)),
        }
    })
    .await
    .map_err(|failures| TextureLoadError { failures }.into())
}
