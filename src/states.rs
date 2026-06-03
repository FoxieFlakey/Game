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

// Rendering engine, only accessible from main thread
define_state!(
    renderer,
    crate::local_resource::Accessor<crate::rendering::Renderer>
);
