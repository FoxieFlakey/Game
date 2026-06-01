#![feature(unsafe_cell_access)]
#![feature(current_thread_id)]

use std::{cell::RefCell, mem, sync::Arc};

use crate::local_resource::LocalResource;

mod local_resource;
mod logging;
mod runtimes;

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

    let (mut resource, accessor) = LocalResource::new(
        "Numeric",
        RefCell::new(std::ptr::without_provenance::<bool>(0)),
    );
    let accessor = Arc::new(accessor);
    let accessor2 = accessor.clone();
    runtimes::background::spawn(async move {
        accessor2
            .with(|resource| crate::info!("Value is {}", resource.borrow().addr()))
            .await;
    });

    crate::info!(
        "Main thread: value is {}",
        accessor.with(|resource| resource.borrow().addr()).await
    );
    *resource.get_mut().borrow_mut() = std::ptr::without_provenance(20);

    resource
        .poll_while(std::time::Instant::now() + std::time::Duration::from_secs(1))
        .await;
}
