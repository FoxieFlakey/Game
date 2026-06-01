#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]

use std::{cell::RefCell, sync::Arc};

use crate::local_resource::LocalResource;

mod local_resource;
mod logging;
mod runtimes;
mod rendering;

fn main() {
    logging::init();
    crate::info!("Hello, world!");

    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(x) => {
            runtimes::main::set(x);
            runtimes::main::get().block_on(async_main())
        }

        Err(e) => {
            crate::fatal!("Error initializing tokio main runtime: {e}");
        }
    }
}

async fn async_main() {
    runtimes::init();
    if let Err(e) = rendering::init() {
        fatal!("Cannot initialize rendering module: {}", e);
        return
    }


}
