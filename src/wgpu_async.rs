use std::ops::Deref;

use wgpu::{CommandBuffer, Device, PollStatus, PollType, Queue};

#[derive(Clone)]
pub struct AsyncQueue {
    device: Device,
    queue: Queue,
}

impl Deref for AsyncQueue {
    type Target = Queue;

    fn deref(&self) -> &Self::Target {
        &self.queue
    }
}

impl AsyncQueue {
    pub fn new(device: Device, queue: Queue) -> Self {
        Self { device, queue }
    }

    pub async fn submit<I: IntoIterator<Item = CommandBuffer>>(&self, command_buffers: I) {
        let submission_id = self.queue.submit(command_buffers);
        let device = self.device.clone();
        tokio::task::spawn_blocking(move || {
            match device
                .poll(PollType::Wait {
                    submission_index: Some(submission_id),
                    timeout: None,
                })
                .unwrap()
            {
                PollStatus::QueueEmpty | PollStatus::WaitSucceeded => (),
                PollStatus::Poll => panic!("intended to wait not poll!"),
            }
        })
        .await
        .unwrap();
    }
}