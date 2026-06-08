use std::fmt::Display;

use futures::{StreamExt, stream::FuturesUnordered};

use crate::{
    registry::Registry,
    runtimes, states,
    util::{
        error::{CustomError, CustomErrorExt},
        identifier::Identifier,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum SingleTextureLoadError {
    #[error("Cannot load image at {path}")]
    FailedToLoadImage {
        path: String,
        #[source]
        error: CustomError<image::ImageError>,
    },
}

#[derive(thiserror::Error, Debug)]
pub struct TextureLoadError {
    pub failures: Vec<CustomError<SingleTextureLoadError>>,
}

impl Display for TextureLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There are {} textures failed to load (",
            self.failures.len()
        )?;
        for failure in &self.failures {
            match failure.get_err() {
                SingleTextureLoadError::FailedToLoadImage { path, error } => {
                    write!(f, "texture {path} failed to load because of '{error}', ")?;
                }
            }
        }
        write!(f, ")")?;
        Ok(())
    }
}

pub async fn load() -> Result<Registry<wgpu::Texture>, CustomError<TextureLoadError>> {
    let mut registry = Registry::new();

    let textures: [(&str, &[u8]); 1] = [("background", include_bytes!("../resources/image.png"))];

    let mut results = Vec::new();
    let mut tasks = FuturesUnordered::new();
    const LOAD_CONCURRENY_COUNT: usize = 8;

    for (path, bytes) in textures {
        if tasks.len() >= LOAD_CONCURRENY_COUNT {
            if let Some(ret) = tasks.next().await {
                match ret {
                    Ok((identifier, texture)) => registry.add(identifier, texture),
                    Err(e) => results.push(e),
                }
            }
        }

        let future = async move {
            let identifier = Identifier::new(path);
            match runtimes::compute::exec(move || {
                Ok(states::data_loader::get().load_texture(image::load_from_memory(bytes)?))
            })
            .await
            {
                Err(e) => Err(SingleTextureLoadError::FailedToLoadImage {
                    path: identifier.to_string(),
                    error: e,
                }
                .into_custom_err()),

                Ok(tex) => Ok((Identifier::new(path), tex)),
            }
        };

        tasks.push(future);
    }

    // Finish the rest of futures that already in loading
    while let Some(ret) = tasks.next().await {
        match ret {
            Ok((identifier, texture)) => registry.add(identifier, texture),
            Err(e) => results.push(e),
        }
    }

    if results.len() > 0 {
        Err(TextureLoadError { failures: results }.into_custom_err())
    } else {
        Ok(registry)
    }
}
