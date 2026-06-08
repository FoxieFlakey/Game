// A helper module for accessing resources belonging to diff thread
// which is !Send and/or !Sync.

use std::{
    cell::{Ref, RefCell, RefMut, UnsafeCell},
    fmt::Display,
    marker::PhantomData,
    panic::Location,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, ThreadId},
};

use tokio::sync::{mpsc, oneshot};

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
    // Accessed only by thread who owns
    // the resource
    data: UnsafeCell<RefCell<T>>,
    info: ResourceInfo,
    owning_thread: ThreadId,
}

unsafe impl<T> Send for Shared<T> {}
unsafe impl<T> Sync for Shared<T> {}

impl<T> Shared<T> {
    // Panics if called on other thread except the owner
    fn get_data(&self) -> &RefCell<T> {
        assert_eq!(
            self.owning_thread,
            thread::current_id(),
            "Attempted to access resource belonging to other thread"
        );
        // SAFETY: There will be no mut access, and the data only
        // accessed by one thread which owns it
        unsafe { self.data.as_ref_unchecked() }
    }
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
                let reference = &self.shared.get_data().try_borrow()
                    .expect(format!("Caller {caller} attempted to borrow resource while its mutably borrowed (both is on same thread)").as_str());
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
    pub fn get<'a>(&'a self) -> Ref<'a, T> {
        // This is the thread that owns the resource
        // there won't be panic due LocalResource can't
        // be sent to other thread
        self.shared.get_data().borrow()
    }

    // May panics following RefCell's normal
    // behaviour if there Ref/RefMut existed
    // before this call
    pub fn get_mut<'a>(&'a mut self) -> RefMut<'a, T> {
        // This is the thread that owns the resource
        // there won't be panic due LocalResource can't
        // be sent to other thread
        self.shared.get_data().borrow_mut()
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
            data: UnsafeCell::new(RefCell::new(data)),
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
                return;
            };

            let reference = self.get();
            req(&reference)
        }
    }
}
