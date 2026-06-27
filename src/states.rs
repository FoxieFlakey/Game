macro_rules! define_state (
    ($name:ident, $type:ty) => {
        pub mod $name {
            use std::sync::OnceLock;

            static STATE: OnceLock<$type> = OnceLock::new();

            #[allow(unused)]
            pub fn set(state: $type) {
                STATE.set(state)
                    .ok()
                    .expect(&format!("Cannot set state for {}", stringify!($name)))
            }

            #[allow(unused)]
            pub fn get() -> &'static ($type) {
                STATE.get()
                    .expect(&format!("State for {} is not initialized", stringify!($name)))
            }
        }
    };
);

// SDL subsystems which only accessible on main thread
define_state!(sdl, crate::local_resource::Accessor<crate::SdlState>);

// Main states the same, only accessible from main thread
define_state!(main, crate::local_resource::Accessor<crate::MainState>);

// Early registry that is initialized and is read only
// since loaded
define_state!(early_registries, crate::registries::EarlyRegistries);

// Rendering engine, only accessible from main thread
define_state!(
    renderer,
    crate::local_resource::Accessor<crate::rendering::Renderer>
);

// Data loader for rendering data accesible anywhere
define_state!(data_loader, crate::rendering::data_loader::DataLoader);

// wgpu::Device referring to active device
// it is accessible anywhere
define_state!(main_dev, wgpu::Device);

// Texture format that renderer will use for lifetime
define_state!(surface_format, wgpu::TextureFormat);

// Stuffs that is only available after boot

// Registries, contains all game registries that is loaded
// its not initialized till game done booting
define_state!(registries, crate::registries::Registries);
