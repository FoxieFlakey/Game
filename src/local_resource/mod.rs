// A helper module for accessing resources belonging to diff thread
// which is !Send and/or !Sync.

use std::{
    cell::UnsafeCell,
    fmt::Display,
    marker::PhantomData,
    panic::Location,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, ThreadId},
    time::Instant,
};

use tokio::{
    select,
    sync::{mpsc, oneshot},
    time::sleep_until,
};

#[derive(Clone)]
pub struct ResourceInfo {
    pub created_at: &'static Location<'static>,
    pub name: String,
}

impl Display for ResourceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Resource '{}' (created at {})",
            self.name, self.created_at
        )
    }
}

struct Shared<T> {
    is_shutdown: AtomicBool,
    data: UnsafeCell<T>,
    info: ResourceInfo,
    owning_thread: ThreadId,
}

pub struct LocalResource<T> {
    shared: Arc<Shared<T>>,
    request_receiver: mpsc::Receiver<Box<dyn FnOnce(&T) + Send>>,
    _not_send_sync: PhantomData<*mut u8>,
}

#[derive(Clone)]
pub struct Accessor<T> {
    shared: Arc<Shared<T>>,
    request_sender: mpsc::Sender<Box<dyn FnOnce(&T) + Send>>,
}

unsafe impl<T> Send for Accessor<T> {}
unsafe impl<T> Sync for Accessor<T> {}

impl<T> Accessor<T> {
    #[track_caller]
    pub fn with<R, F>(&self, closure: F) -> impl Future<Output = R>
    where
        R: Send + 'static,
        F: FnOnce(&T) -> R + Send + 'static,
    {
        let caller = Location::caller();
        if self.shared.is_shutdown.load(Ordering::Relaxed) {
            crate::warn!("Attempting to access resource that is gone");
            crate::warn!("Called by {caller}");
            panic!("Attempting to access resource that is gone");
        }

        async move {
            if self.shared.owning_thread == thread::current_id() {
                // SAFETY: This is the thread that owns the resource, just use it directly
                // there is no &mut access anywhere
                let reference = unsafe { self.shared.data.as_ref_unchecked() };
                return closure(reference);
            }

            let (sender, receiver) = oneshot::channel();
            let resource_info = self.shared.info.clone();

            let ret = self.request_sender.send(Box::new(move |x| {
                if let Err(_) = sender.send(closure(x)) {
                    crate::warn!("Cannot send response for accessing {resource_info}, receiving side is closed");
                    crate::warn!("Called by {caller}");
                }
            })).await;

            if let Err(_) = ret {
                panic!("Cannot send request to owning thread");
            }

            match receiver.await {
                Ok(ret) => return ret,
                Err(e) => {
                    crate::fatal!(
                        "Cannot receive result from requesting access to {}",
                        self.shared.info
                    );
                    crate::fatal!("Called by {caller}");
                    panic!("No response from resource owner: {}", e)
                }
            }
        }
    }
}

impl<T> Drop for LocalResource<T> {
    fn drop(&mut self) {
        self.shared.is_shutdown.store(true, Ordering::Relaxed);
    }
}

impl<T> LocalResource<T> {
    pub fn get(&self) -> &T {
        // SAFETY: This is the thread that owns the resource, just use it directly
        // there no &mut that can happen caused by other threads via sending request
        // as polling only give immutable access to other
        //
        // The guarantee that caller thread own this one resource is enforced by
        // !Send and !Sync
        unsafe { self.shared.data.as_ref_unchecked() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        // SAFETY: This is the thread that owns the resource, just use it directly
        // the &mut guarantee nothing accessing the resource itself
        //
        // The guarantee that caller thread own this one resource is enforced by
        // !Send and !Sync
        unsafe { self.shared.data.as_mut_unchecked() }
    }

    #[track_caller]
    pub fn new<Name: Into<String>>(name: Name, data: T) -> (Self, Accessor<T>) {
        let (sender, receiver) = mpsc::channel(500);
        let shared = Arc::new(Shared {
            info: ResourceInfo {
                name: name.into(),
                created_at: Location::caller(),
            },
            owning_thread: thread::current().id(),
            is_shutdown: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        });

        (
            Self {
                request_receiver: receiver,
                _not_send_sync: PhantomData,
                shared: shared.clone(),
            },
            Accessor {
                request_sender: sender,
                shared: shared,
            },
        )
    }

    // Poll for new requests to access the resource, continuously.
    //
    // # Cancelation safety
    // This function is cancelation safe, no requests lost if canceled
    pub async fn poll_loop(&mut self) {
        loop {
            let Some(req) = self.request_receiver.recv().await else {
                return
            };

            // SAFETY: Self being !Send and !Sync ensures that that reference to the data
            // doesnt violate the safety of it (which is assumed to be !Send and !Sync)
            // Handle request for accessing
            let reference = unsafe { self.shared.data.as_ref_unchecked() };
            req(reference)
        }
    }
}
