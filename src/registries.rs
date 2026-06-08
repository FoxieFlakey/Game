// Contains all builtin registries
// like textures
pub struct Registries {
    
}

pub async fn load_registries() -> Registries {
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    Registries {}
}


