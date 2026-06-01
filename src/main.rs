#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]

use std::{cell::RefCell, sync::Arc};

use crate::{local_resource::LocalResource, util::ErrorWithContext};

mod local_resource;
mod logging;
mod runtimes;
mod rendering;
mod util;

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

    let cause = ErrorWithContext::new("Writing error: disk full");
    let error_while_cleaning = ErrorWithContext::new("Cannot close file: unknown file descriptor");
    let error = ErrorWithContext::with_cause("Cannot save file", cause)
        .add_suppressed(error_while_cleaning);

    info!("err: {}", error);
}





