macro_rules! define_runtime {
    ($name:ident) => {
        pub mod $name {
            use std::{sync::OnceLock, panic::Location};
            use tokio::{runtime::Runtime, task::JoinHandle, sync::oneshot};

            static RUNTIME: OnceLock<Runtime> = OnceLock::new();

            #[allow(unused)]
            pub fn set(runtime: Runtime) {
                RUNTIME.set(runtime)
                    .ok()
                    .expect(&format!("cannot set runtime {}", stringify!($name)))
            }

            #[allow(unused)]
            pub fn spawn<F: Future + Send + 'static>(future: F) -> JoinHandle<F::Output>
               where F::Output: Send + 'static
            {
                get().spawn(future)
            }

            #[allow(unused)]
            #[track_caller]
            pub fn exec<F: Future + Send>(future: F) -> impl Future<Output = F::Output>
               where F::Output: Send
            {
                async fn exec_impl<F: Future + Send>(future: F, caller: &'static Location<'static>) -> F::Output
                   where F::Output: Send
                {
                    let (send, recv) = oneshot::channel();
                    tokio_scoped::scoped(get().handle())
                        .scope(|scope| {
                            scope.spawn(async move {
                                if let Err(_) = send.send(future.await) {
                                    panic!("Cannot send result of runtime exec on {} called by {caller}", stringify!($name))
                                }
                            });
                        });

                    match recv.await {
                        Ok(ret) => ret,
                        Err(e) => {
                            panic!("Cannot receive result of runtime exec on {} called by {caller}: {e}", stringify!($name))
                        }
                    }
                }

                exec_impl(future, Location::caller())
            }

            #[allow(unused)]
            pub fn get() -> &'static Runtime {
                RUNTIME.get()
                    .expect(&format!("Runtime {} not present yet", stringify!($name)))
            }

            #[allow(unused)]
            pub fn init_default() {
                match tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                {
                    Ok(x) => set(x),
                    Err(e) => crate::fatal!("Error initializing {} tokio runtime: {e}", stringify!(name))
                }
            }
        }
    };
}

// Main thread
// NOTE: It has async runtime but it
// run lots of sync codes so too many
// async stuffs shouldn't be run here.
// so response time for async stuffs
// might be worse as entire rendering
// path is sync code, when it came
// the time to render.
define_runtime!(main);

// Miscellanous background stuffs
define_runtime!(background);

pub fn init() {
    self::background::init_default();
}
