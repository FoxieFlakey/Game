use crate::util::error::CustomError;

// Contains all builtin registries
// like textures
pub struct Registries {
    
}

#[derive(Debug, thiserror::Error)]
#[error("failed to initialize registries")]
pub struct LoadError {
}

pub async fn load_registries() -> Result<Registries, CustomError<LoadError>> {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    Ok(Registries {})
}


