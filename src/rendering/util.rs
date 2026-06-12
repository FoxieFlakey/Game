use std::{
    sync::{
        mpsc,
        oneshot::{self, TryRecvError},
    },
    thread,
};

use wgpu::SubmissionIndex;

pub fn wait_device(device: &wgpu::Device, id: wgpu::SubmissionIndex) {
    match device.poll(wgpu::PollType::Wait {
        submission_index: Some(id),
        timeout: None,
    }) {
        Ok(wgpu::PollStatus::WaitSucceeded) | Ok(wgpu::PollStatus::QueueEmpty) => return,
        Ok(wgpu::PollStatus::Poll) => {
            panic!("This shouldn't occur!");
        }

        Err(e) => {
            panic!("Cannot wait for device to complete submission: {e}");
        }
    }
}

pub struct DevicePoller {
    // If option is None, then the thread is notified
    // to shutdown, else do normally
    sender: mpsc::Sender<Option<(SubmissionIndex, oneshot::Sender<()>)>>,
    device: wgpu::Device,
    thread: Option<thread::JoinHandle<()>>,
}

impl DevicePoller {
    pub fn new(device: wgpu::Device) -> Self {
        let (sender, receiver) = mpsc::channel();

        Self {
            sender: sender,
            device: device.clone(),
            thread: Some(thread::spawn(move || {
                while let Ok(Some((index, on_complete))) = receiver.recv() {
                    wait_device(&device, index);

                    // Ignore the return value, if it fails
                    // let it fails, the one pollling for 'index'
                    // might dont care anymore
                    let _ = on_complete.send(());
                }
            })),
        }
    }

    pub fn create_poll(&self, index: wgpu::SubmissionIndex) -> SubmissionPoller {
        let (oneshot_sender, oneshot_receiver) = oneshot::channel();
        self.sender
            .send(Some((index.clone(), oneshot_sender)))
            .unwrap();

        SubmissionPoller {
            device: self.device.clone(),
            submission_index: index,
            receiver: Some(oneshot_receiver),
        }
    }
}

impl Drop for DevicePoller {
    fn drop(&mut self) {
        self.sender.send(None).unwrap();
        self.thread.take().unwrap().join().unwrap();
    }
}

pub struct SubmissionPoller {
    device: wgpu::Device,
    submission_index: wgpu::SubmissionIndex,
    receiver: Option<oneshot::Receiver<()>>,
}

impl SubmissionPoller {
    // True if specific submission completed
    // else false
    pub fn poll(&mut self) -> bool {
        match self.receiver.take() {
            Some(receiver) => match receiver.try_recv() {
                Ok(_) => true,
                Err(TryRecvError::Empty(receiver)) => {
                    self.receiver = Some(receiver);
                    false
                }
                Err(e) => panic!("Cannot poll status of the submission: {e}"),
            },

            // No receiver mean its done
            None => true,
        }
    }

    // Wait for the submission to complete
    // blockingly, and directly poll the
    // index itself. Avoiding synchronizing
    // to other thread under assumption that
    // when something waits that mean they're
    // kind of important to complete
    pub fn wait(self) {
        wait_device(&self.device, self.submission_index);
    }
}
